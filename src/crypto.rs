//! This code should work with `overflow-check = false` in profile
#![allow(arithmetic_overflow)]

/// Values for it are: (currently_completed_step: u8, total_steps: u8)
/// currently_completed_step in decrypting counts from total_steps - 1 to zero.
pub type StepFn = fn(u8, u8);

pub struct Dszru<'a> {

	key: &'a [u8],
	steps: u8,

}

impl<'a> Dszru<'a> {

	/// To change key, just create new object.
	pub fn new(key: &'a [u8]) -> Self {

		Self {
			key,
			steps: gen_step_number(key),
		}
	}


	pub fn encrypt(&self, data: &mut [u8], step_fn: StepFn) {

		for step in 0..self.steps {

			// single byte modification

			let mut key_index = data.len() % self.key.len();

			for data_byte in &mut *data {
				let key_byte = self.key[key_index];

				*data_byte += key_byte;
				*data_byte ^= key_byte;
				*data_byte -= step;
				*data_byte ^= key_byte;
				*data_byte ^= step;
				*data_byte += self.steps;

				key_index -= 1;

				if key_index == 0 - 1 {
					key_index = self.key.len() - 1;
				}

			}


			// byte moving

			key_index = 0;
			let mut byte_index = self.key[key_index] as usize % data.len();

			for _ in 0..data.len() {

				let first_byte = data[byte_index];
				let first_byte_index = byte_index;

				key_index += 1;
				if key_index == self.key.len() {
					key_index = 0;
				}

				byte_index += self.key[key_index] as usize;
				byte_index %= data.len();

				let second_byte = data[byte_index];
				data[first_byte_index] = second_byte;
				data[byte_index] = first_byte;

			}

			step_fn(step, self.steps);
		}

	}

	pub fn decrypt_byte_move_counter(&self, key_index: &mut usize, byte_index: &mut usize, data: &[u8]) {

		*key_index -= 1;
		if *key_index == 0 - 1 {
			*key_index = self.key.len() - 1;
		}

		let key_byte_number = self.key[*key_index] as usize;
		if *byte_index >= key_byte_number {
			*byte_index -= key_byte_number;
		} else {
			*byte_index += data.len() - (key_byte_number % data.len());
			*byte_index %= data.len();
		}	

	}

	pub fn decrypt(&self, data: &mut [u8], step_fn: StepFn) {

		for step in 0..self.steps {
			let step = self.steps - step - 1;

			// byte moving

			let mut byte_index = 0;

			let key_scrolls = (data.len() + 1) / self.key.len();
			let mut key_index = (data.len() + 1) % self.key.len();

			for mul_key_index in 0..self.key.len() {

				let mut multiply_by = key_scrolls;
				if mul_key_index < key_index {
					multiply_by += 1;
				}

				byte_index += self.key[mul_key_index] as usize * multiply_by;

			}

			byte_index %= data.len();

//			self.decrypt_byte_move_counter(&mut key_index, &mut byte_index, data);
//			something was wrong but it made it only worse and didn't fix anything

			for _ in 0..data.len() {

				let second_byte = data[byte_index];
				let second_byte_index = byte_index;

				self.decrypt_byte_move_counter(&mut key_index, &mut byte_index, data);

				let first_byte = data[byte_index];
				data[second_byte_index] = first_byte;
				data[byte_index] = second_byte;

			}

			// single byte modification

			key_index = data.len() % self.key.len();

			for data_byte in &mut *data {
				let key_byte = self.key[key_index];

				*data_byte -= self.steps;
				*data_byte ^= step;
				*data_byte ^= key_byte;
				*data_byte += step;
				*data_byte ^= key_byte;
				*data_byte -= key_byte;

				key_index -= 1;

				if key_index == 0 - 1 {
					key_index = self.key.len() - 1;
				}

			}

			step_fn(step, self.steps);
		}

	}

}

fn gen_step_number(key: &[u8]) -> u8 {
	let mut steps = 0;

	for key_byte in key {
		steps += !*key_byte;
	}

	if steps < 128 {
		steps = !steps;
	}

	steps
}
