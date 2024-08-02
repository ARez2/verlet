use std::time::{Duration, Instant};

use macroquad::prelude::*;

mod simulation;
mod link;
pub use link::Link;
use miniquad::window::screen_size;
use simulation::{Point, Simulation};

pub mod ui;



fn window_conf() -> Conf {
    Conf {
        window_title: "Verlet".to_owned(),
        window_width: 2560,
        window_height: 1440,
        ..Default::default()
    }
}


#[macroquad::main(window_conf)]
async fn main() {
    //rayon::ThreadPoolBuilder::new().num_threads(1).build_global().unwrap();

    let mut simulation = Simulation::new();
    let width = 100;
    let height = 25;
    let spacing = Vec2::from(screen_size()) / Vec2::new(width as f32 + 1.0, height as f32 + 5.0);
    let max_link_len = spacing.y;
    for y in 0..height {
        for x in 0..width {
            let from_idx = y * width + x;
            let mut pt = Point::new(spacing + Vec2::new(x as f32, y as f32) * spacing);
            let stiff = 0.3;
            let damp = 0.995;
            if y == 0 {
                pt = pt.fixed();
            }
            if y < height-1 {
                simulation.add_link(Link::new(from_idx, (y+1) * width + x).max_length(max_link_len).stiffness(stiff).damping(damp));
            }
            if x < width-1 {
                simulation.add_link(Link::new(from_idx, y * width + x+1).max_length(max_link_len).stiffness(stiff).damping(damp));
            }
            simulation.add_point(pt);
        }
    }


    let mut sim_paused = false;

    let mut time_sum = Duration::ZERO;
    let mut num_iterations = 0;

    loop {
        clear_background(BLACK);
        if !sim_paused {
            num_iterations += 1;
            
            let font_size: u16 = 40;
            let text = format!("Avg. update time: {} ns", time_sum.as_nanos() / num_iterations);
            let dims = measure_text(&text, None, font_size, 1.0);
            draw_text(&text, screen_width()/2.0 - dims.width/2.0, dims.height, font_size as f32, GRAY);
        }

        if is_key_pressed(KeyCode::Space) {
            sim_paused = !sim_paused;
        }

        simulation.handle_selection();
        if !sim_paused {
            let start = Instant::now();
            simulation.update(1.0 / 60.0);
            time_sum += start.elapsed();
        }
        simulation.draw();

        next_frame().await
    }
}