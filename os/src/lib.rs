#![no_std]

use crate::hardware::{Hardware, KeypadDriver};
use arrayvec::ArrayString;
use core::fmt::Write;
use embedded_graphics::{
	mono_font::{ascii::FONT_6X13, MonoTextStyle},
	prelude::Point,
	text::{Alignment, Text},
	Drawable,
};
pub use epd_waveshare::color::Color;
use hardware::{DisplayDriver, Key, SystemDriver};

pub mod hardware;
mod log;

const WIDTH: i32 = 296;
const HEIGHT: i32 = 128;

pub fn run<D, KB, SYS>(mut hw: Hardware<D, KB, SYS>) -> !
where
	D: DisplayDriver,
	KB: KeypadDriver,
	SYS: SystemDriver,
{
	let _ = hw.display.clear(Color::White);
	hw.display.refresh();
	let style = MonoTextStyle::new(&FONT_6X13, Color::Black);

	let mut string: ArrayString<32> = ArrayString::new();
	write!(
		string,
		"Memory: {}K/{}K",
		hw.system.memory_used() / 1000,
		hw.system.memory_total() / 1000
	)
	.unwrap();
	let _ = Text::with_alignment(
		&string,
		Point::new(WIDTH / 2, HEIGHT / 2),
		style,
		Alignment::Center,
	)
	.draw(&mut hw.display);

	log::info!("drawn, updating");
	hw.display.update();
	log::info!("updated");

	let mut string: ArrayString<64> = ArrayString::new();
	loop {
		hw.keypad.wait_for_key(u64::MAX);
		while let Some(key) = hw.keypad.read_key() {
			if key == Key::Fn {
				string.clear();
				break;
			}

			write!(string, "{key:?}").unwrap();
		}
		let _ = hw.display.clear(Color::White);
		let _ = Text::new(&string, Point::new(0, 100), style).draw(&mut hw.display);
		hw.display.update();
	}
}
