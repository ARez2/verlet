use std::time::{Duration, Instant};

use macroquad::prelude::*;

mod simulation;
use miniquad::window::screen_size;
use simulation::{IKChain, Link, Point, Simulation};

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
    #[cfg(not(target_arch = "wasm32"))]
    std::env::set_var("RUST_BACKTRACE", "1");

    //rayon::ThreadPoolBuilder::new().num_threads(2).build_global().unwrap();

    let mut simulation = Simulation::new();
    // let width = 100;
    // let height = 22;
    // let spacing = Vec2::from(screen_size()) / Vec2::new(width as f32 + 1.0, height as f32 + 5.0);
    // let max_link_len = spacing.y;
    // for y in 0..height {
    //     for x in 0..width {
    //         let from_idx = y * width + x;
    //         let mut pt = Point::new(spacing + Vec2::new(x as f32, y as f32) * spacing);
    //         let stiff = 0.01;
    //         let damp = 0.9;
    //         if y == 0 {
    //             pt = pt.fixed();
    //         } else if y == height-1 {
    //             pt = pt.mass(5.0);
    //         }
    //         if y < height-1 {
    //             simulation.add_link(Link::new(from_idx, (y+1) * width + x).max_length(max_link_len).stiffness(stiff).damping(damp));
    //         }
    //         if x < width-1 {
    //             simulation.add_link(Link::new(from_idx, y * width + x+1).max_length(max_link_len).stiffness(stiff).damping(damp));
    //         }
    //         simulation.add_point(pt);
    //     }
    // }

    let chain_start_pos = Vec2::new(100.0, 500.0);
    let chain_end_pos = Vec2::new(1800.0, 500.0);
    let num_links = 20;
    let diff = (chain_end_pos - chain_start_pos) / num_links as f32;
    let link_length = diff.length();
    simulation.add_point(Point::new(chain_start_pos).fixed());
    for i in 1..=num_links {
        simulation.add_point(Point::new(chain_start_pos + diff * i as f32));
        simulation.add_link(Link::new(i - 1, i).max_length(link_length).stiffness(0.01).damping(0.9));
    }
    simulation.add_ik_chain(IKChain::new((0..num_links).collect()));

    let mut time_sum = Duration::ZERO;
    let mut num_iterations = 0;

    loop {
        clear_background(BLACK);
        if !simulation.paused {
            num_iterations += 1;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if num_iterations > 0 {
            let font_size: u16 = 40;
            let text = format!("Avg. update time: {time:.*} ms", 3, time=(time_sum.as_millis() as f64 / num_iterations as f64));
            let dims = measure_text(&text, None, font_size, 1.0);
            draw_text(&text, screen_width()/2.0 - dims.width/2.0, dims.height, font_size as f32, GRAY);
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        let start = Instant::now();
        simulation.update(1.0 / 180.0);
        #[cfg(not(target_arch = "wasm32"))]
        if !simulation.paused {
            time_sum += start.elapsed();
        }

        next_frame().await
    }
}