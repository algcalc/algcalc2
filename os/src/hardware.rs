use core::time::Duration;

use embedded_graphics::draw_target::DrawTarget;

pub struct Hardware<D, KB> {
	pub display: D,
	pub keypad: KB,
}

pub trait KeypadDriver {
	fn read_key(&mut self, timeout: Duration) -> Option<Key>;
}

// 7  8  9  F1 F2
// 4  5  6  *  sq
// 1  2  3  -  /
// .  0  =  +  C
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
	Fun1,
	Fun2,
	Add,
	Sub,
	Mul,
	Div,
	Sqrt,
	Eq,
	Clear,
	Dot,
}

pub trait DisplayDriver: DrawTarget {
	fn update(&mut self);
}
