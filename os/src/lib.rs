#![no_std]

use crate::hardware::{Hardware, KeypadDriver};
use embedded_graphics::draw_target::DrawTarget;

mod hardware;
mod log;

pub fn run<D, KB>(mut hw: Hardware<D, KB>)
where
    D: DrawTarget,
    KB: KeypadDriver
{

}
