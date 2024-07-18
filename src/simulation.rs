use macroquad::prelude::*;



#[derive(Debug)]
pub struct Stick {
    from_idx: usize,
    to_idx: usize,
    min_length: f32,
    max_length: f32,
}
impl Stick {
    pub fn new(from_idx: usize, to_idx: usize) -> Self {
        Self {
            from_idx,
            to_idx,
            min_length: 0.0,
            max_length: f32::MAX
        }
    }
    pub fn min_length(mut self, val: f32) -> Self {
        self.min_length = val;
        self
    }
    pub fn max_length(mut self, val: f32) -> Self {
        self.max_length = val;
        self
    }
}


#[derive(Debug)]
pub struct Simulation {
    positions: Vec<Vec2>,
    prev_positions: Vec<Vec2>,
    masses: Vec<f32>,
    fixed: Vec<bool>,
    sticks: Vec<Stick>,
    force: Vec2,
}
impl Simulation {
    pub fn new() -> Self {
        Self {
            positions: vec![],
            prev_positions: vec![],
            masses: vec![],
            fixed: vec![],
            sticks: vec![],
            force: Vec2::new(1.0, 200.0),
        }
    }

    pub fn add_point(&mut self, position: Vec2, mass: f32, fixed: bool) {
        self.positions.push(position);
        self.prev_positions.push(position);
        self.masses.push(mass);
        self.fixed.push(fixed);
    }

    pub fn add_stick(&mut self, stick: Stick) {
        self.sticks.push(stick);
    }

    pub fn update(&mut self, delta: f32) {
        if delta > 1.0 {
            return;
        }
        for i in 0..self.positions.len() {
            if self.fixed[i] {
                continue
            };

            let velocity = self.positions[i] - self.prev_positions[i];
            let mut new_prev_pos = self.positions[i];
            let accel = self.force / self.masses[i];
            let mut new_pos = self.positions[i] + velocity + accel * delta * delta;
            
            // Apply boundary constraints
            let velocity = new_pos - new_prev_pos;
            if new_pos.x < 0.0 {
                new_pos.x = 0.0;
                new_prev_pos.x = new_pos.x + velocity.x;
            } else if new_pos.x > screen_width() {
                new_pos.x = screen_width();
                new_prev_pos.x = new_pos.x + velocity.x;
            }
            if new_pos.y < 0.0 {
                new_pos.y = 0.0;
                new_prev_pos.y = new_pos.y + velocity.y;
            } else if new_pos.y > screen_height() {
                new_pos.y = screen_height();
                new_prev_pos.y = new_pos.y + velocity.y;
            }

            self.positions[i] = new_pos;
            self.prev_positions[i] = new_prev_pos;
        }

        self.constrain();
    }

    fn constrain(&mut self) {
        for stick_idx in 0..self.sticks.len() {
            let (p0_idx, p1_idx, min_length, max_length) = {
                let stick = &self.sticks[stick_idx];
                (stick.from_idx, stick.to_idx, stick.min_length, stick.max_length)
            };
            let p0 = self.positions[p0_idx];
            let p1 = self.positions[p1_idx];
            let delta = p1 - p0;
            let dist = delta.length();
            if dist > min_length && dist < max_length {
                continue;
            }

            let diff = if dist < min_length {
                min_length - dist
            } else { // diff > max_length
                max_length - dist
            };

            let percent = (diff / dist) / 2.0;
            let offset = delta * percent;

            if !self.fixed[p0_idx] {
                self.positions[p0_idx] -= offset;
            }
            if !self.fixed[p1_idx] {
                self.positions[p1_idx] += offset;
            }
        }
    }

    pub fn draw(&self) {
        for i in 0..self.sticks.len() {
            let from = self.positions[self.sticks[i].from_idx];
            let to = self.positions[self.sticks[i].to_idx];
            draw_line(from.x, from.y, to.x, to.y, 2.0, DARKGRAY);
        }
        for i in 0..self.positions.len() {
            let pos = self.positions[i];
            draw_circle(pos.x, pos.y, 7.0, WHITE);
        }
    }
}
