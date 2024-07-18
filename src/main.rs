use macroquad::prelude::*;

mod simulation;
use simulation::{Simulation, Stick};


#[macroquad::main("BasicShapes")]
async fn main() {
    let mut simulation = Simulation::new();
    simulation.add_point(Vec2::new(screen_width() / 2.0, screen_height() / 2.0), 1.0, true);
    simulation.add_point(Vec2::new(screen_width() / 2.0, screen_height() / 2.0 - 200.0), 1.0, false);
    simulation.add_point(Vec2::new(screen_width() / 2.0 + 100.0, screen_height() / 2.0 - 200.0), 3.0, false);
    simulation.add_stick(Stick::new(0, 1).min_length(50.0).max_length(200.0));
    simulation.add_stick(Stick::new(1, 2).min_length(50.0).max_length(200.0));
    
    loop {
        clear_background(BLACK);

        //let delta = macroquad::time::get_frame_time();
        let delta = 1.0/60.0;
        simulation.update(delta);
        simulation.draw();

        next_frame().await
    }
}