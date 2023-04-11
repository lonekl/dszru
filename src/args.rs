use std::process::exit;

pub struct Opt {

	pub files: Vec<String>,
	pub crypt_mode: CryptMode,
	pub verbose: Verbose,
	/// `None` means that password will be on stdin.
	pub password: Password,
	pub progress_bar_length: u16,

}

impl Opt {

	pub fn pass_args() -> Result<Self, Vec<String>> {
		#[derive(PartialEq)]
		enum OptListener {
			No,
			PasswordFile,
			ProgressLength,
		}

		let mut options_enabled = true;
		let mut listening_for_opt_arg = OptListener::No;
		let mut errors = Vec::new();
		let mut files = Vec::new();
		let mut password_file = None;
		let mut password_file_used_twice = false;
		let mut progress_bar_length = None;
		let mut progress_bar_length_used_twice = false;

		let mut opt_help = 0_u16;
		let mut opt_version = 0_u16;
		let mut opt_decrypt = 0_u16;
		let mut opt_verbose = 0_u16;
		let mut opt_display_progress = 0_u16;
		let mut opt_full_password = 0_u16;

		for arg in std::env::args().skip(1) {
			let mut option = false;

			if options_enabled && listening_for_opt_arg == OptListener::No {
				if &arg[0..2] == "--" {
					option = true;

					match arg.as_str() {
						"--" => options_enabled = false,
						"--help" => opt_help += 1,
						"--version" => opt_version += 1,
						"--decrypt" => opt_decrypt += 1,
						"--display-progress" | "--very-verbose" => opt_display_progress += 1,
						"--verbose" => opt_verbose += 1,
						"--full-stdin-password" => opt_full_password += 1,
						"--password-file" => listening_for_opt_arg = OptListener::PasswordFile,
						"--progress-length" => listening_for_opt_arg = OptListener::ProgressLength,
						_ => errors.push(format!("No such option as {arg}.")),
					}

				} else if &arg[0..1] == "-" {
					option = true;

					for arg_char in arg.chars().skip(1) {

						match arg_char {
							'd' => opt_decrypt += 1,
							'v' => opt_verbose += 1,
							'V' => opt_display_progress += 1,
							'f' => opt_full_password += 1,
							'p' => if listening_for_opt_arg == OptListener::No {
								listening_for_opt_arg = OptListener::PasswordFile
							} else {
								errors.push(format!("Tried to use -p with other option requiring argument."));
							},
							'P' => if listening_for_opt_arg == OptListener::No {
								listening_for_opt_arg = OptListener::ProgressLength
							} else {
								errors.push(format!("Tried to use -P with other option requiring argument."));
							},
							_ => errors.push(format!("No such option as -{arg_char}.")),
						}

					}

				}
			}

			if !option {
				match listening_for_opt_arg {
					OptListener::PasswordFile => {

						// Kind of
						option = true;
					
						if password_file == None {
							password_file = Some(arg.clone());
						} else {
							password_file_used_twice = true;
						}

						listening_for_opt_arg = OptListener::No;
					},
					OptListener::ProgressLength => {

						option = true;

						if progress_bar_length == None {
							match arg.parse() {
								Ok(number) => progress_bar_length = Some(number),
								Err(_) => errors.push(format!("invalid number for -P option")),
							}
						} else {
							progress_bar_length_used_twice = true;
						}

						listening_for_opt_arg = OptListener::No;
					},
					OptListener::No => {},
				}
			}

			if !option {
				files.push(arg);
			}

		}

		if opt_help > 1 {
			errors.push(format!("--help option was used more than once."));
		}

		if opt_version > 1 {
			errors.push(format!("--version option was used more than once."));
		}

		if opt_decrypt > 1 {
			errors.push(format!("--decrypt, -d option was used more than once."));
		}

		if opt_verbose > 1 {
			errors.push(format!("--verbose, -v option was used more than once."));
		}

		if opt_display_progress > 1 {
			errors.push(format!("--display-progress, --very-verbose, -V option was used more than once."));
		}

		if opt_full_password > 1 {
			errors.push(format!("--full-stdin-password, -f option was used more than once."));
		}

		if password_file_used_twice {
			errors.push(format!("--password-file, -p option was used more than once."));
		}

		if password_file != None && opt_full_password > 0 {
			errors.push(format!("--password-file, -p and --full-stdin-password, -f options were used both at one time (--password-file, -p implies using file as full password (including ending newline))"));
		}

		if progress_bar_length_used_twice {
			errors.push(format!("--progress-length, -P option was used more than once."));
		}

		match listening_for_opt_arg {
			OptListener::PasswordFile => errors.push(format!("--password-file, -p didn't get any file path")),
			OptListener::ProgressLength => errors.push(format!("--progress-length, -P didn't get any number")),
			OptListener::No => {},
		}

		if opt_verbose > 0 && opt_display_progress > 0 {
			errors.push(format!("--verbose, -v and --display-process, --very-verbose, -V options were used at one time."));
		}

		if opt_help > 0 {

			if files.len() != 0 || opt_version != 0 || opt_decrypt != 0 || opt_verbose != 0 || opt_display_progress != 0 || opt_full_password != 0 || password_file != None {
				errors.push(format!("--help option used with other options or arguments."));
			} else if errors.len() == 0 {
				eprint!("{}", include_str!("help message.txt"));
				exit(0)
			}

		}

		if opt_version > 0 {

			if files.len() != 0 || opt_help != 0 || opt_decrypt != 0 || opt_verbose != 0 || opt_display_progress != 0 || opt_full_password != 0 || password_file != None {
				errors.push(format!("--version option used with other options or arguments."));
			} else if errors.len() == 0 {
				eprint!("{}", include_str!("version message.txt"));
				exit(0)
			}

		}

		if errors.len() != 0 {
			return Err(errors);
		}

		Ok(Self {
			files,
			crypt_mode: if opt_decrypt == 0 {
				CryptMode::Encrypt
			} else {
				CryptMode::Decrypt
			},
			verbose: if opt_verbose != 0 {
				Verbose::Verbose
			} else if opt_display_progress != 0 {
				Verbose::VeryVerbose
			} else {
				Verbose::None
			},
			password: match password_file {
				Some(file_path) => Password::File(file_path),
				None => if opt_full_password > 0 {
					Password::FullStdin
				} else {
					Password::StdinOneLine
				},
			},
			progress_bar_length: match progress_bar_length {
				Some(length) => length,
				None => /* default value */ 30,
			}
		})
	}

}

pub enum CryptMode {
	Encrypt,
	Decrypt,
}

#[derive(PartialEq)]
pub enum Verbose {
	None,
	Verbose,
	VeryVerbose,
}

pub enum Password {
	File(String),
	FullStdin,
	StdinOneLine,
}
