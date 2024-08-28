use macroquad::{prelude::*, ui::{self, hash}};
use miniquad::window::screen_size;
use rayon::prelude::*;

use super::{ui::colorbox, Link};


const POINT_RADIUS: f32 = 7.0;
const SELECT_COLOR: Color = BLUE;
// Helps with selection, by extending "collision shape"
const SELECT_GRACE: f32 = 5.0;


#[derive(Debug, Clone)]
struct SimulationState {
    positions: Vec<Vec2>,
    prev_positions: Vec<Vec2>,
    masses: Vec<f32>,
    colors: Vec<Color>,
    fixed: Vec<bool>,
    links: Vec<Link>,

    force: Vec2,
    wall_damping: f32,
    selection: Option<(SelectTarget, usize)>,
    dragging: bool,
}
impl SimulationState {
    pub fn new() -> Self {
        Self {
            positions: vec![],
            prev_positions: vec![],
            masses: vec![],
            colors: vec![],
            fixed: vec![],
            links: vec![],
            force: Vec2::new(0.0, 200.0),
            wall_damping: 0.9,
            selection: None,
            dragging: false,
        }
    }
}


// Only used for defining points, not in the Simulation itself
#[derive(Debug)]
pub struct Point {
    position: Vec2,
    fixed: bool,
    mass: f32,
    color: Color
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


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum SelectTarget {
    Point,
    Link
}

#[derive(Debug)]
pub struct Simulation {
    // Only ever read from previous_state
    previous_state: SimulationState,
    // And write to next_state
    next_state: SimulationState,
    pub paused: bool,

    color_picker_texture: Texture2D,
    ui_text_stiffness: String,
}
impl Simulation {
    const UPDATE_STEPS: usize = 8;
    const USE_MULTITHREADING: bool = true;

    pub fn new() -> Self {
        let (color_picker_texture, _) = super::ui::color_picker_texture(100, 100);
        Self {
            previous_state: SimulationState::new(),
            next_state: SimulationState::new(),
            paused: false,

            color_picker_texture,
            ui_text_stiffness: String::new()
        }
    }


    pub fn add_point(&mut self, point: Point) {
        self.add_points(&[point])
    }


    pub fn add_points(&mut self, points: &[Point]) {
        for point in points {
            self.next_state.positions.push(point.position);
            self.next_state.prev_positions.push(point.position);
            self.next_state.masses.push(point.mass);
            self.next_state.colors.push(point.color);
            self.next_state.fixed.push(point.fixed);
            
            self.previous_state.positions.push(point.position);
            self.previous_state.prev_positions.push(point.position);
            self.previous_state.masses.push(point.mass);
            self.previous_state.colors.push(point.color);
            self.previous_state.fixed.push(point.fixed);
        }
    }


    pub fn add_link(&mut self, link: Link) {
        self.next_state.links.push(link.clone());
        self.previous_state.links.push(link);
    }


    pub fn handle_selection(&mut self) {
        let mouse_pos = mouse_position();
        let mouse_pos = Vec2::new(mouse_pos.0, mouse_pos.1);

        if is_mouse_button_pressed(MouseButton::Left) && !ui::root_ui().is_mouse_over(mouse_pos) { // Find a point to select
            self.next_state.selection = None;
            let mut selection_distance = f32::MAX;
            for i in 0..self.next_state.positions.len() {
                let pos = self.next_state.positions[i];
                let dist = mouse_pos.distance(pos);
                if dist < POINT_RADIUS {
                    self.next_state.selection = Some((SelectTarget::Point, i));
                    selection_distance = dist - POINT_RADIUS;
                }
            }
            for i in 0..self.next_state.links.len() {
                let link = &self.next_state.links[i];
                let dist = distance_from_line(mouse_pos, self.next_state.positions[link.from_idx], self.next_state.positions[link.to_idx]);
                if dist + POINT_RADIUS < POINT_RADIUS*SELECT_GRACE && dist < selection_distance {
                    self.next_state.selection = Some((SelectTarget::Link, i));
                    self.ui_text_stiffness = self.next_state.links[i].stiffness.to_string();
                    selection_distance = dist;
                }
            }
        }

        
        if let Some(target) = &self.next_state.selection {
            if target.0 == SelectTarget::Point {
                ui::widgets::Window::new(hash!(), vec2(10.0, 10.0), vec2(200.0, 200.0))
                    .label(&format!("Editing Point {}", target.1))
                    .movable(false)
                    .ui(&mut *ui::root_ui(), |ui| {
                        colorbox(
                            ui,
                            hash!(),
                            "Start color",
                            &mut self.next_state.colors[target.1],
                            self.color_picker_texture.clone(),
                        );
                });

                if !ui::root_ui().is_mouse_over(mouse_pos) {
                    if is_mouse_button_pressed(MouseButton::Left) {
                        self.next_state.dragging = true;
                    } else if is_mouse_button_released(MouseButton::Left) {
                        self.next_state.dragging = false;
                    }
                    if self.next_state.dragging && (mouse_delta_position() * Vec2::from(screen_size())).length() > 0.0 {
                        self.next_state.positions[target.1] = Vec2::from(mouse_position());
                        self.next_state.prev_positions[target.1] = Vec2::from(mouse_position());
                    }
                }
            } else if target.0 == SelectTarget::Link {
                ui::widgets::Window::new(hash!(), vec2(10.0, 10.0), vec2(200.0, 200.0))
                    .label("Link editor")
                    .movable(false)
                    .ui(&mut *ui::root_ui(), |ui| {
                        ui.slider(hash!(), "Min length", 0f32..1000f32, &mut self.next_state.links[target.1].min_length);
                        ui.slider(hash!(), "Max length", 0f32..1000f32, &mut self.next_state.links[target.1].max_length);
                        ui.input_text(hash!(), "Stiffness", &mut self.ui_text_stiffness);
                        ui.slider(hash!(), "Damping", 0f32..1f32, &mut self.next_state.links[target.1].damping);
                        self.next_state.links[target.1].min_length = self.next_state.links[target.1].min_length.min(self.next_state.links[target.1].max_length);

                        // Clean up input string a bit and parse it back to a float
                        self.ui_text_stiffness = self.ui_text_stiffness.trim_end().to_string();
                        if let Ok(val) = self.ui_text_stiffness.parse::<f32>() {
                            self.next_state.links[target.1].stiffness = val;
                        };
                });
            }
        };
    }


    pub fn update(&mut self, delta: f32) {
        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
            // Copy over the modified next_state
            if !self.paused {
                let _ = std::mem::replace(&mut self.previous_state, self.next_state.clone());
            }
        }

        if Simulation::USE_MULTITHREADING {
            rayon::in_place_scope(|s| {
                if !self.paused {
                    s.spawn(|_| {
                        for _ in 0..Simulation::UPDATE_STEPS {
                            Simulation::update_state(&mut self.next_state, &self.previous_state, delta);
                        }
                    });
                    Simulation::draw(&self.previous_state);
                } else {
                    Simulation::draw(&self.next_state);
                }
            });
        } else {
            for _ in 0..Simulation::UPDATE_STEPS {
                Simulation::update_state(&mut self.next_state, &self.previous_state, delta);
            }
            Simulation::draw(&self.next_state);
        }

        self.handle_selection();
        if !self.paused {
            std::mem::swap(&mut self.next_state, &mut self.previous_state);
        }
    }


    fn update_state(next_state: &mut SimulationState, previous_state: &SimulationState, delta: f32) {
        if delta > 1.0 {
            return;
        }

        // Somehow macroquad doesnt want to be called inside of a rayon iterator, so call it outside
        let screen_size = screen_size();
        for i in 0..previous_state.positions.len() {
            if previous_state.fixed[i] {
                continue
            };

            let velocity = previous_state.positions[i] - previous_state.prev_positions[i];
            let mut new_prev_pos = previous_state.positions[i];
            // Dont scale gravity by mass
            let accel = previous_state.force;
            let mut new_pos = previous_state.positions[i] + velocity + accel * delta * delta;
            
            // Apply boundary constraints
            let velocity = new_pos - new_prev_pos;
            if new_pos.x < 0.0 || new_pos.x > screen_size.0 {
                new_pos.x = new_pos.x.clamp(0.0, screen_size.0);
                new_prev_pos.x = new_pos.x + velocity.x * previous_state.wall_damping;
            }
            if new_pos.y < 0.0 || new_pos.y > screen_size.1 {
                new_pos.y = new_pos.y.clamp(0.0, screen_size.1);
                new_prev_pos.y = new_pos.y + velocity.y * previous_state.wall_damping;
            }

            next_state.positions[i] = new_pos;
            next_state.prev_positions[i] = new_prev_pos;
        }

        Simulation::constrain(next_state, previous_state, delta);
    }


    fn constrain(next_state: &mut SimulationState, previous_state: &SimulationState, delta: f32) {
        for link_idx in 0..previous_state.links.len() {
            let link = &previous_state.links[link_idx];

            let p0 = previous_state.positions[link.from_idx];
            let p0_mass = previous_state.masses[link.from_idx];
            let p1 = previous_state.positions[link.to_idx];
            let p1_mass = previous_state.masses[link.to_idx];
            let pos_delta = p1 - p0;
            let dist = pos_delta.length().max(f32::EPSILON);

            if dist > link.min_length && dist < link.max_length {
                continue;
            }
            
            let diff = if dist <= link.min_length {
                link.min_length - dist
            } else {
                link.max_length - dist
            };
            let percent = (diff / dist) / 2.0;
            let offset = pos_delta * percent;
            let force = (offset).lerp(offset * link.stiffness, link.damping);
            //force = offset;

            // Scale spring force by mass
            if !previous_state.fixed[link.from_idx] {
                next_state.positions[link.from_idx] -= force / p0_mass;
            }
            if !previous_state.fixed[link.to_idx] {
                next_state.positions[link.to_idx] += force / p1_mass;
            }
        }
    }


    /// Draws all points and links, coloring the selection differently
    fn draw(state: &SimulationState) {
        debug!("draw");
        for i in 0..state.links.len() {
            let from = state.positions[state.links[i].from_idx];
            let to = state.positions[state.links[i].to_idx];
            if let Some(selection) = &state.selection {
                if selection.0 == SelectTarget::Link && selection.1 == i {
                    draw_line(from.x, from.y, to.x, to.y, 2.0, SELECT_COLOR);
                    continue;
                }
            }
            draw_line(from.x, from.y, to.x, to.y, 2.0, DARKGRAY);
        }

        for i in 0..state.positions.len() {
            let pos = state.positions[i];
            if let Some(selection) = &state.selection {
                if selection.0 == SelectTarget::Point && selection.1 == i {
                    draw_circle_lines(pos.x, pos.y, POINT_RADIUS + 2.0, 4.0, SELECT_COLOR);
                }
            }
            draw_circle(pos.x, pos.y, POINT_RADIUS, state.colors[i]);
        }
    }
}




// Thanks to https://iquilezles.org/articles/distfunctions2d/
fn distance_from_line(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let pa = point - line_start;
    let ba = line_end - line_start;
    let h = (pa.dot(ba)/ba.dot(ba)).clamp(0.0, 1.0);
    return (pa - ba*h).length();
}