use std::{
	convert::Infallible,
	process,
	sync::{mpsc, RwLock},
	thread, time::Duration,
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
use sdl2::{keyboard::{Keycode, Mod}, sys::KeyCode};

struct Keypad(mpsc::Receiver<Key>);

impl KeypadDriver for Keypad {
	fn read_key(&mut self, timeout: u64) -> Option<os::hardware::Key> {
		self.0.recv_timeout(Duration::from_millis(timeout)).ok()
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
	tracing_subscriber::fmt::init();
	
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
		.theme(BinaryColorTheme::Default)
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
						keymod: _,
						repeat: _,
					} => {
						key_tx
							.send(match keycode {
								Keycode::NUM_0 | Keycode::KP_0 => Key::D0,
								Keycode::NUM_1 | Keycode::KP_1 => Key::D1,
								Keycode::NUM_2 | Keycode::KP_2 => Key::D2,
								Keycode::NUM_3 | Keycode::KP_3 => Key::D3,
								Keycode::NUM_4 | Keycode::KP_4 => Key::D4,
								Keycode::NUM_5 | Keycode::KP_5 => Key::D5,
								Keycode::NUM_6 | Keycode::KP_6 => Key::D6,
								Keycode::NUM_7 | Keycode::KP_7 => Key::D7,
								Keycode::NUM_8 | Keycode::KP_8 => Key::D8,
								Keycode::NUM_9 | Keycode::KP_9 => Key::D9,
								Keycode::BACKSPACE => Key::Backspace,
								Keycode::LCTRL => Key::Fn,
								Keycode::LEFT => Key::Left,
								Keycode::RIGHT => Key::Right,
								Keycode::KP_PLUS | Keycode::A => Key::Add,
								Keycode::KP_MINUS | Keycode::S => Key::Sub,
								Keycode::KP_MULTIPLY | Keycode::F => Key::Mul,
								Keycode::KP_DIVIDE | Keycode::D => Key::Div,
								Keycode::RETURN | Keycode::KP_ENTER => Key::Eq,
								Keycode::DECIMALSEPARATOR => Key::Dot,
								_ => continue,
							})
							.unwrap();
					}
					SimulatorEvent::Quit => process::exit(0),
					_ => {}
				}
			}
		}
	});
}
