pub use crate::mahjong_tile;

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

// TODO does it matter if the array size is defined statically or via a constant?
pub struct MahjongHand {
    tiles: Vec<mahjong_tile::MahjongTile>, // only tiles in hand (i.e. tiles that can be discarded)
    tile_count_array: Option<[u8; 34]>,    // none if not computed yet
    shanten: Option<i8>,                   // none if not computed yet
}

impl Default for MahjongHand {
    fn default() -> Self {
        Self {
            tiles: vec![],
            tile_count_array: None,
            shanten: None,
        }
    }
}

impl MahjongHand {
    /// Converts our tiles vector to an array of counts per tile type (34 elements, since riichi has 34 different tiles).
    pub fn get_tile_count_array(&self) -> [u8; 34] {
        let mut new_tile_count_array = [0; 34];
        for tile in self.tiles.iter() {
            new_tile_count_array[usize::from(tile.get_id().unwrap())] += 1;
        }
        new_tile_count_array
    }

    /// Adds a tile to this hand
    pub fn add_tile(&mut self, new_tile: mahjong_tile::MahjongTile) {
        self.tiles.push(new_tile);
    }
}

// impl MahjongHand {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_add_tile_and_get_counts_array() {
        let mut hand = MahjongHand {
            tiles: vec![
                mahjong_tile::MahjongTile::from_text("1m").unwrap(),
                mahjong_tile::MahjongTile::from_text("2m").unwrap(),
                mahjong_tile::MahjongTile::from_text("3z").unwrap(),
            ],
            ..Default::default()
        };

        let new_tile = mahjong_tile::MahjongTile::from_text("2m").unwrap();
        hand.add_tile(new_tile);
        assert_eq!(hand.tiles.len(), 4);
        let tile_counts = hand.get_tile_count_array();
        assert_eq!(
            1,
            tile_counts[usize::from(mahjong_tile::get_id_from_tile_text("1m").unwrap())]
        );
        assert_eq!(
            2,
            tile_counts[usize::from(mahjong_tile::get_id_from_tile_text("2m").unwrap())]
        );
        assert_eq!(
            1,
            tile_counts[usize::from(mahjong_tile::get_id_from_tile_text("3z").unwrap())]
        );

        let new_suit_tile = mahjong_tile::MahjongTile::from_text("3s").unwrap();
        hand.add_tile(new_suit_tile);
        assert_eq!(hand.tiles.len(), 5);
        let new_tile_counts = hand.get_tile_count_array();
        assert_eq!(
            1,
            new_tile_counts[usize::from(mahjong_tile::get_id_from_tile_text("1m").unwrap())]
        );
        assert_eq!(
            2,
            new_tile_counts[usize::from(mahjong_tile::get_id_from_tile_text("2m").unwrap())]
        );
        assert_eq!(
            1,
            new_tile_counts[usize::from(mahjong_tile::get_id_from_tile_text("3z").unwrap())]
        );
        assert_eq!(
            1,
            new_tile_counts[usize::from(mahjong_tile::get_id_from_tile_text("3s").unwrap())]
        );
    }
}
