use embedded_graphics::draw_target::DrawTarget;
use epd_waveshare::color::Color;

pub struct Hardware<D, KB, SYS> {
	pub display: D,
	pub keypad: KB,
	pub system: SYS,
}

pub trait KeypadDriver {
	fn read_key(&mut self) -> Option<Key>;
	fn wait_for_key(&mut self, timeout_ms: u64) -> bool;
}

// 7  8  9  BK .
// 4  5  6  +  -
// 1  2  3  *  /
// <  0  >  =  Fn
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Key {
	D0,
	D1,
	D2,
	D3,
	D4,
	D5,
	D6,
	D7,
	D8,
	D9,
	Left,
	Right,
	Backspace,
	Fn,
	Add,
	Sub,
	Mul,
	Div,
	Eq,
	Dot,
}

pub trait DisplayDriver: DrawTarget<Color = Color> {
	fn update(&mut self);
	fn refresh(&mut self) {}
}

pub trait SystemDriver {
	fn memory_used(&mut self) -> u64;
	fn memory_total(&mut self) -> u64;
	fn battery_level(&mut self) -> u8;
}
