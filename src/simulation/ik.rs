use macroquad::{math::Vec2, prelude::debug};

use super::SimulationState;

#[derive(Debug, Clone)]
pub struct IKChain {
    pub(super) links: Vec<usize>,
    pub target_position: Vec2,
    error_margin: f32,
    num_iterations: usize,
    // Gets recalculated each frame by the sum of length of all links
    pub(super) current_max_length: f32,
}
impl IKChain {
    pub fn new(links: Vec<usize>) -> Self {
        Self {
            links,
            target_position: Vec2::ZERO,
            error_margin: 1.0,
            num_iterations: 8,
            current_max_length: 0.0,
        }
    }

    pub fn error_margin(mut self, margin: f32) -> Self {
        self.error_margin = margin;
        self
    }

    pub fn iterations(mut self, i: usize) -> Self {
        self.num_iterations = i;
        self
    }
}



// Without this extra error margin, the chain would sometimes "snap" to the straightened state
// when the target is "just" out of reach.
const FABRIK_EXTRA_ERROR_MARGIN: f32 = 10.0;
pub fn solve_FABRIK(next_state: &mut SimulationState, previous_state: &SimulationState) {
    for chain_idx in 0..previous_state.ik_chains.len() {
        let chain = &previous_state.ik_chains[chain_idx];
        // "Cut" the chain, when a link has been removed this frame
        // Use this instead of chain.links so we can react if a link has just been destroyed
        let chain_links = &previous_state.ik_chains[chain_idx].links.iter().map_while(|link_idx| {
            if !next_state.removed_link_indices.contains(link_idx) {
                Some(*link_idx)
            } else {
                None
            }
        }).collect::<Vec<usize>>();
        next_state.ik_chains[chain_idx].links = chain_links.clone();
        if chain_links.is_empty() {
            continue;
        }

        let start_pos = previous_state.positions[chain_links[0]];
        let target_pos = chain.target_position;
        
        let diff = target_pos - start_pos;
        let dir_to_target = diff.normalize_or_zero();

        // Recalculate the max length in case it has changed (for example by user input)
        let chain_max_length = chain_links.iter().map(|link| {
            let link = &previous_state.links[*link];
            link.max_length
        }).sum::<f32>();
        // If we cant even reach the target, point each link straight towards the target and be done
        if chain_max_length < diff.length() - chain.error_margin - FABRIK_EXTRA_ERROR_MARGIN {
            let mut prev_pos = start_pos;
            for link_idx in chain_links.iter() {
                let link = &next_state.links[*link_idx];
                let next_pos = prev_pos + dir_to_target * link.max_length;
                next_state.positions[link.to_idx] = next_pos;
                prev_pos = next_pos;
            }
            continue;
        }

        // Helpers to easier iterate over the points
        let mut point_positions = vec![];
        let mut point_indices = vec![];
        let mut link_lengths = vec![];
        for link_idx in chain_links {
            let link = &previous_state.links[*link_idx];
            if !point_indices.contains(&link.from_idx) {
                point_indices.push(link.from_idx);
                point_positions.push(previous_state.positions[link.from_idx]);
            }
            if !point_indices.contains(&link.to_idx) {
                point_indices.push(link.to_idx);
                point_positions.push(previous_state.positions[link.to_idx]);
            }
            link_lengths.push(link.max_length);
        }
        let num_points = point_indices.len();

        for _ in 0..chain.num_iterations {
            // Forward reaching
            point_positions[num_points - 1] = target_pos;
            (0..=num_points-2).into_iter().rev().for_each(|pt_idx| {
                let p0 = point_positions[pt_idx];
                let p1 = point_positions[pt_idx + 1];
                let pt_delta = p1 - p0;
                // Ratio of new length to previous length (before IK step)
                let length = link_lengths[pt_idx/link_lengths.len()] / pt_delta.length().max(f32::EPSILON);
                point_positions[pt_idx] = (1.0 - length) * p1 + length * p0;
            });

            // Backward reaching
            point_positions[0] = start_pos;
            (0..=num_points-2).into_iter().for_each(|pt_idx| {
                let p0 = point_positions[pt_idx];
                let p1 = point_positions[pt_idx + 1];
                let pt_delta = p1 - p0;
                // Ratio of new length to previous length (before IK step)
                let length = link_lengths[pt_idx/link_lengths.len()] / pt_delta.length().max(f32::EPSILON);
                point_positions[pt_idx+1] = (1.0 - length) * p0 + length * p1;
            });

            // If the end of the chain is close enough to the target, stop iterating
            if point_positions[num_points-1].distance(target_pos) < chain.error_margin {
                break;
            }
        }
        // Write the temporary positions back into the next state
        point_indices.iter().for_each(|idx| {
            next_state.positions[*idx] = point_positions[*idx];
        });

        next_state.ik_chains[chain_idx].current_max_length = chain_max_length;
    }
}