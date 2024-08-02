use macroquad::prelude::*;

mod simulation;
mod link;
pub use link::Link;
use simulation::{Point, Simulation};

pub mod ui;


#[macroquad::main("Verlet")]
async fn main() {
    let mut simulation = Simulation::new();
    simulation.add_points(&[
        Point::new(Vec2::new(screen_width() / 2.0, screen_height() / 2.0)),
        Point::new(Vec2::new(screen_width() / 2.0, screen_height() / 2.0 - 200.0)),
        Point::new(Vec2::new(screen_width() / 2.0 + 100.0, screen_height() / 2.0 - 200.0)),
        
        // Chain
        Point::new(Vec2::new(screen_width() / 2.0, screen_height() / 2.0)).fixed(),
        Point::new(Vec2::new(screen_width() / 2.0, screen_height() / 2.0 - 30.0)),
        Point::new(Vec2::new(screen_width() / 2.0, screen_height() / 2.0 - 50.0)),

        Point::new(Vec2::new(screen_width() / 2.0 + 500.0, screen_height() / 2.0)).fixed(),
        ]);
        simulation.add_link(Link::new(0, 1).min_length(100.0).max_length(100.0));//
    simulation.add_link(Link::new(1, 2).min_length(100.0).max_length(100.0));
    simulation.add_link(Link::new(0, 2).min_length(100.0).max_length(100.0));
    
    // Chain
    simulation.add_link(Link::new(3, 4).min_length(100.0).max_length(100.0));
    simulation.add_link(Link::new(4, 5).min_length(100.0).max_length(100.0));
    simulation.add_link(Link::new(5, 0).min_length(100.0).max_length(100.0));
    
    simulation.add_link(Link::new(6, 1).min_length(0.0).max_length(300.0).stiffness(0.001).damping(1.0));


    let mut sim_paused = false;

    loop {
        clear_background(BLACK);
        let font_size: u16 = 40;
        let text = "Press Space to pause the simulation";
        let dims = measure_text(text, None, font_size, 1.0);
        draw_text(text, screen_width()/2.0 - dims.width/2.0, dims.height, font_size as f32, GRAY);

        if is_key_pressed(KeyCode::Space) {
            sim_paused = !sim_paused;
        }

        simulation.handle_selection();
        if !sim_paused {
            simulation.perform_update();
        }
        simulation.draw();

        next_frame().await
    }
}