#![no_std]

use crate::hardware::{Hardware, KeypadDriver};
use arrayvec::ArrayString;
use embedded_graphics::{
	mono_font::{ascii::{FONT_10X20, FONT_6X13, FONT_6X13_ITALIC}, MonoTextStyle}, prelude::{Point, Primitive}, primitives::{Line, PrimitiveStyle}, text::{Alignment, Text}, Drawable
};
pub use epd_waveshare::color::Color;
use hardware::{DisplayDriver, SystemDriver};
use core::fmt::Write;

pub mod hardware;
mod log;

const WIDTH: i32 = 296;
const HEIGHT: i32 = 128;

pub fn run<D, KB, SYS>(mut hw: Hardware<D, KB, SYS>) -> !
where
	D: DisplayDriver,
	KB: KeypadDriver,
	SYS: SystemDriver
{
	let _ = hw.display.clear(Color::White);
	let style = MonoTextStyle::new(&FONT_10X20, Color::Black);

	let mut string: ArrayString<32> = ArrayString::new();
	write!(string, "Memory: {}K/{}K", hw.system.memory_used()/1000, hw.system.memory_total()/1000).unwrap();
	let _ = Text::with_alignment(&string, Point::new(WIDTH/2, HEIGHT/2), style, Alignment::Center).draw(&mut hw.display);

	log::info!("drawn, updating");
	hw.display.update();
	log::info!("updated");

	loop {
		log::info!("{:?}", hw.keypad.read_key(u64::MAX));
	}
}
