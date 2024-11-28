use core::convert::Infallible;
use defmt::debug;
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
	prelude::{DisplayRotation, QuickRefresh, WaveshareDisplay},
};
use os::hardware::{DisplayDriver, Key, KeypadDriver, SystemDriver};
use rp_pico::hal::{sio::SioFifo, Timer};

use crate::config::{KEYPAD_COLS, KEYPAD_ROWS};

// DISPLAY

pub(crate) struct Display<SPI, BUSY, DC, RST, DELAY> {
	spi: SPI,
	delay: DELAY,
	epd: Epd2in9<SPI, BUSY, DC, RST, DELAY>,
	fb: Display2in9,
}

impl<SPI, BUSY, DC, RST, DELAY> Display<SPI, BUSY, DC, RST, DELAY>
where
	SPI: SpiDevice,
	BUSY: InputPin,
	DC: OutputPin,
	RST: OutputPin,
	DELAY: DelayNs,
{
	pub(crate) fn new(mut spi: SPI, busy: BUSY, dc: DC, rst: RST, mut delay: DELAY) -> Self {
		let epd = Epd2in9::new(&mut spi, busy, dc, rst, &mut delay, None).unwrap();
		let mut fb = Display2in9::default();
		fb.set_rotation(DisplayRotation::Rotate90);
		Self {
			spi,
			delay,
			epd,
			fb,
		}
	}
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
		self.epd
			.update_and_display_new_frame(&mut self.spi, self.fb.buffer(), &mut self.delay)
			.unwrap();
	}

	fn refresh(&mut self) {
		self.epd
			.update_and_display_frame(&mut self.spi, self.fb.buffer(), &mut self.delay)
			.unwrap();
		self.update();
	}
}

// KEYPAD

pub(crate) struct Keypad {
	fifo: SioFifo,
	timer: Timer,
}

impl KeypadDriver for Keypad {
	fn read_key(&mut self) -> Option<Key> {
		use Key::*;
		const KEYMAP: [[Key; KEYPAD_COLS]; KEYPAD_ROWS] = [
			[D7, D8, D9, Backspace, Dot],
			[D4, D5, D6, Add, Sub],
			[D1, D2, D3, Mul, Div],
			[Left, D0, Right, Eq, Fn],
		];

		let msg = self.fifo.read()?;
		let row = msg as usize >> 16;
		let column = msg as usize & 0xFFFF;
		debug!("key pressed: {:x} (row: {}, col: {})", msg, row, column);
		Some(KEYMAP[row][column])
	}

	fn wait_for_key(&mut self, timeout_ms: u64) -> bool {
		let start = self.timer.get_counter();
		while (self.timer.get_counter() - start).to_millis() < timeout_ms {
			if self.fifo.is_read_ready() {
				return true;
			}
		}
		false
	}
}

impl Keypad {
	pub(crate) fn new(fifo: SioFifo, timer: Timer) -> Self {
		Self { fifo, timer }
	}
}

// SYSTEM

const RAM_BEGIN: u64 = 0x2004_0000;
const RAM_END: u64 = 0x2000_0000;

pub(crate) struct System;

impl SystemDriver for System {
	fn memory_used(&mut self) -> u64 {
		RAM_BEGIN - u64::from(cortex_m::register::msp::read())
	}

	fn memory_total(&mut self) -> u64 {
		RAM_BEGIN - RAM_END
	}
}
