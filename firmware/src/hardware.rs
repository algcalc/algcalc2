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
	prelude::WaveshareDisplay,
};
use os::hardware::{DisplayDriver, Key::{self, *}, KeypadDriver};
use rp_pico::hal::{gpio::{DynPinId, FunctionSioInput, FunctionSioOutput, Pin, PullDown}, timer::Instant, Timer};

// DISPLAY

pub(crate) struct Display<SPI, BUSY, DC, RST, DELAY> {
	pub(crate) spi: SPI,
	pub(crate) delay: DELAY,
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
		self.epd
			.update_and_display_frame(&mut self.spi, self.fb.buffer(), &mut self.delay)
			.unwrap();
	}
}

// KEYPAD
const DEBOUNCE_TIME: u64 = 50;

pub(crate) struct Keypad {
	columns: [Pin<DynPinId, FunctionSioInput, PullDown>; 5],
	rows: [Pin<DynPinId, FunctionSioOutput, PullDown>; 4],
	timer: Timer,
	last_pressed: [[Instant; 5]; 4],
}

impl KeypadDriver for Keypad {
	fn read_key(&mut self, timeout: u64) -> Option<Key> {
		let start = self.timer.get_counter();

		loop {
			for (i_row, row) in self.rows.iter_mut().enumerate() {
				row.set_high().unwrap();
				for (i_col, column) in self.columns.iter_mut().enumerate() {
					let now = self.timer.get_counter();
					
					if column.is_high().unwrap() {
						let since_pressed = now - self.last_pressed[i_row][i_col];
						self.last_pressed[i_row][i_col] = now;
						
						if since_pressed.to_millis() < DEBOUNCE_TIME {
							continue;
						}

						row.set_low().unwrap();
						return Some(KEYMAP[i_row][i_col])
					}
				}
				row.set_low().unwrap();
			}

			if (self.timer.get_counter() - start).to_millis() > timeout {
				return None
			}
		}
	}
}

impl Keypad {
	pub(crate) fn new(columns: [Pin<DynPinId, FunctionSioInput, PullDown>; 5], rows: [Pin<DynPinId, FunctionSioOutput, PullDown>; 4], timer: Timer) -> Self {
		let now = timer.get_counter(); 

		Self {
			columns,
			rows,
			timer,
			last_pressed: [[now; 5]; 4],
		}
	}
}

const KEYMAP: [[Key; 5]; 4] = [
	[D7, D8, D9, Backspace, Dot],
	[D4, D5, D6, Add, Sub],
	[D1, D2, D3, Mul, Div],
	[Left, D0, Right, Eq, Fn],
];
