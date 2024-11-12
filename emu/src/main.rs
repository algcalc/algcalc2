use std::{
	convert::Infallible,
	process,
	sync::{mpsc, RwLock},
	thread,
};

use embedded_graphics::{
	prelude::{Dimensions, DrawTarget, Size},
	primitives::Rectangle,
};
use embedded_graphics_simulator::{
	BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use os::{
	hardware::{DisplayDriver, Hardware, Key, KeypadDriver},
	Color,
};

struct Keypad(mpsc::Receiver<Key>);

impl KeypadDriver for Keypad {
	fn read_key(&mut self, timeout: std::time::Duration) -> Option<os::hardware::Key> {
		self.0.recv_timeout(timeout).ok()
	}
}

struct Display<'a> {
	fb: &'a RwLock<SimulatorDisplay<Color>>,
	bounding_box: Rectangle,
}

impl<'a> DrawTarget for Display<'a> {
	type Color = os::Color;

	type Error = Infallible;

	fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
	where
		I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
	{
		self.fb.write().unwrap().draw_iter(pixels)
	}
}

impl<'a> Dimensions for Display<'a> {
	fn bounding_box(&self) -> embedded_graphics::primitives::Rectangle {
		self.bounding_box
	}
}

impl<'a> DisplayDriver for Display<'a> {
	fn update(&mut self) {}
}

fn main() {
	let (key_tx, key_rx) = mpsc::channel();
	let keypad = Keypad(key_rx);

	let simulator = SimulatorDisplay::<Color>::new(Size::new(296, 128));
	let bounding_box = simulator.bounding_box();

	let simulator = RwLock::new(simulator);
	let display = Display {
		fb: &simulator,
		bounding_box,
	};
	let output_settings = OutputSettingsBuilder::new()
		.theme(BinaryColorTheme::Inverted)
		.max_fps(15)
		.pixel_spacing(0)
		.build();
	let mut win = Window::new("algcalc2 emulator", &output_settings);

	thread::scope(|s| {
		s.spawn(|| os::run(Hardware { display, keypad }));
		loop {
			win.update(&simulator.read().unwrap());
			for event in win.events() {
				match event {
					SimulatorEvent::KeyUp {
						keycode,
						keymod,
						repeat,
					} => todo!(),
					SimulatorEvent::Quit => process::exit(0),
					_ => {}
				}
			}
		}
	});
}
