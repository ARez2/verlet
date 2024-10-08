use macroquad::{prelude::*, ui::{self, hash}};
use miniquad::window::screen_size;

mod link;
pub use link::Link;
mod point;
pub use point::Point;
mod ik;
pub use ik::IKChain;

use super::ui::colorbox;


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
    removed_link_indices: Vec<usize>,
    // A list of all the IK chains, represented as list of SimulationState::links indices
    ik_chains: Vec<IKChain>,

    force: Vec2,
    wall_damping: f32,
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
            removed_link_indices: vec![],
            ik_chains: vec![],
            force: Vec2::new(0.0, 200.0),
            wall_damping: 0.75
        }
    }
}



#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum SelectTarget {
    Point,
    Link
}
type Selection = Option<(SelectTarget, usize)>;


#[derive(Debug)]
pub struct Simulation {
    // Only ever read from previous_state
    previous_state: SimulationState,
    // And write to next_state
    next_state: SimulationState,
    selection: Selection,
    dragging: bool,
    pub paused: bool,
    frame: i32,

    color_picker_texture: Texture2D,
    ui_text_stiffness: String,
}
impl Simulation {
    const UPDATE_STEPS: usize = 4;
    const USE_MULTITHREADING: bool = true;
    const MAX_VELOCITY: f32 = 15.0;
    const MOTION_DAMPENING: f32 = 0.999;
    const MAX_LINK_STRESS: f32 = 3.0;

    pub fn new() -> Self {
        let (color_picker_texture, _) = super::ui::color_picker_texture(100, 100);
        Self {
            previous_state: SimulationState::new(),
            next_state: SimulationState::new(),
            selection: None,
            dragging: false,
            paused: false,
            frame: 0,

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
        self.previous_state.links.push(link.clone());
    }


    pub fn add_ik_chain(&mut self, mut ik_chain: IKChain) {
        ik_chain.current_max_length = ik_chain.links.iter().map(|link| {
            let link = &self.previous_state.links[*link];
            link.max_length
        }).sum::<f32>();
        self.next_state.ik_chains.push(ik_chain.clone());
        self.previous_state.ik_chains.push(ik_chain);
    }


    // User input, that isnt selection
    pub fn handle_interaction(&mut self) {
        let mouse_pos = Vec2::from(mouse_position());
        let prev_mouse_pos = mouse_pos - mouse_delta_position() * Vec2::from(screen_size());
        let middle_mouse_pos = (mouse_pos + prev_mouse_pos) * 0.5;
        let is_dragging = is_mouse_button_down(MouseButton::Right) && mouse_delta_position().length() > 0.0;

        let mut new_links = if self.paused {
            self.next_state.links.clone()
        } else {
            self.previous_state.links.clone()
        };
        let mut already_removed_links = 0;
        new_links = new_links.into_iter().enumerate().filter_map(|(i, link)| {
            if self.next_state.removed_link_indices.binary_search(&i).is_ok() {
                already_removed_links += 1;
                return None;
            }
            let p0 = self.next_state.positions[self.next_state.links[i - already_removed_links].from_idx];
            let p1 = self.next_state.positions[self.next_state.links[i - already_removed_links].to_idx];
            
            let side_of_mouse_pos = side_of_line(mouse_pos, p0, p1);
            let side_of_prev_mouse_pos = side_of_line(prev_mouse_pos, p0, p1);
            let length_of_link = p0.distance(p1);
            if is_dragging && side_of_mouse_pos != side_of_prev_mouse_pos && middle_mouse_pos.distance((p1 + p0) * 0.5) < length_of_link * 0.5 {
                self.next_state.removed_link_indices.push(i);
                None
            } else {
                Some(link)
            }
        }).collect();
        self.next_state.links = new_links;

        if !self.next_state.ik_chains.is_empty() {
            self.next_state.ik_chains[0].target_position = mouse_pos;
        }
    }


    pub fn handle_selection(&mut self) {
        let mouse_pos = mouse_position();
        let mouse_pos = Vec2::new(mouse_pos.0, mouse_pos.1).clamp(Vec2::ZERO, Vec2::from(screen_size()));
        let mouse_over_ui = ui::root_ui().is_mouse_over(mouse_pos);

        if is_mouse_button_pressed(MouseButton::Left) && !mouse_over_ui { // Find a point to select
            self.selection = None;
            let mut selection_distance = f32::MAX;
            for i in 0..self.next_state.positions.len() {
                let pos = self.next_state.positions[i];
                let dist = mouse_pos.distance(pos);
                if dist < POINT_RADIUS {
                    self.selection = Some((SelectTarget::Point, i));
                    selection_distance = dist - POINT_RADIUS;
                }
            }
            for i in 0..self.next_state.links.len() {
                let link = &self.next_state.links[i];
                let dist = distance_from_line(mouse_pos, self.next_state.positions[link.from_idx], self.next_state.positions[link.to_idx]);
                if dist + POINT_RADIUS < POINT_RADIUS*SELECT_GRACE && dist < selection_distance {
                    self.selection = Some((SelectTarget::Link, i));
                    self.ui_text_stiffness = self.next_state.links[i].stiffness.to_string();
                    selection_distance = dist;
                }
            }
        }

        
        if let Some(target) = &self.selection {
            if target.0 == SelectTarget::Point {
                ui::widgets::Window::new(hash!(), vec2(10.0, 10.0), vec2(200.0, 200.0))
                    .label(&format!("Editing Point {}", target.1))
                    .movable(false)
                    .ui(&mut ui::root_ui(), |ui| {
                        colorbox(
                            ui,
                            hash!(),
                            "Start color",
                            &mut self.next_state.colors[target.1],
                            self.color_picker_texture.clone(),
                        );
                        ui.checkbox(hash!(), "Fixed", &mut self.next_state.fixed[target.1])
                });

                if !mouse_over_ui {
                    if is_mouse_button_down(MouseButton::Left) && mouse_delta_position().length() > 0.0 {
                        self.dragging = true;
                    } else if !is_mouse_button_down(MouseButton::Left) {
                        self.dragging = false;
                    }
                    if self.dragging {
                        self.next_state.positions[target.1] = mouse_pos;
                        self.next_state.prev_positions[target.1] = mouse_pos;
                    }
                }
            } else if target.0 == SelectTarget::Link {
                ui::widgets::Window::new(hash!(), vec2(10.0, 10.0), vec2(200.0, 200.0))
                    .label(&format!("Editing Link {}", target.1))
                    .movable(false)
                    .ui(&mut ui::root_ui(), |ui| {
                        ui.slider(hash!(), "Min length", 0f32..1000f32, &mut self.next_state.links[target.1].min_length);
                        ui.slider(hash!(), "Max length", 0f32..1000f32, &mut self.next_state.links[target.1].max_length);
                        ui.input_text(hash!(), "Stiffness", &mut self.ui_text_stiffness);
                        
                        
                        
                        // TODO: Maybe pass in "edit_state" into handle_selection and handle_interaction which is either next_state or previous_state depending on self.paused
                        
                        
                        
                        ui.slider(hash!(), "Damping", 0f32..1f32, &mut self.previous_state.links[target.1].damping);
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
        self.frame += 1;
        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
            // Copy over the modified next_state if unpaused
            if !self.paused {
                let _ = std::mem::replace(&mut self.previous_state, self.next_state.clone());
            }
        }

        self.handle_selection();
        self.handle_interaction();

        if Simulation::USE_MULTITHREADING && !cfg!(target_arch="wasm32") {
            rayon::in_place_scope(|s| {
                if !self.paused {
                    let prev_draw = self.previous_state.clone();
                    s.spawn(|_| {
                        for _ in 0..Simulation::UPDATE_STEPS {
                            Simulation::update_state(&mut self.next_state, &self.previous_state, delta);
                            std::mem::swap(&mut self.next_state, &mut self.previous_state);
                        }
                    });
                    Simulation::draw(&prev_draw, &self.selection);
                } else {
                    Simulation::draw(&self.next_state, &self.selection);
                }
            });
        } else {
            if !self.paused {
                for _ in 0..Simulation::UPDATE_STEPS {
                    Simulation::update_state(&mut self.next_state, &self.previous_state, delta);
                    std::mem::swap(&mut self.next_state, &mut self.previous_state);
                }
            }
            Simulation::draw(&self.next_state, &self.selection);
        }

        
        self.next_state.removed_link_indices.clear();
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
        for i in 0..next_state.positions.len() {
            if previous_state.fixed[i] {
                continue;
            };
    
            let mut velocity = previous_state.positions[i] - previous_state.prev_positions[i];
            if velocity.length() > f32::EPSILON {
                velocity = velocity.clamp_length_max(Simulation::MAX_VELOCITY) * Simulation::MOTION_DAMPENING;
            }
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
        };

        ik::solve_FABRIK(next_state, previous_state);
        Simulation::constrain(next_state, previous_state);
    }


    fn constrain(next_state: &mut SimulationState, previous_state: &SimulationState) {
        let mut link_idx: i32 = -1;
        next_state.links.retain_mut(|link| {
            link_idx += 1;
            let p0 = previous_state.positions[link.from_idx];
            let p0_mass = previous_state.masses[link.from_idx];
            let p1 = previous_state.positions[link.to_idx];
            let p1_mass = previous_state.masses[link.to_idx];
            let pos_delta = p1 - p0;
            let dist = pos_delta.length().max(f32::EPSILON);
    
            if dist > link.min_length && dist < link.max_length {
                return true;
            }
            
            let mut diff = if dist <= link.min_length {
                link.min_length - dist
            } else {
                link.max_length - dist
            };
            diff /= dist;
            let offset = pos_delta * diff * 0.5;
            let offset = (offset).lerp(offset * link.stiffness, link.damping).clamp_length_max(100.0);
            
            if offset.length() > Simulation::MAX_LINK_STRESS {
                next_state.removed_link_indices.push(link_idx as usize);
                return false;
            }
            
            let mass1 = p1_mass / (p0_mass + p1_mass);
            let mass2 = p0_mass / (p0_mass + p1_mass);
    
            // Scale spring force by mass
            if !previous_state.fixed[link.from_idx] {
                next_state.positions[link.from_idx] -= offset * mass1;
            }
            if !previous_state.fixed[link.to_idx] {
                next_state.positions[link.to_idx] += offset * mass2;
            }
            true
        });
    }


    


    /// Draws all points and links, coloring the selection differently
    fn draw(state: &SimulationState, selection: &Selection) {
        for i in 0..state.links.len() {
            let from = state.positions[state.links[i].from_idx];
            let to = state.positions[state.links[i].to_idx];
            if let Some(selection) = selection {
                if selection.0 == SelectTarget::Link && selection.1 == i {
                    draw_line(from.x, from.y, to.x, to.y, 2.0, SELECT_COLOR);
                    continue;
                }
            }
            draw_line(from.x, from.y, to.x, to.y, 2.0, DARKGRAY);
        }

        for i in 0..state.positions.len() {
            let pos = state.positions[i];
            if let Some(selection) = selection {
                if selection.0 == SelectTarget::Point && selection.1 == i {
                    draw_poly_lines(pos.x, pos.y, 10, POINT_RADIUS + 2.0, 0., 4.0, SELECT_COLOR);
                }
            }
            //draw_circle(pos.x, pos.y, POINT_RADIUS, state.colors[i]);
            draw_poly(pos.x, pos.y, 7, POINT_RADIUS, 0., state.colors[i]);
        }
    }
}




// Thanks to https://iquilezles.org/articles/distfunctions2d/
fn distance_from_line(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let pa = point - line_start;
    let ba = line_end - line_start;
    let h = (pa.dot(ba)/ba.dot(ba)).clamp(0.0, 1.0);
    (pa - ba*h).length()
}


// Returns 0 if the points is on the line, 1 if its left of the line, -1 if its right
// Thanks to https://stackoverflow.com/a/1560510
fn side_of_line(point: Vec2, line_start: Vec2, line_end: Vec2) -> i32 {
    (((line_end.x - line_start.x) * (point.y - line_start.y) - (line_end.y - line_start.y) * (point.x - line_start.x)) as i32).signum()
}