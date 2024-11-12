use core::convert::Infallible;
use embedded_graphics::{
	draw_target::DrawTarget, geometry::Dimensions, primitives::Rectangle, Pixel,
};
use embedded_hal::{
	delay::DelayNs,
	digital::{InputPin, OutputPin},
	spi::SpiDevice,
};
use epd_waveshare::{
	color::Color,
	epd2in9_v2::{Display2in9, Epd2in9},
};
use os::hardware::{DisplayDriver, KeypadDriver};

// DISPLAY

pub(crate) struct Display<SPI, BUSY, DC, RST, DELAY> {
	pub(crate) epd: Epd2in9<SPI, BUSY, DC, RST, DELAY>,
	pub(crate) fb: Display2in9,
}

impl<SPI, BUSY, DC, RST, DELAY> DrawTarget for Display<SPI, BUSY, DC, RST, DELAY> {
	type Color = Color;
	type Error = Infallible;

	fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
	where
		I: IntoIterator<Item = Pixel<Self::Color>>,
	{
		self.fb.draw_iter(pixels)
	}
}

impl<SPI, BUSY, DC, RST, DELAY> Dimensions for Display<SPI, BUSY, DC, RST, DELAY> {
	fn bounding_box(&self) -> Rectangle {
		self.fb.bounding_box()
	}
}

impl<SPI, BUSY, DC, RST, DELAY> DisplayDriver for Display<SPI, BUSY, DC, RST, DELAY>
where
	SPI: SpiDevice,
	BUSY: InputPin,
	DC: OutputPin,
	RST: OutputPin,
	DELAY: DelayNs,
{
	fn update(&mut self) {
		
	}
}

// KEYPAD

pub(crate) struct Keypad {}

impl KeypadDriver for Keypad {
	fn read_key(&mut self, timeout: core::time::Duration) -> Option<os::hardware::Key> {
		None
	}
}
