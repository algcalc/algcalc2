use core::convert::Infallible;
use embedded_graphics::{
	draw_target::DrawTarget, geometry::Dimensions, pixelcolor::BinaryColor, primitives::Rectangle,
	Pixel,
};
use embedded_hal::{
	delay::DelayNs,
	digital::{InputPin, OutputPin},
	spi::SpiDevice,
};
use epd_waveshare::color::Color;
use epd_waveshare::epd2in9_v2::{Display2in9, Epd2in9};
use os::hardware::DisplayDriver;

pub struct Display<SPI, BUSY, DC, RST, DELAY> {
	pub epd: Epd2in9<SPI, BUSY, DC, RST, DELAY>,
	pub fb: Display2in9,
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
	fn update(&mut self) {}
}
