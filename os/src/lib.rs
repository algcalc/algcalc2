#![no_std]

use core::time::Duration;

use crate::hardware::{Hardware, KeypadDriver};
use embedded_graphics::{
	draw_target::DrawTarget,
	prelude::{Point, Primitive},
	primitives::{Line, PrimitiveStyle},
	Drawable,
};
pub use epd_waveshare::color::Color;
use hardware::DisplayDriver;

pub mod hardware;
mod log;

pub fn run<D, KB>(mut hw: Hardware<D, KB>) -> !
where
	D: DrawTarget<Color = Color> + DisplayDriver,
	KB: KeypadDriver,
{
	let _ = Line::new(Point::new(10, 50), Point::new(10, 100))
		.into_styled(PrimitiveStyle::with_stroke(Color::Black, 5))
		.draw(&mut hw.display);
	hw.display.update();
	loop {
		hw.keypad.read_key(Duration::MAX);
	}
}
