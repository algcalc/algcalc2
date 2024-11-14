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
	prelude::{DisplayRotation, WaveshareDisplay},
};
use os::hardware::{DisplayDriver, Key, KeypadDriver, SystemDriver};
use rp_pico::hal::{
	gpio::{DynPinId, FunctionSioInput, FunctionSioOutput, Pin, PullDown},
	timer::Instant,
	Timer,
};

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
			.update_and_display_frame(&mut self.spi, self.fb.buffer(), &mut self.delay)
			.unwrap();
	}
}

// KEYPAD

const DEBOUNCE_TIME_MS: u64 = 50;
const ROWS: usize = 4;
const COLS: usize = 5;

pub(crate) struct Keypad {
	columns: [Pin<DynPinId, FunctionSioInput, PullDown>; COLS],
	rows: [Pin<DynPinId, FunctionSioOutput, PullDown>; ROWS],
	timer: Timer,
	last_pressed: [[Instant; COLS]; ROWS],
}

impl KeypadDriver for Keypad {
	fn read_key(&mut self, timeout_ms: u64) -> Option<Key> {
		use Key::*;
		const KEYMAP: [[Key; COLS]; ROWS] = [
			[D7, D8, D9, Backspace, Dot],
			[D4, D5, D6, Add, Sub],
			[D1, D2, D3, Mul, Div],
			[Left, D0, Right, Eq, Fn],
		];

		let start = self.timer.get_counter();

		loop {
			for (i_row, row) in self.rows.iter_mut().enumerate() {
				row.set_high().unwrap();
				for (i_col, column) in self.columns.iter_mut().enumerate() {
					let now = self.timer.get_counter();

					if column.is_high().unwrap() {
						let since_pressed = now - self.last_pressed[i_row][i_col];
						self.last_pressed[i_row][i_col] = now;

						if since_pressed.to_millis() < DEBOUNCE_TIME_MS {
							continue;
						}

						row.set_low().unwrap();
						return Some(KEYMAP[i_row][i_col]);
					}
				}
				row.set_low().unwrap();
			}

			if (self.timer.get_counter() - start).to_millis() > timeout_ms {
				return None;
			}
		}
	}
}

impl Keypad {
	pub(crate) fn new(
		columns: [Pin<DynPinId, FunctionSioInput, PullDown>; COLS],
		rows: [Pin<DynPinId, FunctionSioOutput, PullDown>; ROWS],
		timer: Timer,
	) -> Self {
		let now = timer.get_counter();

		Self {
			columns,
			rows,
			timer,
			last_pressed: [[now; COLS]; ROWS],
		}
	}
}

// SYSTEM

const RAM_BEGIN: u64 = 0x2004_0000;
const RAM_END: u64 = 0x2000_0000;

pub(crate) struct System;

impl SystemDriver for System {
	fn memory_used(&mut self) -> u64 {
		RAM_BEGIN - cortex_m::register::msp::read() as u64
	}

	fn memory_total(&mut self) -> u64 {
		RAM_BEGIN - RAM_END
	}
}
