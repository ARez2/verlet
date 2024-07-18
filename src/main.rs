use macroquad::prelude::*;

mod simulation;
use simulation::{Point, Simulation, Stick};


#[macroquad::main("BasicShapes")]
async fn main() {
    let mut simulation = Simulation::new();
    simulation.add_point(
        Point::new(Vec2::new(screen_width() / 2.0, screen_height() / 2.0)).fixed()
    );
    simulation.add_point(Point::new(Vec2::new(screen_width() / 2.0, screen_height() / 2.0 - 200.0)));
    simulation.add_point(Point::new(Vec2::new(screen_width() / 2.0 + 100.0, screen_height() / 2.0 - 200.0)).mass(3.0));
    simulation.add_stick(Stick::new(0, 1).min_length(200.0).max_length(200.0));
    simulation.add_stick(Stick::new(1, 2).min_length(50.0).max_length(200.0));
    
    loop {
        clear_background(BLACK);

        let delta = 1.0/60.0;
        simulation.update(delta);
        simulation.draw();

        next_frame().await
    }
}