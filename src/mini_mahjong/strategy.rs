use crate::mini_mahjong::simulator;
use crate::mini_mahjong::tenpai;
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

pub fn discard_hold_initial_wait(game_state: &simulator::MiniGameState) -> usize {
    // build count by rank of hand tiles
    let mut hand_tiles_by_rank: HashMap<u32, u32> = HashMap::new();
    for tile in &game_state.hand_tiles {
        let count = hand_tiles_by_rank.entry(tile.rank()).or_insert(0);
        *count += 1;
    }
    let hand_tiles_by_rank = hand_tiles_by_rank;

    // determine which tile is "extra" from the best wait
    for tile_idx in 0..game_state.hand_tiles.len() {
        let tile = game_state
            .hand_tiles
            .get(tile_idx)
            .expect("should be a valid index");

        match tile.rank() {
            5 => {
                // this is a winning tile (shouldn't get to here, but also should not discard)
                continue;
            }
            8 => {
                // this is a winning tile (shouldn't get to here, but also should not discard)
                continue;
            }
            2 => {
                if hand_tiles_by_rank.get(&2).unwrap_or(&0) > &2 {
                    return tile_idx;
                }
            }
            6 => {
                if hand_tiles_by_rank.get(&6).unwrap_or(&0) > &1 {
                    return tile_idx;
                }
            }
            7 => {
                if hand_tiles_by_rank.get(&7).unwrap_or(&0) > &1 {
                    return tile_idx;
                }
            }
            _ => {
                return tile_idx;
            }
        };
    }
    panic!("not expected to reach this part, should have found a tile to discard already!")
}

// pub fn hold_tenpai(game_state: &simulator::MiniGameState) -> usize {
//     // first: identify if a hand of 4 tiles is in tenpai (i.e. is able to win off of any of the tiles)
//     // if discarding a tile achieves tenpai, then do that, and future discards will only discard the tiles that are drawn (i.e. maintain the initial tenpai wait)
//     // if no tile discard achieves tenpai, discard randomly
//     let tenpai_tiles = tenpai::tenpai(game_state);
//     if !tenpai_tiles.is_empty() {
//         // TODO pick one of the tenpai winning tiles

//     } else {
//         discard_random(game_state)
//     }
// }
