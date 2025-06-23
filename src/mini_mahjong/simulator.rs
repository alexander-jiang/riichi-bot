use crate::mini_mahjong::mini_game;
use std::collections::HashMap;
use std::fmt;

use rand::prelude::*;

pub fn display_hand(tiles: &Vec<mini_game::MiniTile>) -> String {
    let mut tile_ranks: Vec<String> = tiles.into_iter().map(|&t| t.rank().to_string()).collect();
    tile_ranks.sort();
    tile_ranks.join("")
}

pub struct MiniGameState {
    pub hand_tiles: Vec<mini_game::MiniTile>,
    pub dead_tiles_by_rank: HashMap<u32, u32>,
}

impl fmt::Debug for MiniGameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MiniGameState [{} {:?}]",
            display_hand(&self.hand_tiles),
            &self.dead_tiles_by_rank
        )
    }
}

fn unshuffled_wall_tiles(game_state: &MiniGameState) -> Vec<mini_game::MiniTile> {
    let mut tile_serial_nums: Vec<u32> = Vec::new();
    for rank in 1..=9 {
        let num_dead_tiles = game_state
            .dead_tiles_by_rank
            .get(&rank)
            .copied()
            .unwrap_or(0);
        assert!(
            num_dead_tiles <= 4,
            "number of dead tiles in game state must be between 0 and 4 inclusive!"
        );
        let num_tiles_left = 4 - num_dead_tiles;
        let num_tiles_left = usize::try_from(num_tiles_left).unwrap();
        let rank_serial_nums = [rank - 1, rank - 1 + 9, rank - 1 + 2 * 9, rank - 1 + 3 * 9];
        for i in 0..num_tiles_left {
            tile_serial_nums.push(rank_serial_nums[i]);
        }
    }

    tile_serial_nums
        .into_iter()
        .map(|n| mini_game::MiniTile { serial: n })
        .collect()
}

pub fn initialize_mini_game_state() -> MiniGameState {
    let mut rng: ThreadRng = rand::rng();

    let empty_game_state = MiniGameState {
        hand_tiles: vec![],
        dead_tiles_by_rank: HashMap::new(),
    };

    let mut tile_wall = unshuffled_wall_tiles(&empty_game_state);
    tile_wall.shuffle(&mut rng);
    let mut hand_tiles = Vec::new();
    let mut dead_tiles_by_rank: HashMap<u32, u32> = HashMap::new();
    while hand_tiles.len() < 5 && !tile_wall.is_empty() {
        let tile_to_draw = tile_wall.pop().expect("should be a tile in wall");
        hand_tiles.push(tile_to_draw);
        let count = dead_tiles_by_rank.entry(tile_to_draw.rank()).or_insert(0);
        *count += 1;
    }

    let hand_tiles = hand_tiles;
    let dead_tiles_by_rank = dead_tiles_by_rank;

    MiniGameState {
        hand_tiles: hand_tiles,
        dead_tiles_by_rank: dead_tiles_by_rank,
    }
}

pub fn set_up_wall(game_state: &MiniGameState) -> Vec<mini_game::MiniTile> {
    let mut rng: ThreadRng = rand::rng();

    let mut tile_wall = unshuffled_wall_tiles(game_state);
    // println!("unshuffled tile wall:");
    // for tile in &tile_wall {
    //     print!("{}", tile.rank());
    // }
    // println!("");

    tile_wall.shuffle(&mut rng);
    // println!("shuffled tile wall:");
    // for tile in &tile_wall {
    //     print!("{}", tile.rank());
    // }
    // println!("");

    tile_wall
}

pub fn play_game(
    game_state: &MiniGameState,
    discard_strategy: fn(&MiniGameState) -> usize,
) -> (i32, bool) {
    let mut tile_wall = set_up_wall(game_state);

    let mut current_game_state = MiniGameState {
        hand_tiles: game_state.hand_tiles.clone(),
        dead_tiles_by_rank: game_state.dead_tiles_by_rank.clone(),
    };

    // draw initial hand of 5
    // if not winning hand, discard

    while current_game_state.hand_tiles.len() < 5 && !tile_wall.is_empty() {
        let tile_to_draw = tile_wall.pop().expect("should be a tile in wall");
        current_game_state.hand_tiles.push(tile_to_draw);
        let count = current_game_state
            .dead_tiles_by_rank
            .entry(tile_to_draw.rank())
            .or_insert(0);
        *count += 1;
    }

    // println!("initial game state: {:?}", current_game_state);

    let mut draws = 0;
    while !mini_game::is_winning_mini_hand(&current_game_state.hand_tiles) && !tile_wall.is_empty()
    {
        // discard a tile
        let index_to_discard = discard_strategy(&current_game_state);
        // let tile_to_discard = current_game_state
        //     .hand_tiles
        //     .get(index_to_discard)
        //     .expect("should be discarding a tile from hand");
        // println!("discarding: {}", tile_to_discard.rank());
        current_game_state.hand_tiles.swap_remove(index_to_discard);

        // draw a new tile
        let drawn_tile = tile_wall.pop().expect("should be a tile in wall");
        current_game_state.hand_tiles.push(drawn_tile);
        draws += 1;
        // println!(
        //     "just drew: {} - new hand: {}",
        //     drawn_tile.rank(),
        //     display_hand(&current_game_state.hand_tiles)
        // );

        // update game state
        let count = current_game_state
            .dead_tiles_by_rank
            .entry(drawn_tile.rank())
            .or_insert(0);
        *count += 1;

        // println!("updated game state: {:?}", current_game_state);
    }

    if mini_game::is_winning_mini_hand(&current_game_state.hand_tiles) {
        // println!("achieved winning hand in {} draws", draws);
        (draws, true)
    } else {
        // println!("no winning hand after drawing all {} tiles", draws);
        (draws, false)
    }
}

// TODO make it so we can evaluate from any position:
// position involves: current hand of 5 tiles, and which tiles are dead / have been seen
// e.g. if your hand is 1,2,2,3,4 and you have seen discards 2,5,5
// you could discard 2p to get a 1234 nobetan (wait on 6 tiles: the remaining 1p & 4p, none of which has been discarded)
// or you could discard 1p to get 2234 aryanmen (wait on 6 tiles: the remaining 2p & 5p, three of which have been discarded)

// this way, you can simulate the outcomes from any given position (i.e. from an arbitrary hand of 5 tiles, after knowing some dead tiles, which can influence the optimal discard)
