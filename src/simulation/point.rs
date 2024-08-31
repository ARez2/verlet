use macroquad::{prelude::*, color::Color, math::Vec2};

// Only used for letting the user define points, not in the Simulation itself
#[derive(Debug)]
pub struct Point {
    pub(super) position: Vec2,
    pub(super) fixed: bool,
    pub(super) mass: f32,
    pub(super) color: Color
}
impl Point {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            mass: 1.0,
            fixed: false,
            color: WHITE
        }
    }

    pub fn mass(mut self, val: f32) -> Self {
        self.mass = val;
        self
    }
    pub fn fixed(mut self) -> Self {
        self.fixed = true;
        self
    }
    pub fn color(mut self, val: Color) -> Self {
        self.color = val;
        self
    }
}