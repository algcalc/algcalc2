#![no_std]

use core::time::Duration;

use crate::hardware::{Hardware, KeypadDriver};
use embedded_graphics::{
	mono_font::{ascii::FONT_6X13_ITALIC, MonoTextStyle}, prelude::{Point, Primitive}, primitives::{Line, PrimitiveStyle}, text::{Alignment, Text}, Drawable
};
pub use epd_waveshare::color::Color;
use hardware::DisplayDriver;

pub mod hardware;
mod log;

const WIDTH: u32 = 296;
const HEIGHT: u32 = 128;

pub fn run<D, KB>(mut hw: Hardware<D, KB>) -> !
where
	D: DisplayDriver,
	KB: KeypadDriver,
{
	let _ = hw.display.clear(Color::White);
	let _ = Line::new(Point::new(10, 50), Point::new(10, 100))
		.into_styled(PrimitiveStyle::with_stroke(Color::Black, 5))
		.draw(&mut hw.display);
	let style = MonoTextStyle::new(&FONT_6X13_ITALIC, Color::Black);

	let _ = Text::with_alignment("Hello Rust!", Point::new(20, 30), style, Alignment::Left).draw(&mut hw.display);
	log::info!("drawn, updating");
	hw.display.update();
	log::info!("updated");
	loop {
		log::info!("{:?}", hw.keypad.read_key(u64::MAX));
	}
}
