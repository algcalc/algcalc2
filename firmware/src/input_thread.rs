use embedded_hal::digital::{InputPin, OutputPin};
use rp_pico::{
	hal::{
		gpio::{DynPinId, FunctionSioInput, FunctionSioOutput, Pin, PullDown},
		Sio, Timer,
	},
	pac,
};

use crate::config;

pub(crate) struct KeypadPins {
	pub(crate) columns: [Pin<DynPinId, FunctionSioInput, PullDown>; config::KEYPAD_COLS],
	pub(crate) rows: [Pin<DynPinId, FunctionSioOutput, PullDown>; config::KEYPAD_ROWS],
}

pub(crate) fn run(mut keypad: KeypadPins, timer: Timer) {
	let pac = unsafe { pac::Peripherals::steal() };
	let mut fifo = Sio::new(pac.SIO).fifo;

	let start = timer.get_counter();
	let mut keypad_last_pressed = [[start; config::KEYPAD_COLS]; config::KEYPAD_ROWS];

	'outer: loop {
		let now = timer.get_counter();
		for (i_row, row) in keypad.rows.iter_mut().enumerate() {
			row.set_high().unwrap();
			for (i_col, column) in keypad.columns.iter_mut().enumerate() {
				if column.is_high().unwrap() {
					let since_pressed = now - keypad_last_pressed[i_row][i_col];
					keypad_last_pressed[i_row][i_col] = now;

					if since_pressed.to_millis() < config::DEBOUNCE_TIME_MS {
						continue;
					}

					row.set_low().unwrap();
					let msg = i_col | (i_row << 16);
					fifo.write_blocking(msg as u32);
					continue 'outer;
				}
			}
			row.set_low().unwrap();
		}
	}
}
