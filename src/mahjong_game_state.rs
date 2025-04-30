// use crate::mahjong_hand;
// use crate::mahjong_tile;
// use std::collections::HashMap;

// pub enum WindDirection {
//     East,
//     South,
//     West,
//     North,
// }

// impl WindDirection {
//     pub fn wind_tile_rank(&self) -> mahjong_tile::MahjongTileRank {
//         match self {
//             Self::East => mahjong_tile::MahjongTileRank::East,
//             Self::South => mahjong_tile::MahjongTileRank::South,
//             Self::West => mahjong_tile::MahjongTileRank::West,
//             Self::North => mahjong_tile::MahjongTileRank::North,
//         }
//     }
// }

// pub struct MahjongPlayerGameState {
//     hand: MahjongHand,
//     discarded_tiles: Vec<MahjongTile>,
//     seat_wind: WindDirection,
//     round_wind: WindDirection, // could we inherit this from the MahjongGameState?
//     dora: MahjongTile,         // could we inherit this from the MahjongGameState?
// }

// pub struct MahjongGameState {
//     player_game_states: Vec<MahjongPlayerGameState>,
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn game_state() {
//         let player_hand = mahjong_hand::MahjongHand {
//             closed_tiles: vec![],
//             open_melds: vec![],
//             tile_counts: HashMap::new(),
//             additional_tile_info: None,
//         };
//         let player_game_state = MahjongPlayerGameState {
//             hand: player_hand,
//             discarded_tiles: vec![],
//             seat_wind: WindDirection::West,
//             round_wind: WindDirection::East,
//             dora: mahjong_tile::MahjongTile::from_mspz("5s"),
//         };
//         let game_state = MahjongGameState {
//             player_game_states: vec![player_game_state],
//         };
//     }
// }
