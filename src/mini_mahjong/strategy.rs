use crate::mini_mahjong::simulator;
use std::collections::HashMap;

use rand::prelude::*;

pub fn discard_random(game_state: &simulator::MiniGameState) -> usize {
    let mut rng: ThreadRng = rand::thread_rng();
    rng.gen_range(0..game_state.hand_tiles.len())
}

pub fn discard_lowest_rank(game_state: &simulator::MiniGameState) -> usize {
    let mut index_to_discard = 0;
    let mut min_tile_rank_so_far = u32::MAX;
    for tile_idx in 0..game_state.hand_tiles.len() {
        let tile = game_state
            .hand_tiles
            .get(tile_idx)
            .expect("should be a valid index");
        if tile.rank() < min_tile_rank_so_far {
            min_tile_rank_so_far = tile.rank();
            index_to_discard = tile_idx;
        }
    }
    index_to_discard
}

pub fn discard_highest_rank(game_state: &simulator::MiniGameState) -> usize {
    let mut index_to_discard = 0;
    let mut max_tile_rank_so_far = u32::MIN;
    for tile_idx in 0..game_state.hand_tiles.len() {
        let tile = game_state
            .hand_tiles
            .get(tile_idx)
            .expect("should be a valid index");
        if tile.rank() > max_tile_rank_so_far {
            max_tile_rank_so_far = tile.rank();
            index_to_discard = tile_idx;
        }
    }
    index_to_discard
}

pub fn discard_isolated(game_state: &simulator::MiniGameState) -> usize {
    let mut rng: ThreadRng = rand::thread_rng();
    let mut isolated_tile_idxs: Vec<usize> = Vec::new();

    // build count by rank of hand tiles
    let mut hand_tiles_by_rank: HashMap<u32, u32> = HashMap::new();
    for tile in &game_state.hand_tiles {
        let count = hand_tiles_by_rank.entry(tile.rank()).or_insert(0);
        *count += 1;
    }
    let hand_tiles_by_rank = hand_tiles_by_rank;

    // determine which tiles are isolated
    for tile_idx in 0..game_state.hand_tiles.len() {
        let tile = game_state
            .hand_tiles
            .get(tile_idx)
            .expect("should be a valid index");

        let is_paired = hand_tiles_by_rank.get(&tile.rank()).unwrap_or(&0) > &1;
        let has_lower_rank =
            tile.rank() > 1 && hand_tiles_by_rank.get(&(tile.rank() - 1)).unwrap_or(&0) > &0;
        let has_higher_rank =
            tile.rank() < 9 && hand_tiles_by_rank.get(&(tile.rank() + 1)).unwrap_or(&0) > &0;

        if !is_paired && !has_lower_rank && !has_higher_rank {
            // println!("found isolated tile: {}", tile.rank());
            isolated_tile_idxs.push(tile_idx);
        }
    }

    if !isolated_tile_idxs.is_empty() {
        // prioritize a discard among the isolated tiles
        let random_isolated_tile_list_idx = rng.gen_range(0..isolated_tile_idxs.len());
        let index_to_discard = isolated_tile_idxs
            .get(random_isolated_tile_list_idx)
            .expect("should be a valid element of isolated_tile_idxs");
        *index_to_discard
    } else {
        // no isolated tiles: pick a tile at random
        rng.gen_range(0..game_state.hand_tiles.len())
    }
}
