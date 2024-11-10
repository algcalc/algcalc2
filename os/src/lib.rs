#![no_std]

use crate::hardware::{Hardware, KeypadDriver};
use embedded_graphics::draw_target::DrawTarget;
use epd_waveshare::color::Color;

pub mod hardware;
mod log;

pub fn run<D, KB>(hw: Hardware<D, KB>)
where
    D: DrawTarget<Color = Color>,
    KB: KeypadDriver
{

}
