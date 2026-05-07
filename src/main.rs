use std::time::{Duration, Instant};

use macroquad::{miniquad::window::screen_size, prelude::*};

mod simulation;
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

fn setup_cloth(simulation: &mut Simulation) {
    let width = 100;
    let height = 22;
    let spacing = Vec2::from(screen_size()) / Vec2::new(width as f32 + 1.0, height as f32 + 5.0);
    let max_link_len = spacing.y;
    for y in 0..height {
        for x in 0..width {
            let from_idx = y * width + x;
            let mut pt = Point::new(spacing + Vec2::new(x as f32, y as f32) * spacing);
            let stiff = 0.01;
            let damp = 0.9;
            if y == 0 {
                pt = pt.fixed();
            } else if y == height - 1 {
                pt = pt.mass(5.0);
            }
            if y < height - 1 {
                simulation.add_link(
                    Link::new(from_idx, (y + 1) * width + x)
                        .max_length(max_link_len)
                        .stiffness(stiff)
                        .damping(damp),
                );
            }
            if x < width - 1 {
                simulation.add_link(
                    Link::new(from_idx, y * width + x + 1)
                        .max_length(max_link_len)
                        .stiffness(stiff)
                        .damping(damp),
                );
            }
            simulation.add_point(pt);
        }
    }
}

fn setup_IK(simulation: &mut Simulation) {
    let chain_start_pos = Vec2::new(100.0, 500.0);
    let chain_end_pos = Vec2::new(1800.0, 500.0);
    let num_links = 20;
    let diff = (chain_end_pos - chain_start_pos) / num_links as f32;
    let link_length = diff.length();
    simulation.add_point(Point::new(chain_start_pos).fixed());
    for i in 1..=num_links {
        simulation.add_point(Point::new(chain_start_pos + diff * i as f32));
        simulation.add_link(
            Link::new(i - 1, i)
                .max_length(link_length)
                .stiffness(0.01)
                .damping(0.9),
        );
    }
    simulation.add_ik_chain(IKChain::new((0..num_links).collect()));
}

enum TextPlacement {
    Centered,
    CenteredHorizontally,
    TopLeftCorner,
    TopRightCorner,
    AtPosition,
}

fn place_text(text: &str, x: f32, y: f32, placement: TextPlacement, color: Color, font_size: u16) {
    let dims = measure_text(text, None, font_size, 1.0);
    let position = match placement {
        TextPlacement::CenteredHorizontally => (x - dims.width / 2.0, y + dims.height),
        TextPlacement::TopLeftCorner => (x, y + dims.height),
        TextPlacement::TopRightCorner => (x - dims.width, y + dims.height),
        TextPlacement::Centered => (x - dims.width / 2.0, y + dims.height / 2.0),
        TextPlacement::AtPosition => (x, y),
    };
    draw_text(text, position.0, position.1, font_size as f32, color);
}

#[macroquad::main(window_conf)]
async fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    std::env::set_var("RUST_BACKTRACE", "1");

    //rayon::ThreadPoolBuilder::new().num_threads(2).build_global().unwrap();

    let mut simulation = Simulation::new();
    setup_cloth(&mut simulation);

    let mut time_sum = Duration::ZERO;
    let mut num_iterations = 0;

    loop {
        clear_background(BLACK);
        place_text(
            "Press SPACE to pause",
            0.0,
            0.0,
            TextPlacement::TopLeftCorner,
            GRAY,
            40,
        );
        if !simulation.paused {
            num_iterations += 1;
        }
        #[cfg(not(target_arch = "wasm32"))]
        if num_iterations > 0 {
            let text = format!(
                "Avg. update time: {time:.*} ms",
                3,
                time = (time_sum.as_millis() as f64 / num_iterations as f64)
            );
            place_text(
                &text,
                screen_width() / 2.0,
                0.0,
                TextPlacement::CenteredHorizontally,
                GRAY,
                40,
            );
        }

        #[cfg(not(target_arch = "wasm32"))]
        let start = Instant::now();
        simulation.update(1.0 / 180.0);
        #[cfg(not(target_arch = "wasm32"))]
        if !simulation.paused {
            time_sum += start.elapsed();
        }

        place_text(
            "Press 1 for cloth, 2 for IK",
            screen_width() - 40.0,
            0.0,
            TextPlacement::TopRightCorner,
            GRAY,
            40,
        );
        if is_key_released(KeyCode::Key1) || is_key_released(KeyCode::Key2) {
            simulation = Simulation::new();
            time_sum = Duration::ZERO;
            num_iterations = 0;
            if is_key_released(KeyCode::Key1) {
                setup_cloth(&mut simulation);
            } else if is_key_released(KeyCode::Key2) {
                setup_IK(&mut simulation);
            }
        }

        next_frame().await
    }
}
