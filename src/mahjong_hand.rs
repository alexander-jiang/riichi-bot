pub use crate::mahjong_tile;
use std::collections::VecDeque;

// TODO does it matter if the array size is defined statically or via a constant?
pub struct MahjongHand {
    tiles: Vec<mahjong_tile::MahjongTile>, // only tiles in hand (i.e. tiles that can be discarded)
    tile_count_array: Option<[u8; 34]>,    // none if not computed yet
    shanten: Option<i8>,                   // none if not computed yet
                                           // TODO track fixed/declared melds (open melds or closed kans)
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

/// private struct for winning hand computation (in iterative fashion)
struct PartialState {
    tile_count_array: [u8; 34],
    num_melds_left: u8,
    num_pairs_left: u8,
}

fn tile_id_is_isolated(tile_count_array: &[u8; 34], tile_id: u8) -> bool {
    let tile_idx = usize::from(tile_id);
    if tile_count_array[tile_idx] != 1 {
        // multiple copies of a tile -> not isolated (could form a pair or triplet)
        // and zero copies of a tile -> not isolated by definition
        return false;
    }
    // from this point onward, there's only one copy of this tile
    if tile_id >= mahjong_tile::FIRST_HONOR_ID {
        // a single honor tile is always isolated (honor tiles cannot form sequences)
        return true;
    }
    let tile_rank = tile_id % 9; // from 0-8 (corresponding to ranks 1-9)
    if tile_rank == 0 {
        // single copy of 1 tile is isolated if there is no 2 tile in that suit
        return tile_count_array[tile_idx + 1] == 0;
    } else if tile_rank == 8 {
        // single copy of 9 tile is isolated if there is no 8 tile in that suit
        return tile_count_array[tile_idx - 1] == 0;
    } else {
        // single copy of n-tile isolated if there is no (n-1) tile and no (n+1) tile in that suit
        return tile_count_array[tile_idx + 1] == 0 && tile_count_array[tile_idx - 1] == 0;
    }
}

impl MahjongHand {
    /// Converts our tiles vector to an array of counts per tile type (34 elements, since riichi has 34 different tiles).
    pub fn get_tile_count_array(&self) -> [u8; 34] {
        if self.tile_count_array.is_some() {
            return self.tile_count_array.unwrap();
        }

        let mut new_tile_count_array = [0; 34];
        for tile in self.tiles.iter() {
            new_tile_count_array[usize::from(tile.get_id().unwrap())] += 1;
        }
        new_tile_count_array
    }

    /// Updates the tile_count_array in-place, and returns the updated array.
    pub fn update_tile_count_array(&mut self) -> [u8; 34] {
        let mut new_tile_count_array = [0; 34];
        for tile in self.tiles.iter() {
            new_tile_count_array[usize::from(tile.get_id().unwrap())] += 1;
        }

        // update/overwrite the stored tile_count_array
        self.tile_count_array = Some(new_tile_count_array);

        new_tile_count_array
    }

    /// Adds a tile to this hand
    pub fn add_tile(&mut self, new_tile: mahjong_tile::MahjongTile) {
        // update self.tile_count_array (if it was previously computed)
        if self.tile_count_array.is_some() {
            let mut new_tile_count_array = self.tile_count_array.unwrap();
            let new_tile_id = new_tile.get_id().unwrap();
            new_tile_count_array[usize::from(new_tile_id)] += 1;
            self.tile_count_array = Some(new_tile_count_array);
        }
        self.tiles.push(new_tile);
    }

    /// identify complete hand (with standard shape: 4 melds + 1 pair), ignoring 7 pairs, 13 orphans, and presence of yaku
    pub fn is_winning_shape_iterative(&self) -> bool {
        // maintain how many melds + pair are accounted for, which is updated as we go through the process
        // start with 0 melds, 0 pair (max of 4 melds and 1 pair)

        let tile_count_array = self.get_tile_count_array();

        // TODO deduct tile counts from any open melds (which cannot be altered or broken apart)

        let mut queue: VecDeque<PartialState> = VecDeque::new();
        queue.push_back(PartialState {
            tile_count_array,
            num_melds_left: 4,
            num_pairs_left: 1,
        });
        while !queue.is_empty() {
            let current_item = queue.pop_front().unwrap();
            let current_count_array = current_item.tile_count_array;
            let current_melds_left = current_item.num_melds_left;
            let current_pairs_left = current_item.num_pairs_left;
            println!(
                "tile counts {:?}, melds left: {}, pairs left: {}",
                current_count_array, current_melds_left, current_pairs_left
            );

            let mut tile_id: u8 = 0;
            // get to first tile value in the hand
            while usize::from(tile_id) < current_count_array.len()
                && current_count_array[usize::from(tile_id)] == 0
            {
                tile_id += 1;
            }

            // if no tiles left, did we find a winning hand?
            if tile_id == mahjong_tile::NUM_DISTINCT_TILE_VALUES {
                if current_melds_left == 0 && current_pairs_left == 0 {
                    return true;
                } else {
                    // no tiles left but some melds left? skip
                    continue;
                }
            }

            let can_make_sequence = current_count_array[usize::from(tile_id)] >= 1
                && (if tile_id < mahjong_tile::FIRST_HONOR_ID {
                    if tile_id % 9 <= 6 {
                        // sequence valid if starting at 1-7 (assume sequences starting from lower rank tiles are already found)
                        current_count_array[usize::from(tile_id) + 1] >= 1
                            && current_count_array[usize::from(tile_id) + 2] >= 1
                    } else {
                        // sequence cannot wrap around
                        false
                    }
                } else {
                    // honor tiles cannot form sequences
                    false
                });

            let can_make_pair = current_count_array[usize::from(tile_id)] >= 2;
            let can_make_triplet = current_count_array[usize::from(tile_id)] >= 3;
            // TODO do we need to check quads?

            // if we can't make a sequence, pair, or triplet, then this cannot be a possible winning hand
            // (this tile is isolated)
            if !((can_make_sequence && current_melds_left > 0)
                || (can_make_pair && current_pairs_left > 0)
                || (can_make_triplet && current_melds_left > 0))
            {
                continue;
            }

            if can_make_sequence && current_melds_left > 0 {
                // copy and update count array
                let mut new_tile_count_array: [u8; 34] = current_count_array;
                new_tile_count_array[usize::from(tile_id)] -= 1;
                new_tile_count_array[usize::from(tile_id) + 1] -= 1;
                new_tile_count_array[usize::from(tile_id) + 2] -= 1;
                println!(
                    "can form a sequence starting from id={}, new tile counts: {:?}",
                    tile_id, new_tile_count_array
                );
                queue.push_back(PartialState {
                    tile_count_array: new_tile_count_array,
                    num_melds_left: current_melds_left - 1,
                    num_pairs_left: current_pairs_left,
                });
            }
            if can_make_pair && current_pairs_left > 0 {
                // copy and update count array
                let mut new_tile_count_array: [u8; 34] = current_count_array;
                new_tile_count_array[usize::from(tile_id)] -= 2;
                println!(
                    "can form a pair with id={}, new tile counts: {:?}",
                    tile_id, new_tile_count_array
                );
                queue.push_back(PartialState {
                    tile_count_array: new_tile_count_array,
                    num_melds_left: current_melds_left,
                    num_pairs_left: current_pairs_left - 1,
                });
            }
            if can_make_triplet && current_melds_left > 0 {
                // copy and update count array
                let mut new_tile_count_array: [u8; 34] = current_count_array;
                new_tile_count_array[usize::from(tile_id)] -= 3;
                println!(
                    "can form a triplet with id={}, new tile counts: {:?}",
                    tile_id, new_tile_count_array
                );
                queue.push_back(PartialState {
                    tile_count_array: new_tile_count_array,
                    num_melds_left: current_melds_left - 1,
                    num_pairs_left: current_pairs_left,
                });
            }
        }

        // if no winning hand found, then assume not winning hand by default
        return false;
    }

    fn is_winning_shape_recursive_helper(
        &self,
        tile_count_array: [u8; 34],
        num_melds_left: u8,
        num_pairs_left: u8,
    ) -> bool {
        println!(
            "tile counts {:?}, melds left: {}, pairs left: {}",
            tile_count_array, num_melds_left, num_pairs_left
        );

        let mut tile_id: u8 = 0;
        // get to first tile value in the hand
        while usize::from(tile_id) < tile_count_array.len()
            && tile_count_array[usize::from(tile_id)] == 0
        {
            tile_id += 1;
        }

        // if no tiles left, did we find a winning hand?
        if tile_id == mahjong_tile::NUM_DISTINCT_TILE_VALUES {
            if num_melds_left == 0 && num_pairs_left == 0 {
                return true;
            } else {
                // no tiles left but some melds left? skip
                return false;
            }
        }

        let can_make_sequence = tile_count_array[usize::from(tile_id)] >= 1
            && (if tile_id < mahjong_tile::FIRST_HONOR_ID {
                if tile_id % 9 <= 6 {
                    // sequence valid if starting at 1-7 (assume sequences starting from lower rank tiles are already found)
                    tile_count_array[usize::from(tile_id) + 1] >= 1
                        && tile_count_array[usize::from(tile_id) + 2] >= 1
                } else {
                    // sequence cannot wrap around
                    false
                }
            } else {
                // honor tiles cannot form sequences
                false
            });

        let can_make_pair = tile_count_array[usize::from(tile_id)] >= 2;
        let can_make_triplet = tile_count_array[usize::from(tile_id)] >= 3;
        // TODO do we need to check quads?

        // if we can't make a sequence, pair, or triplet, then this cannot be a possible winning hand
        // (this tile is isolated)
        if !((can_make_sequence && num_melds_left > 0)
            || (can_make_pair && num_pairs_left > 0)
            || (can_make_triplet && num_melds_left > 0))
        {
            return false;
        }

        if can_make_sequence && num_melds_left > 0 {
            // copy and update count array
            let mut new_tile_count_array: [u8; 34] = tile_count_array;
            new_tile_count_array[usize::from(tile_id)] -= 1;
            new_tile_count_array[usize::from(tile_id) + 1] -= 1;
            new_tile_count_array[usize::from(tile_id) + 2] -= 1;
            println!(
                "can form a sequence starting from id={}, new tile counts: {:?}",
                tile_id, new_tile_count_array
            );
            let recursive_result = self.is_winning_shape_recursive_helper(
                new_tile_count_array,
                num_melds_left - 1,
                num_pairs_left,
            );
            if recursive_result {
                return true;
            }
        }
        if can_make_pair && num_pairs_left > 0 {
            // copy and update count array
            let mut new_tile_count_array: [u8; 34] = tile_count_array;
            new_tile_count_array[usize::from(tile_id)] -= 2;
            println!(
                "can form a pair with id={}, new tile counts: {:?}",
                tile_id, new_tile_count_array
            );
            let recursive_result = self.is_winning_shape_recursive_helper(
                new_tile_count_array,
                num_melds_left,
                num_pairs_left - 1,
            );
            if recursive_result {
                return true;
            }
        }
        if can_make_triplet && num_melds_left > 0 {
            // copy and update count array
            let mut new_tile_count_array: [u8; 34] = tile_count_array;
            new_tile_count_array[usize::from(tile_id)] -= 3;
            println!(
                "can form a triplet with id={}, new tile counts: {:?}",
                tile_id, new_tile_count_array
            );
            let recursive_result = self.is_winning_shape_recursive_helper(
                new_tile_count_array,
                num_melds_left - 1,
                num_pairs_left,
            );
            if recursive_result {
                return true;
            }
        }
        return false;
    }

    /// identify complete hand (with standard shape: 4 melds + 1 pair), ignoring 7 pairs, 13 orphans, and presence of yaku
    pub fn is_winning_shape_recursive(&self) -> bool {
        // maintain how many melds + pair are accounted for, which is updated as we go through the process
        // start with 0 melds, 0 pair (max of 4 melds and 1 pair)

        let tile_count_array = self.get_tile_count_array();

        // TODO deduct tile counts from any open melds (which cannot be altered or broken apart)
        let num_melds_left = 4;
        let num_pairs_left = 1;
        self.is_winning_shape_recursive_helper(tile_count_array, num_melds_left, num_pairs_left)
    }

    fn is_winning_shape_recursive_heuristic_helper(
        &self,
        tile_count_array: [u8; 34],
        num_melds_left: u8,
        num_pairs_left: u8,
    ) -> bool {
        // println!(
        //     "tile counts {:?}, melds left: {}, pairs left: {}",
        //     tile_count_array, num_melds_left, num_pairs_left
        // );

        // first check for isolated tiles (any isolated tiles = not a winning shape)
        let mut tile_id: u8 = 0;
        while usize::from(tile_id) < tile_count_array.len() {
            if tile_id_is_isolated(&tile_count_array, tile_id) {
                // println!("found isolated tile id={}", tile_id);
                return false;
            }
            tile_id += 1;
        }

        // get to first tile value in the hand
        tile_id = 0;
        while usize::from(tile_id) < tile_count_array.len()
            && tile_count_array[usize::from(tile_id)] == 0
        {
            tile_id += 1;
        }
        // if no tiles left, did we find a winning hand?
        if tile_id == mahjong_tile::NUM_DISTINCT_TILE_VALUES {
            if num_melds_left == 0 && num_pairs_left == 0 {
                return true;
            } else {
                // no tiles left but some melds left? skip
                return false;
            }
        }

        let tile_idx = usize::from(tile_id);
        let tile_num_rank = mahjong_tile::get_num_tile_rank(tile_id);
        let can_make_sequence = tile_count_array[tile_idx] >= 1
            && (if tile_num_rank.map_or(false, |r| r <= 7) {
                // sequence valid if starting at 1-7 (assume sequences starting from lower rank tiles are already found)
                tile_count_array[usize::from(tile_id) + 1] >= 1
                    && tile_count_array[usize::from(tile_id) + 2] >= 1
            } else {
                // either the tile is an honor tile (cannot form sequence),
                // or the tile is a 8 or 9 in a numbered suit, and sequences cannot wrap around
                false
            });

        let can_make_pair = tile_count_array[tile_idx] >= 2;
        let can_make_triplet = tile_count_array[tile_idx] >= 3;
        let can_make_quad = tile_count_array[tile_idx] >= 4;

        // if we can't make a sequence, pair, or triplet, then this cannot be a possible winning hand
        // (this tile is isolated)
        if !((can_make_sequence && num_melds_left > 0)
            || (can_make_pair && num_pairs_left > 0)
            || (can_make_triplet && num_melds_left > 0)
            || (can_make_quad && num_melds_left > 0))
        {
            return false;
        }

        if can_make_quad && num_melds_left > 0 {
            let mut new_tile_count_array: [u8; 34] = tile_count_array;
            new_tile_count_array[tile_idx] -= 4;
            // println!("can form a quad with id={}", tile_id);
            let recursive_result = self.is_winning_shape_recursive_heuristic_helper(
                new_tile_count_array,
                num_melds_left - 1,
                num_pairs_left,
            );
            if recursive_result {
                return true;
            }
        }

        if can_make_triplet && num_melds_left > 0 {
            let mut new_tile_count_array: [u8; 34] = tile_count_array;
            new_tile_count_array[tile_idx] -= 3;
            // println!("can form a triplet with id={}", tile_id);
            let recursive_result = self.is_winning_shape_recursive_heuristic_helper(
                new_tile_count_array,
                num_melds_left - 1,
                num_pairs_left,
            );
            if recursive_result {
                return true;
            }
        }

        if can_make_pair && num_pairs_left > 0 {
            let mut new_tile_count_array: [u8; 34] = tile_count_array;
            new_tile_count_array[tile_idx] -= 2;
            // println!("can form a pair with id={}", tile_id);
            let recursive_result = self.is_winning_shape_recursive_heuristic_helper(
                new_tile_count_array,
                num_melds_left,
                num_pairs_left - 1,
            );
            if recursive_result {
                return true;
            }
        }

        // if we reached this point, we can't form a quad, pair, or triplet, so the only choice is to
        // use up all copies of this tile to make sequences (because we already ruled out pairs/triplets/quads)
        if can_make_sequence && num_melds_left > 0 {
            let num_copies = tile_count_array[tile_idx];
            // do we have enough tiles to make multiple sequence melds
            if tile_count_array[tile_idx + 1] < num_copies
                || tile_count_array[tile_idx + 2] < num_copies
            {
                // println!(
                //     "cannot form {} sequences starting from id={}",
                //     num_copies, tile_id
                // );
                return false;
            }

            if num_copies > num_melds_left {
                // we know we couldn't form a quad, triplet, or pair, and this means there would be left over
                // copies of the tile that cannot be used
                return false;
            }
            let mut new_tile_count_array: [u8; 34] = tile_count_array;
            new_tile_count_array[tile_idx] -= num_copies;
            new_tile_count_array[tile_idx + 1] -= num_copies;
            new_tile_count_array[tile_idx + 2] -= num_copies;
            // println!(
            //     "can form {} sequence(s) starting from id={}",
            //     num_copies, tile_id
            // );
            let recursive_result = self.is_winning_shape_recursive_heuristic_helper(
                new_tile_count_array,
                num_melds_left - num_copies,
                num_pairs_left,
            );
            if recursive_result {
                return true;
            }
        }

        return false;
    }

    /// identify complete hand (with standard shape: 4 melds + 1 pair), ignoring 7 pairs, 13 orphans, and presence of yaku
    pub fn is_winning_shape_recursive_heuristic(&self) -> bool {
        // maintain how many melds + pair are accounted for, which is updated as we go through the process
        // start with 0 melds, 0 pair (max of 4 melds and 1 pair)

        let mut tile_count_array = self.get_tile_count_array();
        let mut num_melds_left = 4;
        let mut num_pairs_left = 1;

        // first check for isolated tiles (any isolated tiles = not a winning shape)
        let mut tile_id: u8 = 0;
        while usize::from(tile_id) < tile_count_array.len() {
            if tile_id_is_isolated(&tile_count_array, tile_id) {
                // println!("found isolated tile id={}", tile_id);
                return false;
            }
            tile_id += 1;
        }

        // then check honor tiles: we know there are no isolated tiles, so each honor tile must be completely consumed
        tile_id = mahjong_tile::FIRST_HONOR_ID;
        while usize::from(tile_id) < tile_count_array.len() {
            let tile_idx = usize::from(tile_id);
            let honor_tile_count = tile_count_array[tile_idx];
            if honor_tile_count == 0 {
                tile_id += 1;
                continue;
            } else if honor_tile_count == 1 {
                // single honor tile is isolated -> cannot be winning
                return false;
            } else if honor_tile_count == 2 {
                if num_pairs_left == 0 {
                    return false; // cannot form another pair from honor tiles
                }
                num_pairs_left -= 1;
            } else if honor_tile_count == 3 || honor_tile_count == 4 {
                if num_melds_left == 0 {
                    return false; // cannot form another meld from honor tiles
                }
                num_melds_left -= 1;
            } else {
                // more than four tiles??
                return false;
            }
            // for honor tiles, we have to use all the tiles (as honor tiles can only form sets: triplets or quads)
            tile_count_array[tile_idx] = 0;

            tile_id += 1;
        }
        // println!(
        //     "after checking honor tiles: tile counts {:?}, melds left: {}, pairs left: {}",
        //     tile_count_array, num_melds_left, num_pairs_left
        // );

        // TODO deduct tile counts from any open melds (which cannot be altered or broken apart)

        self.is_winning_shape_recursive_heuristic_helper(
            tile_count_array,
            num_melds_left,
            num_pairs_left,
        )
    }

    /// Returns true if the hand is in tenpai (i.e. adding a single copy of certain tile(s) to the hand will form a winning shape)
    pub fn is_tenpai(&self) -> bool {
        todo!()
    }
}

// impl MahjongHand {
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
    use std::time::Instant;

    #[test]
    fn hand_add_tile_and_get_tile_count_array() {
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

    #[test]
    fn hand_get_and_update_tile_count_array() {
        let mut hand = MahjongHand {
            tiles: vec![
                mahjong_tile::MahjongTile::from_text("1m").unwrap(),
                mahjong_tile::MahjongTile::from_text("2m").unwrap(),
                mahjong_tile::MahjongTile::from_text("3z").unwrap(),
            ],
            ..Default::default()
        };
        assert!(hand.tile_count_array.is_none());
        let mut expected_tile_count_array = [0; 34];
        expected_tile_count_array
            [usize::from(mahjong_tile::get_id_from_tile_text("1m").unwrap())] = 1;
        expected_tile_count_array
            [usize::from(mahjong_tile::get_id_from_tile_text("2m").unwrap())] = 1;
        expected_tile_count_array
            [usize::from(mahjong_tile::get_id_from_tile_text("3z").unwrap())] = 1;
        assert_eq!(hand.get_tile_count_array(), expected_tile_count_array);

        hand.update_tile_count_array();
        assert!(
            hand.tile_count_array.is_some()
                && hand.tile_count_array.unwrap() == expected_tile_count_array
        );
    }

    #[test]
    fn hand_is_winning_shape() {
        let hand = MahjongHand {
            // hand: 22234m789s345p33z - waits on 2m,5m,3z
            tiles: vec![
                mahjong_tile::MahjongTile::from_text("3z").unwrap(),
                mahjong_tile::MahjongTile::from_text("3z").unwrap(),
                mahjong_tile::MahjongTile::from_text("2m").unwrap(),
                mahjong_tile::MahjongTile::from_text("2m").unwrap(),
                mahjong_tile::MahjongTile::from_text("2m").unwrap(),
                mahjong_tile::MahjongTile::from_text("3m").unwrap(),
                mahjong_tile::MahjongTile::from_text("4m").unwrap(),
                mahjong_tile::MahjongTile::from_text("7s").unwrap(),
                mahjong_tile::MahjongTile::from_text("8s").unwrap(),
                mahjong_tile::MahjongTile::from_text("9s").unwrap(),
                mahjong_tile::MahjongTile::from_text("3p").unwrap(),
                mahjong_tile::MahjongTile::from_text("4p").unwrap(),
                mahjong_tile::MahjongTile::from_text("5p").unwrap(),
                // add a 2m tile to make it a winning hand
                mahjong_tile::MahjongTile::from_text("2m").unwrap(),
            ],
            ..Default::default()
        };
        assert!(hand.is_winning_shape_iterative());
        assert!(hand.is_winning_shape_recursive());
        assert!(hand.is_winning_shape_recursive_heuristic());
    }

    fn complex_winning_shape_hand() -> MahjongHand {
        MahjongHand {
            // hand: 2223444567m111z - waits on 1m,2m,3m,4m,5m,8m
            tiles: vec![
                mahjong_tile::MahjongTile::from_text("1z").unwrap(),
                mahjong_tile::MahjongTile::from_text("1z").unwrap(),
                mahjong_tile::MahjongTile::from_text("1z").unwrap(),
                mahjong_tile::MahjongTile::from_text("2m").unwrap(),
                mahjong_tile::MahjongTile::from_text("2m").unwrap(),
                mahjong_tile::MahjongTile::from_text("2m").unwrap(),
                mahjong_tile::MahjongTile::from_text("3m").unwrap(),
                mahjong_tile::MahjongTile::from_text("4m").unwrap(),
                mahjong_tile::MahjongTile::from_text("4m").unwrap(),
                mahjong_tile::MahjongTile::from_text("4m").unwrap(),
                mahjong_tile::MahjongTile::from_text("5m").unwrap(),
                mahjong_tile::MahjongTile::from_text("6m").unwrap(),
                mahjong_tile::MahjongTile::from_text("7m").unwrap(),
                // add a 3m tile to make it a winning hand
                mahjong_tile::MahjongTile::from_text("3m").unwrap(),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn hand_is_winning_shape_complex() {
        let hand = complex_winning_shape_hand();
        assert!(hand.is_winning_shape_iterative());
        assert!(hand.is_winning_shape_recursive());
        assert!(hand.is_winning_shape_recursive_heuristic());
    }

    fn not_winning_shape_hand() -> MahjongHand {
        MahjongHand {
            // hand: 122234m789s345p33z
            tiles: vec![
                mahjong_tile::MahjongTile::from_text("3z").unwrap(),
                mahjong_tile::MahjongTile::from_text("3z").unwrap(),
                mahjong_tile::MahjongTile::from_text("1m").unwrap(),
                mahjong_tile::MahjongTile::from_text("2m").unwrap(),
                mahjong_tile::MahjongTile::from_text("2m").unwrap(),
                mahjong_tile::MahjongTile::from_text("2m").unwrap(),
                mahjong_tile::MahjongTile::from_text("3m").unwrap(),
                mahjong_tile::MahjongTile::from_text("4m").unwrap(),
                mahjong_tile::MahjongTile::from_text("7s").unwrap(),
                mahjong_tile::MahjongTile::from_text("8s").unwrap(),
                mahjong_tile::MahjongTile::from_text("9s").unwrap(),
                mahjong_tile::MahjongTile::from_text("3p").unwrap(),
                mahjong_tile::MahjongTile::from_text("4p").unwrap(),
                mahjong_tile::MahjongTile::from_text("5p").unwrap(),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn hand_is_not_winning_shape() {
        let hand = not_winning_shape_hand();
        assert!(!hand.is_winning_shape_iterative());
        assert!(!hand.is_winning_shape_recursive());
        assert!(!hand.is_winning_shape_recursive_heuristic());
    }

    #[test]
    fn time_is_winning_shape_iterative() {
        let hand = complex_winning_shape_hand();
        let before = Instant::now();
        hand.is_winning_shape_iterative();
        println!(
            "Elapsed time for is_winning_shape_iterative: {:.2?}",
            before.elapsed()
        );
    }

    #[test]
    fn time_is_winning_shape_recursive() {
        let hand = complex_winning_shape_hand();
        let before_recursive = Instant::now();
        hand.is_winning_shape_recursive();
        println!(
            "Elapsed time for is_winning_shape_recursive: {:.2?}",
            before_recursive.elapsed()
        );
    }

    #[test]
    fn time_is_winning_shape_recursive_heuristic() {
        let hand = complex_winning_shape_hand();
        let before_recursive_heuristic = Instant::now();
        hand.is_winning_shape_recursive_heuristic();
        println!(
            "Elapsed time for is_winning_shape_recursive_heuristic: {:.2?}",
            before_recursive_heuristic.elapsed()
        );
    }

    #[test]
    fn time_not_winning_shape_iterative() {
        let hand = not_winning_shape_hand();
        let before = Instant::now();
        hand.is_winning_shape_iterative();
        println!(
            "Elapsed time for is_winning_shape_iterative: {:.2?}",
            before.elapsed()
        );
    }

    #[test]
    fn time_not_winning_shape_recursive() {
        let hand = not_winning_shape_hand();
        let before_recursive = Instant::now();
        hand.is_winning_shape_recursive();
        println!(
            "Elapsed time for is_winning_shape_recursive: {:.2?}",
            before_recursive.elapsed()
        );
    }

    #[test]
    fn time_not_winning_shape_recursive_heuristic() {
        let hand = not_winning_shape_hand();
        let before_recursive_heuristic = Instant::now();
        hand.is_winning_shape_recursive_heuristic();
        println!(
            "Elapsed time for is_winning_shape_recursive_heuristic: {:.2?}",
            before_recursive_heuristic.elapsed()
        );
    }
}
