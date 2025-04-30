// pub use crate::mahjong_tile;
// use std::collections::HashMap;

// pub enum TileSource {
//     Draw,
//     DeadWallDraw,
//     FromOpponentDiscard,
// }

// pub enum MeldType {
//     Set,      // includes triplets and quadruplets
//     Sequence, // tiles in consecutive rank
//     Pair,
// }

// pub struct MahjongMeld {
//     tiles: Vec<mahjong_tile::MahjongTile>,
//     meld_type: MeldType,
// }

// pub struct AdditionalTileInfo {
//     tile: mahjong_tile::MahjongTile,
//     tile_source: TileSource,
// }

// pub struct MahjongHand {
//     closed_tiles: Vec<mahjong_tile::MahjongTile>, // only tiles in hand (i.e. tiles that can be discarded)
//     open_melds: Vec<MahjongMeld>,
//     // TODO how to both cache tile_counts and initialize it / update it correctly?
//     tile_counts: HashMap<mahjong_tile::MahjongTileSuit, [u8; 9]>, // total count of tiles in hand (ignore red fives), honor tiles are ordered: winds first (E, S, W, N), then dragons (White, Green, Red)
//     // TODO for variants other than riichi, need an additional field for flower tiles
//     additional_tile_info: Option<AdditionalTileInfo>, // the tile in closed_tiles that was added (if any)
// }

// fn tile_count_idx(tile: mahjong_tile::MahjongTile) -> Result<usize, &'static str> {
//     // returns the array index in the tile_counts subarray
//     match tile.suit {
//         mahjong_tile::MahjongTileSuit::Man
//         | mahjong_tile::MahjongTileSuit::Sou
//         | mahjong_tile::MahjongTileSuit::Pin => match tile.rank {
//             mahjong_tile::MahjongTileRank::One => Ok(0),
//             mahjong_tile::MahjongTileRank::Two => Ok(1),
//             mahjong_tile::MahjongTileRank::Three => Ok(2),
//             mahjong_tile::MahjongTileRank::Four => Ok(3),
//             mahjong_tile::MahjongTileRank::Five => Ok(4),
//             mahjong_tile::MahjongTileRank::Six => Ok(5),
//             mahjong_tile::MahjongTileRank::Seven => Ok(6),
//             mahjong_tile::MahjongTileRank::Eight => Ok(7),
//             mahjong_tile::MahjongTileRank::Nine => Ok(8),
//             _ => Err("Should be able to convert number tile's rank to a number"),
//         },
//         mahjong_tile::MahjongTileSuit::Honor => match tile.rank {
//             mahjong_tile::MahjongTileRank::East => Ok(0),
//             mahjong_tile::MahjongTileRank::South => Ok(1),
//             mahjong_tile::MahjongTileRank::West => Ok(2),
//             mahjong_tile::MahjongTileRank::North => Ok(3),
//             mahjong_tile::MahjongTileRank::White => Ok(4),
//             mahjong_tile::MahjongTileRank::Green => Ok(5),
//             mahjong_tile::MahjongTileRank::Red => Ok(6),
//             _ => Err("invalid rank for honor tile suit"),
//         },
//     }
// }

// fn tile_from_suit_and_count_idx(
//     suit: mahjong_tile::MahjongTileSuit,
//     count_idx: usize,
// ) -> Result<mahjong_tile::MahjongTile, &'static str> {
//     // returns the designated tile (rank) given the suit and the array index in the tile_counts subarray
//     let tile_rank = match suit {
//         mahjong_tile::MahjongTileSuit::Man
//         | mahjong_tile::MahjongTileSuit::Sou
//         | mahjong_tile::MahjongTileSuit::Pin => match count_idx {
//             0 => mahjong_tile::MahjongTileRank::One,
//             1 => mahjong_tile::MahjongTileRank::Two,
//             2 => mahjong_tile::MahjongTileRank::Three,
//             3 => mahjong_tile::MahjongTileRank::Four,
//             4 => mahjong_tile::MahjongTileRank::Five,
//             5 => mahjong_tile::MahjongTileRank::Six,
//             6 => mahjong_tile::MahjongTileRank::Seven,
//             7 => mahjong_tile::MahjongTileRank::Eight,
//             8 => mahjong_tile::MahjongTileRank::Nine,
//             _ => return Err("invalid array index for number tile suit"),
//         },
//         mahjong_tile::MahjongTileSuit::Honor => match count_idx {
//             0 => mahjong_tile::MahjongTileRank::East,
//             1 => mahjong_tile::MahjongTileRank::South,
//             2 => mahjong_tile::MahjongTileRank::West,
//             3 => mahjong_tile::MahjongTileRank::North,
//             4 => mahjong_tile::MahjongTileRank::White,
//             5 => mahjong_tile::MahjongTileRank::Green,
//             6 => mahjong_tile::MahjongTileRank::Red,
//             _ => return Err("invalid array index for honor tile suit"),
//         },
//     };
//     Ok(mahjong_tile::MahjongTile {
//         suit: suit,
//         rank: tile_rank,
//         modifier: mahjong_tile::MahjongTileModifier::None,
//     })
// }

// impl MahjongHand {
//     pub fn add_tile(&mut self, new_tile: mahjong_tile::MahjongTile) -> &mut Self {
//         self.closed_tiles.push(new_tile);
//         match tile_count_idx(new_tile) {
//             Ok(idx) => match self.tile_counts.get(&new_tile.suit) {
//                 Some(&suit_counts) => {
//                     let mut new_suit_counts = [0; 9];
//                     new_suit_counts.copy_from_slice(&suit_counts);
//                     new_suit_counts[idx] += 1;
//                     self.tile_counts.insert(new_tile.suit, new_suit_counts);
//                 }
//                 None => {
//                     let mut new_suit_counts = [0; 9];
//                     new_suit_counts[idx] += 1;
//                     self.tile_counts.insert(new_tile.suit, new_suit_counts);
//                 }
//             },
//             Err(_msg) => {
//                 // swallow error?
//             }
//         };
//         self.additional_tile_info = Some(AdditionalTileInfo {
//             tile: new_tile,
//             tile_source: TileSource::Draw,
//         });
//         self
//     }

//     // pub fn discard_tile(&mut self, discard_tile: mahjong_tile::MahjongTile) -> &mut Self {
//     //     // TODO we should update in-place?
//     // }

//     pub fn has_simples(&self) -> bool {
//         // if any tile in the hand is a 2 through 8
//         for num_suit in mahjong_tile::NUMBER_SUITS {
//             match self.tile_counts.get(&num_suit) {
//                 Some(&suit_counts) => {
//                     for idx in 1..8 {
//                         if suit_counts[idx] > 0 {
//                             return true;
//                         }
//                     }
//                 }
//                 None => {}
//             };
//         }
//         return false;
//     }

//     pub fn has_terminals(&self) -> bool {
//         // if any tile in the hand is a 1 or a 9
//         for num_suit in mahjong_tile::NUMBER_SUITS {
//             match self.tile_counts.get(&num_suit) {
//                 Some(&suit_counts) => {
//                     if suit_counts[0] > 0 {
//                         return true;
//                     }
//                     if suit_counts[8] > 0 {
//                         return true;
//                     }
//                 }
//                 None => {}
//             };
//         }
//         return false;
//     }

//     pub fn has_honors(&self) -> bool {
//         // if any tile in the hand is an honor tile (a wind or a dragon)
//         let honor_suit = mahjong_tile::MahjongTileSuit::Honor;
//         match self.tile_counts.get(&honor_suit) {
//             Some(&suit_counts) => {
//                 for idx in 0..7 {
//                     if suit_counts[idx] > 0 {
//                         return true;
//                     }
//                 }
//             }
//             None => {}
//         };
//         return false;
//     }

//     pub fn is_winning_shape(&self) -> bool {
//         todo!()
//     }

//     pub fn is_seven_pairs_shape(&self) -> bool {
//         if !self.open_melds.is_empty() {
//             // open hands cannot meet 7-pairs
//             return false;
//         }
//         if self.closed_tiles.len() != 14 {
//             // need 14 tiles to be seven pairs shape
//             return false;
//         }

//         let mut num_pairs = 0;
//         for tile_suit in mahjong_tile::ALL_SUITS {
//             match self.tile_counts.get(&tile_suit) {
//                 Some(&suit_counts) => {
//                     for idx in 0..9 {
//                         if suit_counts[idx] == 2 {
//                             num_pairs += 1;
//                         }
//                     }
//                 }
//                 None => {}
//             };
//         }
//         num_pairs == 7
//     }

//     pub fn is_thirteen_orphans_shape(&self) -> bool {
//         if !self.open_melds.is_empty() {
//             // open hands cannot meet 13-orphans
//             return false;
//         }
//         if self.closed_tiles.len() != 14 {
//             // need 14 tiles to be seven pairs shape
//             return false;
//         }

//         // check counts: every terminal + honor tile must have count <= 2,
//         // only one terminal can have count == 2,
//         // and every simple must have count == 0
//         let mut terminal_with_two_copies: Option<mahjong_tile::MahjongTile> = None;
//         for number_suit in mahjong_tile::NUMBER_SUITS {
//             match self.tile_counts.get(&number_suit) {
//                 Some(&suit_counts) => {
//                     if suit_counts[0] > 2 {
//                         return false;
//                     } else if suit_counts[0] > 1 {
//                         if terminal_with_two_copies.is_none() {
//                             terminal_with_two_copies = tile_from_suit_and_count_idx(number_suit, 0)
//                                 .map_or_else(|_e| None, |t| Some(t));
//                         } else {
//                             return false;
//                         }
//                     }
//                     for idx in 2..8 {
//                         if suit_counts[idx] > 0 {
//                             return false;
//                         }
//                     }
//                     if suit_counts[8] > 2 {
//                         return false;
//                     } else if suit_counts[8] > 1 {
//                         if terminal_with_two_copies.is_none() {
//                             terminal_with_two_copies = tile_from_suit_and_count_idx(number_suit, 8)
//                                 .map_or_else(|_e| None, |t| Some(t));
//                         } else {
//                             return false;
//                         }
//                     }
//                 }
//                 None => return false,
//             }
//         }

//         let honor_suit = mahjong_tile::MahjongTileSuit::Honor;
//         match self.tile_counts.get(&honor_suit) {
//             Some(&suit_counts) => {
//                 for idx in 0..7 {
//                     if suit_counts[idx] > 2 {
//                         return false;
//                     } else if suit_counts[idx] > 1 {
//                         if terminal_with_two_copies.is_none() {
//                             terminal_with_two_copies =
//                                 tile_from_suit_and_count_idx(honor_suit, idx)
//                                     .map_or_else(|_e| None, |t| Some(t));
//                         } else {
//                             return false;
//                         }
//                     }
//                 }
//             }
//             None => return false,
//         }

//         return true;
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn hand_add_tile() {
//         let mut tile_counts = HashMap::new();
//         tile_counts.insert(
//             mahjong_tile::MahjongTileSuit::Man,
//             [1, 1, 0, 0, 0, 0, 0, 0, 0],
//         );
//         tile_counts.insert(
//             mahjong_tile::MahjongTileSuit::Honor,
//             [0, 0, 1, 0, 0, 0, 0, 0, 0],
//         );
//         let mut initial_hand = MahjongHand {
//             closed_tiles: vec![
//                 mahjong_tile::MahjongTile::from_mspz("1m").unwrap(),
//                 mahjong_tile::MahjongTile::from_mspz("2m").unwrap(),
//                 mahjong_tile::MahjongTile::from_mspz("3z").unwrap(),
//             ],
//             open_melds: vec![],
//             tile_counts,
//             additional_tile_info: None,
//         };

//         let new_tile = mahjong_tile::MahjongTile::from_mspz("2m").unwrap();
//         let new_hand = initial_hand.add_tile(new_tile);
//         match new_hand
//             .tile_counts
//             .get(&mahjong_tile::MahjongTileSuit::Man)
//         {
//             Some(&suit_counts) => {
//                 assert_eq!(suit_counts, [1, 2, 0, 0, 0, 0, 0, 0, 0]);
//             }
//             None => assert!(false),
//         };
//         assert_eq!(new_hand.closed_tiles.len(), 4);
//         // match new_hand.additional_tile_info {
//         //     Some(tile_info) => {
//         //         assert_eq!(tile_info.tile == new_tile);
//         //     }
//         //     None => assert!(false),
//         // };

//         let new_suit_tile = mahjong_tile::MahjongTile::from_mspz("3s").unwrap();
//         let hand_with_new_suit = new_hand.add_tile(new_suit_tile);
//         match hand_with_new_suit
//             .tile_counts
//             .get(&mahjong_tile::MahjongTileSuit::Sou)
//         {
//             Some(&suit_counts) => {
//                 assert_eq!(suit_counts, [0, 0, 1, 0, 0, 0, 0, 0, 0]);
//             }
//             None => assert!(false),
//         };
//         assert_eq!(hand_with_new_suit.closed_tiles.len(), 5);
//         // assert!(
//         //     hand_with_new_suit.additional_tile_info.is_some()
//         //         && hand_with_new_suit.additional_tile_info.unwrap().tile == new_suit_tile
//         // );
//     }

//     #[test]
//     fn hand_has_simples() {
//         let mut tile_counts = HashMap::new();
//         tile_counts.insert(
//             mahjong_tile::MahjongTileSuit::Man,
//             [1, 1, 0, 0, 0, 0, 0, 0, 0],
//         );
//         tile_counts.insert(
//             mahjong_tile::MahjongTileSuit::Honor,
//             [0, 0, 1, 0, 0, 0, 0, 0, 0],
//         );
//         let hand_with_simples = MahjongHand {
//             closed_tiles: vec![
//                 mahjong_tile::MahjongTile::from_mspz("1m").unwrap(),
//                 mahjong_tile::MahjongTile::from_mspz("2m").unwrap(),
//                 mahjong_tile::MahjongTile::from_mspz("3z").unwrap(),
//             ],
//             open_melds: vec![],
//             tile_counts,
//             additional_tile_info: None,
//         };
//         assert!(hand_with_simples.has_simples());
//     }

//     #[test]
//     fn hand_has_terminals() {
//         let mut tile_counts = HashMap::new();
//         tile_counts.insert(
//             mahjong_tile::MahjongTileSuit::Man,
//             [0, 1, 0, 0, 1, 0, 0, 0, 0],
//         );
//         tile_counts.insert(
//             mahjong_tile::MahjongTileSuit::Honor,
//             [0, 0, 1, 0, 0, 0, 0, 0, 0],
//         );
//         let no_terminals_hand = MahjongHand {
//             closed_tiles: vec![
//                 mahjong_tile::MahjongTile::from_mspz("5m").unwrap(),
//                 mahjong_tile::MahjongTile::from_mspz("2m").unwrap(),
//                 mahjong_tile::MahjongTile::from_mspz("3z").unwrap(),
//             ],
//             open_melds: vec![],
//             tile_counts,
//             additional_tile_info: None,
//         };
//         assert!(!no_terminals_hand.has_terminals());
//     }

//     #[test]
//     fn hand_has_honors() {
//         let mut tile_counts = HashMap::new();
//         tile_counts.insert(
//             mahjong_tile::MahjongTileSuit::Man,
//             [0, 0, 0, 0, 1, 0, 0, 0, 1],
//         );
//         tile_counts.insert(
//             mahjong_tile::MahjongTileSuit::Honor,
//             [0, 0, 0, 0, 0, 1, 0, 0, 0],
//         );
//         let hand_with_honors = MahjongHand {
//             closed_tiles: vec![
//                 mahjong_tile::MahjongTile::from_mspz("5m").unwrap(),
//                 mahjong_tile::MahjongTile::from_mspz("9m").unwrap(),
//                 mahjong_tile::MahjongTile::from_mspz("6z").unwrap(),
//             ],
//             open_melds: vec![],
//             tile_counts,
//             additional_tile_info: None,
//         };
//         assert!(hand_with_honors.has_honors());
//     }

//     // #[test]
//     // fn hand_is_winning_shape() {
//     //     todo!()
//     // }

//     // #[test]
//     // fn hand_is_seven_pairs_shape() {
//     //     todo!()
//     // }

//     // #[test]
//     // fn hand_is_thirteen_orphans_shape() {
//     //     todo!()
//     // }
// }
