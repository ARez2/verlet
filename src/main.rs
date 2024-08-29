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
        window_width: 1920,
        window_height: 1080,
        ..Default::default()
    }
}


#[macroquad::main(window_conf)]
async fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    //rayon::ThreadPoolBuilder::new().num_threads(2).build_global().unwrap();

    let mut simulation = Simulation::new();
    let width = 100;
    let height = 22;
    let spacing = Vec2::from(screen_size()) / Vec2::new(width as f32 + 1.0, height as f32 + 5.0);
    let max_link_len = spacing.y;
    for y in 0..height {
        for x in 0..width {
            let from_idx = y * width + x;
            let mut pt = Point::new(spacing + Vec2::new(x as f32, y as f32) * spacing);
            let stiff = 0.1;
            let damp = 0.9;
            if y == 0 {
                pt = pt.fixed();
            } else if y == height-1 {
                pt = pt.mass(5.0);
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

    let mut time_sum = Duration::ZERO;
    let mut num_iterations = 0;

    loop {
        clear_background(BLACK);
        if !simulation.paused {
            num_iterations += 1;
        }
        if num_iterations > 0 {
            let font_size: u16 = 40;
            let text = format!("Avg. update time: {time:.*} ms", 3, time=(time_sum.as_millis() as f64 / num_iterations as f64));
            let dims = measure_text(&text, None, font_size, 1.0);
            draw_text(&text, screen_width()/2.0 - dims.width/2.0, dims.height, font_size as f32, GRAY);
        }
        
        let start = Instant::now();
        simulation.update(1.0 / 180.0);
        if !simulation.paused {
            time_sum += start.elapsed();
        }

        next_frame().await
    }
}