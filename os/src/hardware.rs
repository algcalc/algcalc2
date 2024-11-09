use embedded_graphics::draw_target::DrawTarget;

pub struct Hardware<D, KB> {
    pub display: D,
    pub keypad: KB
}

pub trait KeypadDriver {
    // TODO: add methods
}

pub trait DisplayDriver: DrawTarget {
    fn update(&mut self);
}
