pub mod crypto;
pub mod args;

use std::fs::File;
use std::io::{Write, Read, stderr, stdin};
use std::process::exit;

use crypto::Dszru;
use args::{Opt, CryptMode, Verbose, Password};

fn main() {
	let opt = match Opt::pass_args() {
		Ok(opt) => opt,
		Err(errors) => {
			eprintln!("Errors in arguments:");
			
			for error in errors {
				eprintln!("Error: {error}");
			}

			exit(1)
		},
	};

	unsafe {
		STEP_FN_PROGRESS_BAR_LENGTH = opt.progress_bar_length;
	}

	fn default_step_fn(_step: u8, _steps: u8) {}

	let mut step_fn: fn(u8, u8) = default_step_fn;
	if opt.verbose == Verbose::VeryVerbose {

		match opt.crypt_mode {
			CryptMode::Encrypt => step_fn = step_fn_encrypting,
			CryptMode::Decrypt => step_fn = step_fn_decrypting,
		}

	}

	let higher_password;
	let dszru = match opt.password {
		Password::File(password_file_path) => match File::open(password_file_path.clone()) {
			Ok(mut password_file) => {
				let mut password = Vec::new();

				match password_file.read_to_end(&mut password) {
					Ok(_) => {

						higher_password = password;
						Dszru::new(&higher_password)
					},
					Err(error) => {
						eprintln!("Error: failed to read password file {password_file_path}, os error {error}.");
						exit(1)
					},
				}
			},
			Err(error) => {
				eprintln!("Error: failed to open password file {password_file_path}, os error {error}.");
				exit(1)
			},
		}
		Password::FullStdin => {
			eprint!("password (end it with ^D): ");
			let _ = stderr().flush();
			let mut password = Vec::new();

			match stdin().read_to_end(&mut password) {
				Ok(_) => {

					eprintln!();

					higher_password = password;
					Dszru::new(&higher_password)
				},
				Err(error) => {
					eprintln!("Error: failed to read password from stdin, os error {error}.");
					exit(1)
				},
			}
		},
		Password::StdinOneLine => {
			eprint!("password: ");
			let _ = stderr().flush();
			let mut password = String::new();

			match stdin().read_line(&mut password) {
				Ok(_) => {
					let _ = password.pop();
					let password = password.into_bytes();

					higher_password = password;
					Dszru::new(&higher_password)
				},
				Err(error) => {
					eprintln!("Error: failed to read string password from stdin, (os?) error {error}.");
					exit(1)
				},
			}
		},
	};

	for input_file_path in opt.files {
		match File::open(&input_file_path) {
			Ok(mut input_file) => {
				let mut output_file_path = input_file_path.clone();
				
				match opt.crypt_mode {
					CryptMode::Encrypt => if output_file_path.ends_with(".udszru") {
						output_file_path.truncate(output_file_path.len() - 7);
					} else {
						output_file_path.push_str(".dszru");
					},
					CryptMode::Decrypt => if output_file_path.ends_with(".dszru") {
						output_file_path.truncate(output_file_path.len() - 6);
					} else {
						output_file_path.push_str(".udszru");
					},
				}

				match File::create(&output_file_path) {
					Ok(mut output_file) => {
						let mut data_buffer = Vec::new();

						match opt.verbose {
							Verbose::Verbose | Verbose::VeryVerbose => match opt.crypt_mode {
								CryptMode::Encrypt => eprintln!("Encrypting {input_file_path} into {output_file_path}."),
								CryptMode::Decrypt => eprintln!("Decrypting {input_file_path} into {output_file_path}."),
							},
							Verbose::None => {},
						}

						match input_file.read_to_end(&mut data_buffer) {
							Ok(_) => {

								match opt.crypt_mode {
									CryptMode::Encrypt => dszru.encrypt(&mut data_buffer, step_fn),
									CryptMode::Decrypt => dszru.decrypt(&mut data_buffer, step_fn),
								}

								match output_file.write(&data_buffer) {
									Ok(_) => {},
									Err(error) => {
										eprintln!("Error: failed to write to {output_file_path} file, os error: {error}");
									},
								}

							},
							Err(error) => {
								eprintln!("Error: failed to read {input_file_path} file, os error: {error}");
							},
						}

					},
					Err(error) => {
						eprintln!("Error: failed to create {output_file_path} file, os error {error}");
					},
				}
			},
			Err(error) => {
				eprintln!("Error: failed to open {input_file_path} file, os error: {error}");
			},
		}

	}

}

static mut STEP_FN_PROGRESS_BAR_LENGTH: u16 = 30;

fn step_fn_encrypting(step: u8, steps: u8) {
	let ratio = step as f32 / steps as f32;
	let precent = ratio * 100.0;
	let progress_bar_completed = step as u16 * unsafe {STEP_FN_PROGRESS_BAR_LENGTH} / steps as u16;
	let progress_bar_uncompleted = unsafe {STEP_FN_PROGRESS_BAR_LENGTH} - progress_bar_completed;

	let mut progress_bar = String::new();
	progress_bar.reserve(unsafe {STEP_FN_PROGRESS_BAR_LENGTH} as usize * 4);

	for _ in 0..progress_bar_completed {
		progress_bar.push('█');
	}

	for _ in 0..progress_bar_uncompleted {
		progress_bar.push('░');
	}

	let mut output = stderr();

	match write!(
		output,
		"\rEncrypting progress [{progress_bar}], {precent}%, {step} step completed out of {steps}.    \x1b[D\x1b[D\x1b[D\x1b[D",
		step = step + 1,
		steps = steps - 1,
	) {
		Ok(_) => {let _ = output.flush();},
		Err(_) => {},
	}

	if step + 1 == steps {
		let fill_length = 51 + 10 + 3 + 3 + unsafe {STEP_FN_PROGRESS_BAR_LENGTH} as usize;
		let fill_bytes = vec![b' '; fill_length];
		let fill_string = String::from_utf8(fill_bytes).unwrap_or(String::new());

		match write!(output, "\r{fill_string}\r") {
			Ok(_) => {},
			Err(_) => {},
		}

	}
}

fn step_fn_decrypting(step: u8, steps: u8) {
	let ratio = 1.0 - step as f32 / steps as f32;
	let precent = ratio * 100.0;
	let progress_bar_step = steps - step;
	let progress_bar_completed = progress_bar_step as u16 * unsafe {STEP_FN_PROGRESS_BAR_LENGTH} / steps as u16;
	let progress_bar_uncompleted = unsafe {STEP_FN_PROGRESS_BAR_LENGTH} - progress_bar_completed;

	let mut progress_bar = String::new();
	progress_bar.reserve(unsafe {STEP_FN_PROGRESS_BAR_LENGTH} as usize * 4);

	for _ in 0..progress_bar_completed {
		progress_bar.push('█');
	}

	for _ in 0..progress_bar_uncompleted {
		progress_bar.push('░');
	}

	let mut output = stderr();

	match write!(
		output,
		"\rDecrypting progress [{progress_bar}], {precent}%, {step} step completed out of {steps}.    \x1b[D\x1b[D\x1b[D\x1b[D",
		step = step + 1,
		steps = steps - 1,
	) {
		Ok(_) => {let _ = output.flush();},
		Err(_) => {},
	}

	if step == 0 {
		let fill_length = 51 + 10 + 3 + 3 + unsafe {STEP_FN_PROGRESS_BAR_LENGTH} as usize;
		let fill_bytes = vec![b' '; fill_length];
		let fill_string = String::from_utf8(fill_bytes).unwrap_or_default();

		let _ = write!(output, "\r{fill_string}\r");

	}

}
