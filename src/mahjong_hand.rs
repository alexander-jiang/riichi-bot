pub use crate::mahjong_tile;
use std::collections::VecDeque;
use std::fmt;

// TODO does it matter if the array size is defined statically or via a constant?
pub struct MahjongHand {
    tiles: Vec<mahjong_tile::MahjongTile>, // only tiles in hand (i.e. tiles that can be discarded)
    tile_count_array: Option<[u8; 34]>,    // none if not computed yet
    _shanten: Option<i8>,                  // none if not computed yet TODO update this eventually
                                           // TODO track fixed/declared melds (open melds or closed kans)
}

impl Default for MahjongHand {
    fn default() -> Self {
        Self {
            tiles: vec![],
            tile_count_array: None,
            _shanten: None,
        }
    }
}

/// private struct for winning hand computation (in iterative fashion)
struct PartialState {
    tile_count_array: [u8; 34],
    num_melds_left: u8,
    num_pairs_left: u8,
}

#[derive(Clone, Debug)]
pub struct PartialHandGroupingState {
    tile_count_array: [u8; 34],
    groups_tile_counts: Vec<[u8; 34]>,
    num_complete_groups: u8,
    num_incomplete_groups: u8,
    num_pairs: u8,
}

impl fmt::Display for PartialHandGroupingState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tile_groups_string_vec = vec![];
        for tile_group_count_array in self.groups_tile_counts.iter() {
            tile_groups_string_vec.push(tile_count_array_to_string(&tile_group_count_array));
        }
        let tile_groups_string = tile_groups_string_vec.join(", ");
        write!(f, "remaining tiles={}, tile_groups=[{}], num_complete_groups={}, num_incomplete_groups={}, num_pairs={}",
            tile_count_array_to_string(&self.tile_count_array),
            tile_groups_string,
            self.num_complete_groups,
            self.num_incomplete_groups,
            self.num_pairs,
        )
    }
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
    let tile_rank = mahjong_tile::get_num_tile_rank(tile_id).unwrap(); // from 1-9
    if tile_rank == 1 {
        // single copy of 1 tile is isolated if there is no 2 tile in that suit
        return tile_count_array[tile_idx + 1] == 0;
    } else if tile_rank == 9 {
        // single copy of 9 tile is isolated if there is no 8 tile in that suit
        return tile_count_array[tile_idx - 1] == 0;
    } else {
        // single copy of n-tile isolated if there is no (n-1) tile and no (n+1) tile in that suit
        return tile_count_array[tile_idx + 1] == 0 && tile_count_array[tile_idx - 1] == 0;
    }
}

pub fn tile_count_array_to_string(tile_count_array: &[u8; 34]) -> String {
    let mut output = String::new();
    // hardcode array len so that the tile_id var is a u8 type
    for tile_id in 0..34 {
        let tile_count = tile_count_array[usize::from(tile_id)];
        let tile_string = mahjong_tile::get_tile_text_from_id(tile_id).unwrap();
        for _i in 0..tile_count {
            output.push_str(&tile_string);
        }
    }
    output
}

/// sequence = three consecutive tiles (e.g. 123 or 678)
pub fn can_make_sequence(tile_count_array: &[u8; 34], tile_id: u8) -> bool {
    let tile_idx = usize::from(tile_id);
    let tile_num_rank = mahjong_tile::get_num_tile_rank(tile_id);
    tile_count_array[tile_idx] >= 1
        && (match tile_num_rank {
            Some(r) if r == 1 => {
                tile_count_array[usize::from(tile_id) + 1] >= 1
                    && tile_count_array[usize::from(tile_id) + 2] >= 1
            }
            Some(r) if r == 2 => {
                (tile_count_array[usize::from(tile_id) + 1] >= 1
                    && tile_count_array[usize::from(tile_id) + 2] >= 1)
                    || (tile_count_array[usize::from(tile_id) - 1] >= 1
                        && tile_count_array[usize::from(tile_id) + 1] >= 1)
            }
            Some(r) if r >= 3 && r <= 7 => {
                (tile_count_array[usize::from(tile_id) + 1] >= 1
                    && tile_count_array[usize::from(tile_id) + 2] >= 1)
                    || (tile_count_array[usize::from(tile_id) - 1] >= 1
                        && tile_count_array[usize::from(tile_id) + 1] >= 1)
                    || (tile_count_array[usize::from(tile_id) - 2] >= 1
                        && tile_count_array[usize::from(tile_id) - 1] >= 1)
            }
            // sequences cannot wrap around
            Some(r) if r == 8 => {
                (tile_count_array[usize::from(tile_id) - 1] >= 1
                    && tile_count_array[usize::from(tile_id) + 1] >= 1)
                    || (tile_count_array[usize::from(tile_id) - 2] >= 1
                        && tile_count_array[usize::from(tile_id) - 1] >= 1)
            }
            Some(r) if r == 9 => {
                tile_count_array[usize::from(tile_id) - 2] >= 1
                    && tile_count_array[usize::from(tile_id) - 1] >= 1
            }
            Some(_) => false,
            // the tile is an honor tile (cannot form sequence),
            None => false,
        })
}

/// ryanmen = two-sided wait (e.g. 23 or 78)
pub fn can_make_ryanmen(tile_count_array: &[u8; 34], tile_id: u8) -> bool {
    let tile_idx = usize::from(tile_id);
    let tile_num_rank = mahjong_tile::get_num_tile_rank(tile_id);
    tile_count_array[tile_idx] >= 1
        && (match tile_num_rank {
            Some(r) if r == 2 => tile_count_array[usize::from(tile_id) + 1] >= 1,
            Some(r) if r >= 3 && r <= 7 => {
                tile_count_array[usize::from(tile_id) + 1] >= 1
                    || tile_count_array[usize::from(tile_id) - 1] >= 1
            }
            Some(r) if r == 8 => tile_count_array[usize::from(tile_id) - 1] >= 1,
            // cannot form ryanmen with a terminal (1 or 9)
            Some(_) => false,
            // the tile is an honor tile (cannot form sequence)
            None => false,
        })
}

/// penchan = one-sided wait (only 12 or 89)
pub fn can_make_penchan(tile_count_array: &[u8; 34], tile_id: u8) -> bool {
    let tile_idx = usize::from(tile_id);
    let tile_num_rank = mahjong_tile::get_num_tile_rank(tile_id);
    tile_count_array[tile_idx] >= 1
        && (match tile_num_rank {
            // must form a penchan with a terminal (12 or 89)
            Some(r) if r == 1 || r == 8 => tile_count_array[usize::from(tile_id) + 1] >= 1,
            Some(r) if r == 2 || r == 9 => tile_count_array[usize::from(tile_id) - 1] >= 1,
            Some(_) => false,
            // the tile is an honor tile (cannot form sequence)
            None => false,
        })
}

// kanchan = inner wait (e.g. 13 or 79)
pub fn can_make_kanchan(tile_count_array: &[u8; 34], tile_id: u8) -> bool {
    let tile_idx = usize::from(tile_id);
    let tile_num_rank = mahjong_tile::get_num_tile_rank(tile_id);
    tile_count_array[tile_idx] >= 1
        && (match tile_num_rank {
            Some(r) if r == 1 || r == 2 => tile_count_array[usize::from(tile_id) + 2] >= 1,
            Some(r) if r >= 3 && r <= 7 => {
                tile_count_array[usize::from(tile_id) + 2] >= 1
                    || tile_count_array[usize::from(tile_id) - 2] >= 1
            }
            Some(r) if r == 8 || r == 9 => tile_count_array[usize::from(tile_id) - 2] >= 1,
            Some(_) => false,
            // the tile is an honor tile (cannot form sequence)
            None => false,
        })
}

// TODO test
// TODO check total number of tiles
fn is_incomplete_group(tile_count_array: &[u8; 34]) -> bool {
    if get_total_num_tiles(tile_count_array) > 2 {
        return false;
    }
    for tile_id in 0..34u8 {
        let tile_idx = usize::from(tile_id);
        if tile_count_array[tile_idx] == 0 {
            continue;
        }

        // pair does not count as complete group
        let is_complete =
            can_make_sequence(tile_count_array, tile_id) || tile_count_array[tile_idx] >= 3;
        return !is_complete;
    }
    return false;
}

fn get_total_num_tiles(tile_count_array: &[u8; 34]) -> u16 {
    let mut total_num_tiles: u16 = 0;
    for tile_id in 0..34u8 {
        let tile_idx = usize::from(tile_id);
        total_num_tiles = total_num_tiles.saturating_add(tile_count_array[tile_idx].into());
    }
    total_num_tiles
}

fn is_pair(tile_count_array: &[u8; 34]) -> bool {
    if get_total_num_tiles(tile_count_array) != 2 {
        return false;
    }
    for tile_id in 0..34u8 {
        let tile_idx = usize::from(tile_id);
        if tile_count_array[tile_idx] == 0 {
            continue;
        }

        return tile_count_array[tile_idx] == 2;
    }
    return false;
}

// TODO test
// TODO should we refactor this? the tile_count_array is no longer useful - would be better to know the meld type and tiles
fn tile_ids_to_complete_group(tile_count_array: &[u8; 34]) -> Option<Vec<u8>> {
    for tile_id in 0..34u8 {
        let tile_idx = usize::from(tile_id);
        if tile_count_array[tile_idx] == 0 {
            continue;
        }

        let is_complete =
            can_make_sequence(tile_count_array, tile_id) || tile_count_array[tile_idx] >= 3;
        if is_complete {
            return None;
        }
        if can_make_ryanmen(tile_count_array, tile_id) {
            if tile_idx + 1 < tile_count_array.len() && tile_count_array[tile_idx + 1] > 0 {
                return Some(vec![tile_id - 1, tile_id + 2]);
            } else {
                return Some(vec![tile_id - 2, tile_id + 1]);
            }
        } else if can_make_penchan(tile_count_array, tile_id) {
            let tile_num_rank = mahjong_tile::get_num_tile_rank(tile_id);
            // TODO refactor to use match
            if tile_num_rank.is_none() {
                panic!("invalid penchan");
            }
            let tile_num_rank = tile_num_rank.unwrap();
            if tile_num_rank == 8 {
                return Some(vec![tile_id - 1]);
            } else if tile_num_rank == 9 {
                return Some(vec![tile_id - 2]);
            } else if tile_num_rank == 1 {
                return Some(vec![tile_id + 2]);
            } else if tile_num_rank == 2 {
                return Some(vec![tile_id + 1]);
            } else {
                panic!("invalid penchan");
            }
        } else if can_make_kanchan(tile_count_array, tile_id) {
            if tile_idx + 2 < tile_count_array.len() && tile_count_array[tile_idx + 2] > 0 {
                return Some(vec![tile_id + 1]);
            } else {
                return Some(vec![tile_id - 1]);
            }
        } else {
            return Some(vec![tile_id]);
        }
    }
    return None;
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

    pub fn is_winning_shape_build_shapes(&self) -> bool {
        !self.build_shapes(0).is_empty()
    }

    pub fn is_winning_shape(&self) -> bool {
        // uses the best implementation so far
        self.is_winning_shape_recursive_heuristic()
    }

    /// Returns true if the hand is in tenpai (i.e. adding a single copy of certain tile(s) to the hand will form a winning shape)
    pub fn is_tenpai_brute_force(&self) -> bool {
        for tile_id in 0..34 {
            let mut new_hand = MahjongHand {
                tiles: self.tiles.clone(),
                ..Default::default()
            };
            let new_tile = mahjong_tile::MahjongTile::from_id(tile_id).unwrap();
            new_hand.add_tile(new_tile);
            if new_hand.is_winning_shape() {
                // println!(
                //     "hand is tenpai, wins on {}",
                //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
                // );
                return true;
            }
        }
        return false;
    }

    /// Returns true if the hand is in tenpai (i.e. adding a single copy of certain tile(s) to the hand will form a winning shape)
    pub fn is_tenpai_build_shapes(&self) -> bool {
        !self.build_shapes(1).is_empty()
    }

    /// Returns true if the hand is in tenpai (i.e. adding a single copy of certain tile(s) to the hand will form a winning shape)
    pub fn is_tenpai(&self) -> bool {
        // uses the best implementation so far
        self.is_tenpai_brute_force()
    }

    pub fn get_tenpai_tiles_brute_force(&self) -> Vec<u8> {
        // if returns empty vec, then the hand is not in tenpai
        let mut tenpai_tiles = vec![];
        for tile_id in 0..34 {
            let mut new_hand = MahjongHand {
                tiles: self.tiles.clone(),
                ..Default::default()
            };
            let new_tile = mahjong_tile::MahjongTile::from_id(tile_id).unwrap();
            new_hand.add_tile(new_tile);
            if new_hand.is_winning_shape() {
                // println!(
                //     "hand is tenpai, wins on {}",
                //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
                // );
                tenpai_tiles.push(tile_id);
            }
        }
        tenpai_tiles
    }

    pub fn get_tenpai_tiles_build_shapes(&self) -> Vec<u8> {
        // if returns empty vec, then the hand is not in tenpai
        let mut tenpai_tiles = vec![];
        let tenpai_groupings = self.build_shapes(1);
        for grouping in tenpai_groupings.iter() {
            for group in &grouping.groups_tile_counts {
                // if there's 3 complete groups and this group is the only pair, we can't "complete" the pair into a triplet to make a winning hand
                // but if we have 3 complete groups and 2 pairs, then we can complete either pair (shanpon wait)
                if is_incomplete_group(group)
                    && !(grouping.num_complete_groups == 3
                        && grouping.num_pairs == 1
                        && is_pair(group))
                {
                    let tile_ids_to_complete = tile_ids_to_complete_group(group).unwrap();
                    for tile_id in tile_ids_to_complete {
                        if !tenpai_tiles.contains(&tile_id) {
                            tenpai_tiles.push(tile_id);
                        }
                    }
                }
            }
        }
        tenpai_tiles
    }

    pub fn build_shapes(&self, incomplete_group_budget: u8) -> Vec<PartialHandGroupingState> {
        let tile_count_array = self.get_tile_count_array();
        let mut partial_state = PartialHandGroupingState {
            tile_count_array,
            groups_tile_counts: vec![],
            num_complete_groups: 0,
            num_incomplete_groups: 0,
            num_pairs: 0,
        };

        // TODO deduct tile counts from any open melds (which cannot be altered or broken apart)
        // and update the partial hand grouping state

        // look through honor tiles
        let mut tile_id = mahjong_tile::FIRST_HONOR_ID;
        while usize::from(tile_id) < partial_state.tile_count_array.len() {
            let tile_idx: usize = usize::from(tile_id);
            let honor_tile_count = partial_state.tile_count_array[tile_idx];
            // println!(
            //     "considering honor tile {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );
            if honor_tile_count == 0 {
                tile_id += 1;
                continue;
            }
            if honor_tile_count == 1 {
                partial_state.num_incomplete_groups += 1;
            } else if honor_tile_count == 2 {
                partial_state.num_pairs += 1;
            } else if honor_tile_count == 3 || honor_tile_count == 4 {
                partial_state.num_complete_groups += 1;
            } else {
                panic!("invalid number of honor tiles");
            }
            // for honor tiles, we have to use all the tiles (as honor tiles can only form sets: triplets or quads)
            partial_state.tile_count_array[tile_idx] = 0;
            let mut group_tile_count = [0; 34];
            group_tile_count[tile_idx] = honor_tile_count;
            partial_state.groups_tile_counts.push(group_tile_count);

            tile_id += 1;
        }

        let mut queue: VecDeque<PartialHandGroupingState> = VecDeque::new();
        if partial_state.num_incomplete_groups <= incomplete_group_budget {
            queue.push_back(partial_state);
        } else {
            // println!("too many incomplete groups, already over budget");
        }

        let mut completed_partial_states = Vec::new();

        while !queue.is_empty() {
            let current_state = queue.pop_front().unwrap();
            // println!("current state: {}", current_state,);
            let current_count_array = current_state.tile_count_array;

            let mut tile_id = 0;
            // assumes the honor tiles were handled already
            while tile_id < mahjong_tile::FIRST_HONOR_ID
                && usize::from(tile_id) < current_count_array.len()
                && current_count_array[usize::from(tile_id)] == 0
            {
                tile_id += 1;
            }

            if tile_id >= mahjong_tile::FIRST_HONOR_ID {
                // println!("adding current state to completed list: {}", current_state,);
                completed_partial_states.push(current_state);
                continue;
            }

            let tile_idx = usize::from(tile_id);
            let num_tile_count = current_count_array[tile_idx];

            // println!(
            //     "considering tile {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );

            if num_tile_count >= 3 {
                // recursively try to add a triplet or a pair
                let mut new_state_after_triplet = current_state.clone();
                let mut group_tile_count = [0; 34];
                group_tile_count[tile_idx] = 3;
                new_state_after_triplet.tile_count_array[tile_idx] = num_tile_count - 3;
                new_state_after_triplet
                    .groups_tile_counts
                    .push(group_tile_count);
                new_state_after_triplet.num_complete_groups += 1;
                queue.push_back(new_state_after_triplet);
                // println!(
                //     "will recursively try forming a triplet {}",
                //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
                // );

                let mut new_state_after_pair = current_state.clone();
                let mut group_tile_count = [0; 34];
                group_tile_count[tile_idx] = 2;
                new_state_after_pair.tile_count_array[tile_idx] = num_tile_count - 2;
                new_state_after_pair
                    .groups_tile_counts
                    .push(group_tile_count);
                new_state_after_pair.num_pairs += 1;
                if new_state_after_pair.num_pairs > 1 {
                    new_state_after_pair.num_incomplete_groups += 1;
                }
                if new_state_after_pair.num_incomplete_groups <= incomplete_group_budget {
                    queue.push_back(new_state_after_pair);
                    // println!(
                    //     "will recursively try forming a pair {}",
                    //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
                    // );
                } else {
                    // println!("exceeded incomplete group budget, cannot form an additional pair")
                }

                continue;
            }
            if num_tile_count == 2 {
                // recursively try to add as a pair (and let it continue to try adding as a single)
                let mut new_state_after_pair = current_state.clone();
                let mut group_tile_count = [0; 34];
                group_tile_count[tile_idx] = 2;
                new_state_after_pair.tile_count_array[tile_idx] = num_tile_count - 2;
                new_state_after_pair
                    .groups_tile_counts
                    .push(group_tile_count);
                new_state_after_pair.num_pairs += 1;
                if new_state_after_pair.num_pairs > 1 {
                    new_state_after_pair.num_incomplete_groups += 1;
                }
                if new_state_after_pair.num_incomplete_groups <= incomplete_group_budget {
                    queue.push_back(new_state_after_pair);
                    // println!(
                    //     "will recursively try forming a pair {}",
                    //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
                    // );
                } else {
                    // println!("exceeded incomplete group budget, cannot form an additional pair")
                }
            }

            // try to add as a sequence, or as a partial wait, or as a isolated tile
            let can_make_sequence = can_make_sequence(&current_count_array, tile_id);
            let can_make_ryanmen = can_make_ryanmen(&current_count_array, tile_id);
            let can_make_penchan = can_make_penchan(&current_count_array, tile_id);
            let can_make_kanchan = can_make_kanchan(&current_count_array, tile_id);

            if can_make_sequence {
                let mut new_state_after_sequence = current_state.clone();
                let mut group_tile_count = [0; 34];
                group_tile_count[tile_idx] = 1;
                group_tile_count[tile_idx + 1] = 1;
                group_tile_count[tile_idx + 2] = 1;
                new_state_after_sequence.tile_count_array[tile_idx] =
                    current_count_array[tile_idx] - 1;
                new_state_after_sequence.tile_count_array[tile_idx + 1] =
                    current_count_array[tile_idx + 1] - 1;
                new_state_after_sequence.tile_count_array[tile_idx + 2] =
                    current_count_array[tile_idx + 2] - 1;
                new_state_after_sequence
                    .groups_tile_counts
                    .push(group_tile_count);
                new_state_after_sequence.num_complete_groups += 1;
                queue.push_back(new_state_after_sequence);
                // println!(
                //     "will recursively try forming a sequence from {}",
                //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
                // );
            }

            if can_make_ryanmen && current_state.num_incomplete_groups < incomplete_group_budget {
                let mut new_state_after_ryanmen = current_state.clone();
                let mut group_tile_count = [0; 34];
                group_tile_count[tile_idx] = 1;
                group_tile_count[tile_idx + 1] = 1;
                new_state_after_ryanmen.tile_count_array[tile_idx] =
                    current_count_array[tile_idx] - 1;
                new_state_after_ryanmen.tile_count_array[tile_idx + 1] =
                    current_count_array[tile_idx + 1] - 1;
                new_state_after_ryanmen
                    .groups_tile_counts
                    .push(group_tile_count);
                new_state_after_ryanmen.num_incomplete_groups += 1;
                queue.push_back(new_state_after_ryanmen);
                // println!(
                //     "will recursively try forming a ryanmen from {}",
                //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
                // );
            }

            if can_make_penchan && current_state.num_incomplete_groups < incomplete_group_budget {
                let mut new_state_after_penchan = current_state.clone();
                let mut group_tile_count = [0; 34];
                group_tile_count[tile_idx] = 1;
                group_tile_count[tile_idx + 1] = 1;
                new_state_after_penchan.tile_count_array[tile_idx] =
                    current_count_array[tile_idx] - 1;
                new_state_after_penchan.tile_count_array[tile_idx + 1] =
                    current_count_array[tile_idx + 1] - 1;
                new_state_after_penchan
                    .groups_tile_counts
                    .push(group_tile_count);
                new_state_after_penchan.num_incomplete_groups += 1;
                queue.push_back(new_state_after_penchan);
                // println!(
                //     "will recursively try forming a penchan from {}",
                //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
                // );
            }

            // is it true that we should not try to make a kanchan if there is a possible sequence?
            // e.g. 2344
            if can_make_kanchan
                && !can_make_sequence
                && current_state.num_incomplete_groups < incomplete_group_budget
            {
                let mut new_state_after_kanchan = current_state.clone();
                let mut group_tile_count = [0; 34];
                group_tile_count[tile_idx] = 1;
                group_tile_count[tile_idx + 2] = 1;
                new_state_after_kanchan.tile_count_array[tile_idx] =
                    current_count_array[tile_idx] - 1;
                new_state_after_kanchan.tile_count_array[tile_idx + 2] =
                    current_count_array[tile_idx + 2] - 1;
                new_state_after_kanchan
                    .groups_tile_counts
                    .push(group_tile_count);
                new_state_after_kanchan.num_incomplete_groups += 1;
                queue.push_back(new_state_after_kanchan);
                // println!(
                //     "will recursively try forming a kanchan from {}",
                //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
                // );
            }

            // only add this as an isolated tile if we can still form an incomplete group
            if current_state.num_incomplete_groups < incomplete_group_budget {
                let mut new_state_after_isolated = current_state.clone();
                let mut group_tile_count = [0; 34];
                group_tile_count[tile_idx] = 1;
                new_state_after_isolated.tile_count_array[tile_idx] =
                    current_count_array[tile_idx] - 1;
                new_state_after_isolated
                    .groups_tile_counts
                    .push(group_tile_count);
                // how to indicate this is a floating tile vs. a taatsu / protogroup that is only one away
                new_state_after_isolated.num_incomplete_groups += 1;
                queue.push_back(new_state_after_isolated);
                // println!(
                //     "will recursively try forming an isolated tile from {}",
                //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
                // );
            }
        }

        completed_partial_states
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
            tiles: mahjong_tile::get_tiles_from_string("12m3z"),
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
            tiles: mahjong_tile::get_tiles_from_string("12m3z"),
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
            // hand: 22234m789s345p33z - waits on 2m,5m,3z, so add a 2m tile to make it a winning hand
            tiles: mahjong_tile::get_tiles_from_string("22234m789s345p33z2m"),
            ..Default::default()
        };
        assert!(hand.is_winning_shape_iterative());
        assert!(hand.is_winning_shape_recursive());
        assert!(hand.is_winning_shape_recursive_heuristic());
        assert!(hand.is_winning_shape_build_shapes());
        assert!(hand.is_winning_shape());
    }

    fn complex_winning_shape_hand() -> MahjongHand {
        MahjongHand {
            // hand: 2223444567m111z - waits on 1m,2m,3m,4m,5m,8m, so add a 3m tile to make it a winning hand
            tiles: mahjong_tile::get_tiles_from_string("2223444567m111z3m"),
            ..Default::default()
        }
    }

    #[test]
    fn hand_is_winning_shape_complex() {
        let hand = complex_winning_shape_hand();
        assert!(hand.is_winning_shape_iterative());
        assert!(hand.is_winning_shape_recursive());
        assert!(hand.is_winning_shape_recursive_heuristic());
        assert!(hand.is_winning_shape_build_shapes());
        assert!(hand.is_winning_shape());
    }

    fn not_winning_shape_hand() -> MahjongHand {
        MahjongHand {
            // hand: 122234m789s345p33z
            tiles: mahjong_tile::get_tiles_from_string("122234m789s345p33z"),
            ..Default::default()
        }
    }

    #[test]
    fn hand_is_not_winning_shape() {
        let hand = not_winning_shape_hand();
        assert!(!hand.is_winning_shape_iterative());
        assert!(!hand.is_winning_shape_recursive());
        assert!(!hand.is_winning_shape_recursive_heuristic());
        assert!(!hand.is_winning_shape_build_shapes());
        assert!(!hand.is_winning_shape());
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
    fn time_is_winning_shape_build_shapes() {
        let hand = complex_winning_shape_hand();
        let before_build_shapes = Instant::now();
        hand.is_winning_shape_build_shapes();
        println!(
            "Elapsed time for is_winning_shape_build_shapes: {:.2?}",
            before_build_shapes.elapsed()
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

    #[test]
    fn time_not_winning_shape_build_shapes() {
        let hand = not_winning_shape_hand();
        let before_build_shapes = Instant::now();
        hand.is_winning_shape_build_shapes();
        println!(
            "Elapsed time for is_winning_shape_build_shapes: {:.2?}",
            before_build_shapes.elapsed()
        );
    }

    fn ryanmen_tenpai_hand() -> MahjongHand {
        MahjongHand {
            // hand: 22s111234p34789m (wins on 25m)
            tiles: mahjong_tile::get_tiles_from_string("22s111234p34789m"),
            ..Default::default()
        }
    }

    fn shanpon_tenpai_hand() -> MahjongHand {
        MahjongHand {
            // hand: 22s111234p33789m (wins on 2s3m)
            tiles: mahjong_tile::get_tiles_from_string("22s111234p33789m"),
            ..Default::default()
        }
    }

    fn tanki_tenpai_hand() -> MahjongHand {
        MahjongHand {
            // hand: 2s111234p333789m (wins on 2s)
            tiles: mahjong_tile::get_tiles_from_string("2s111234p333789m"),
            ..Default::default()
        }
    }

    #[test]
    fn hand_is_tenpai_ryanmen() {
        let hand = ryanmen_tenpai_hand();
        assert!(hand.is_tenpai_brute_force());
        assert!(hand.is_tenpai_build_shapes());
        assert!(hand.is_tenpai());
        let tenpai_tile_ids = hand.get_tenpai_tiles_brute_force();
        assert_eq!(tenpai_tile_ids.len(), 2);
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("2m").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("5m").unwrap()));

        let tenpai_tile_ids = hand.get_tenpai_tiles_build_shapes();
        // let tenpai_tile_id_strs = mahjong_tile::tile_ids_to_strings(&tenpai_tile_ids);
        // println!("{}", tenpai_tile_id_strs.join(", "));
        assert_eq!(tenpai_tile_ids.len(), 2);
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("2m").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("5m").unwrap()));
    }

    #[test]
    fn hand_is_tenpai_shanpon() {
        let hand = shanpon_tenpai_hand();
        assert!(hand.is_tenpai_brute_force());
        assert!(hand.is_tenpai_build_shapes());
        assert!(hand.is_tenpai());
        let tenpai_tile_ids = hand.get_tenpai_tiles_brute_force();
        assert_eq!(tenpai_tile_ids.len(), 2);
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("2s").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("3m").unwrap()));

        let tenpai_tile_ids = hand.get_tenpai_tiles_build_shapes();
        // let tenpai_tile_id_strs = mahjong_tile::tile_ids_to_strings(&tenpai_tile_ids);
        // println!("{}", tenpai_tile_id_strs.join(", "));
        assert_eq!(tenpai_tile_ids.len(), 2);
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("2s").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("3m").unwrap()));
    }

    #[test]
    fn hand_is_tenpai_tanki() {
        let hand = tanki_tenpai_hand();
        assert!(hand.is_tenpai_brute_force());
        assert!(hand.is_tenpai_build_shapes());
        assert!(hand.is_tenpai());
        let tenpai_tile_ids = hand.get_tenpai_tiles_brute_force();
        assert_eq!(tenpai_tile_ids.len(), 1);
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("2s").unwrap()));

        let tenpai_tile_ids = hand.get_tenpai_tiles_build_shapes();
        // let tenpai_tile_id_strs = mahjong_tile::tile_ids_to_strings(&tenpai_tile_ids);
        // println!("{}", tenpai_tile_id_strs.join(", "));
        assert_eq!(tenpai_tile_ids.len(), 1);
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("2s").unwrap()));
    }

    #[test]
    fn time_is_tenpai_brute_force() {
        let hand = ryanmen_tenpai_hand();
        let before = Instant::now();
        hand.is_tenpai_brute_force();
        println!(
            "Elapsed time for is_tenpai_brute_force: {:.2?}",
            before.elapsed()
        );
    }

    #[test]
    fn time_is_tenpai_build_shapes() {
        let hand = ryanmen_tenpai_hand();
        let before = Instant::now();
        hand.is_tenpai_build_shapes();
        println!(
            "Elapsed time for is_tenpai_build_shapes: {:.2?}",
            before.elapsed()
        );
    }

    #[test]
    fn time_get_tenpai_tiles_brute_force() {
        let hand = ryanmen_tenpai_hand();
        let before = Instant::now();
        hand.get_tenpai_tiles_brute_force();
        println!(
            "Elapsed time for get_tenpai_tiles_brute_force: {:.2?}",
            before.elapsed()
        );
    }

    fn not_tenpai_hand() -> MahjongHand {
        MahjongHand {
            // hand: 23445588s345p11z
            tiles: mahjong_tile::get_tiles_from_string("23445588s345p11z"),
            ..Default::default()
        }
    }

    #[test]
    fn hand_is_not_tenpai() {
        let hand = not_tenpai_hand();
        assert!(!hand.is_tenpai_brute_force());
        assert!(!hand.is_tenpai_build_shapes());
        assert!(!hand.is_tenpai());
        let tenpai_tile_ids = hand.get_tenpai_tiles_brute_force();
        assert_eq!(tenpai_tile_ids.len(), 0);
        let tenpai_tile_ids = hand.get_tenpai_tiles_build_shapes();
        assert_eq!(tenpai_tile_ids.len(), 0);
    }

    fn chuuren_poutou_tenpai_hand() -> MahjongHand {
        // aka pure nine gates
        MahjongHand {
            // hand: 1112345678999p (wins on 123456789p)
            tiles: mahjong_tile::get_tiles_from_string("1112345678999p"),
            ..Default::default()
        }
    }

    #[test]
    fn chuuren_poutou_hand_is_tenpai() {
        let hand = chuuren_poutou_tenpai_hand();
        assert!(hand.is_tenpai_brute_force());
        assert!(hand.is_tenpai_build_shapes());
        assert!(hand.is_tenpai());
        let tenpai_tile_ids = hand.get_tenpai_tiles_brute_force();
        let tenpai_tile_id_strs = mahjong_tile::tile_ids_to_strings(&tenpai_tile_ids);
        println!("{}", tenpai_tile_id_strs.join(", "));
        assert_eq!(tenpai_tile_ids.len(), 9);
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("1p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("2p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("3p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("4p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("5p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("6p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("7p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("8p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("9p").unwrap()));

        let tenpai_tile_ids = hand.get_tenpai_tiles_build_shapes();
        assert_eq!(tenpai_tile_ids.len(), 9);
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("1p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("2p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("3p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("4p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("5p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("6p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("7p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("8p").unwrap()));
        assert!(tenpai_tile_ids.contains(&mahjong_tile::get_id_from_tile_text("9p").unwrap()));
    }

    #[test]
    fn time_get_tenpai_tiles_brute_force_chuuren_poutou() {
        let hand = chuuren_poutou_tenpai_hand();
        let before = Instant::now();
        hand.get_tenpai_tiles_brute_force();
        println!(
            "Elapsed time for get_tenpai_tiles_brute_force (on chuuren poutou hand): {:.2?}",
            before.elapsed()
        );
    }

    #[test]
    fn test_can_make_sequence() {
        // valid sequence 1m-2m-3m
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_1m = mahjong_tile::get_id_from_tile_text("1m").unwrap();
        tile_counts[usize::from(tile_id_1m)] = 1;
        let tile_id_2m = mahjong_tile::get_id_from_tile_text("2m").unwrap();
        tile_counts[usize::from(tile_id_2m)] = 1;
        let tile_id_3m = mahjong_tile::get_id_from_tile_text("3m").unwrap();
        tile_counts[usize::from(tile_id_3m)] = 1;
        let tile_id_4m = mahjong_tile::get_id_from_tile_text("4m").unwrap();

        assert!(can_make_sequence(&tile_counts, tile_id_1m));
        assert!(can_make_sequence(&tile_counts, tile_id_2m));
        assert!(can_make_sequence(&tile_counts, tile_id_3m));
        assert!(!can_make_sequence(&tile_counts, tile_id_4m));

        // valid sequence 4s-5s-6s
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_4s = mahjong_tile::get_id_from_tile_text("4s").unwrap();
        tile_counts[usize::from(tile_id_4s)] = 1;
        let tile_id_5s = mahjong_tile::get_id_from_tile_text("5s").unwrap();
        tile_counts[usize::from(tile_id_5s)] = 1;
        let tile_id_6s = mahjong_tile::get_id_from_tile_text("6s").unwrap();
        tile_counts[usize::from(tile_id_6s)] = 1;
        let tile_id_7s = mahjong_tile::get_id_from_tile_text("7s").unwrap();

        assert!(can_make_sequence(&tile_counts, tile_id_4s));
        assert!(can_make_sequence(&tile_counts, tile_id_5s));
        assert!(can_make_sequence(&tile_counts, tile_id_6s));
        assert!(!can_make_sequence(&tile_counts, tile_id_7s));

        // sequence cannot wrap around (9m-1m-2m is not a sequence)
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_9m = mahjong_tile::get_id_from_tile_text("9m").unwrap();
        tile_counts[usize::from(tile_id_9m)] = 1;
        let tile_id_1m = mahjong_tile::get_id_from_tile_text("1m").unwrap();
        tile_counts[usize::from(tile_id_1m)] = 1;
        let tile_id_2m = mahjong_tile::get_id_from_tile_text("2m").unwrap();
        tile_counts[usize::from(tile_id_2m)] = 1;

        assert!(!can_make_sequence(&tile_counts, tile_id_9m));
        assert!(!can_make_sequence(&tile_counts, tile_id_1m));
        assert!(!can_make_sequence(&tile_counts, tile_id_2m));

        // sequence cannot wrap in tile_id (9m-1p-2p is not a sequence)
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_9m = mahjong_tile::get_id_from_tile_text("9m").unwrap();
        tile_counts[usize::from(tile_id_9m)] = 1;
        let tile_id_1p = tile_id_9m + 1;
        tile_counts[usize::from(tile_id_1p)] = 1;
        let tile_id_2p = tile_id_9m + 2;
        tile_counts[usize::from(tile_id_2p)] = 1;

        assert!(!can_make_sequence(&tile_counts, tile_id_9m));
        assert!(!can_make_sequence(&tile_counts, tile_id_1p));
        assert!(!can_make_sequence(&tile_counts, tile_id_2p));

        // sequence must be in the same suit
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_7m = mahjong_tile::get_id_from_tile_text("7m").unwrap();
        tile_counts[usize::from(tile_id_7m)] = 1;
        let tile_id_8p = mahjong_tile::get_id_from_tile_text("8p").unwrap();
        tile_counts[usize::from(tile_id_8p)] = 1;
        let tile_id_9p = mahjong_tile::get_id_from_tile_text("9p").unwrap();
        tile_counts[usize::from(tile_id_9p)] = 1;

        assert!(!can_make_sequence(&tile_counts, tile_id_7m));
        assert!(!can_make_sequence(&tile_counts, tile_id_8p));
        assert!(!can_make_sequence(&tile_counts, tile_id_9p));

        // honor tiles cannot form a sequence
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_1z = mahjong_tile::get_id_from_tile_text("1z").unwrap();
        tile_counts[usize::from(tile_id_1z)] = 1;
        let tile_id_2z = mahjong_tile::get_id_from_tile_text("2z").unwrap();
        tile_counts[usize::from(tile_id_2z)] = 1;
        let tile_id_3z = mahjong_tile::get_id_from_tile_text("3z").unwrap();
        tile_counts[usize::from(tile_id_3z)] = 1;
        assert!(!can_make_sequence(&tile_counts, tile_id_1z));
        assert!(!can_make_sequence(&tile_counts, tile_id_2z));
        assert!(!can_make_sequence(&tile_counts, tile_id_3z));
    }

    #[test]
    fn test_can_make_ryanmen() {
        // valid ryanmen 2m-3m
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_1m = mahjong_tile::get_id_from_tile_text("1m").unwrap();
        let tile_id_2m = mahjong_tile::get_id_from_tile_text("2m").unwrap();
        tile_counts[usize::from(tile_id_2m)] = 1;
        let tile_id_3m = mahjong_tile::get_id_from_tile_text("3m").unwrap();
        tile_counts[usize::from(tile_id_3m)] = 1;
        let tile_id_4m = mahjong_tile::get_id_from_tile_text("4m").unwrap();

        assert!(!can_make_ryanmen(&tile_counts, tile_id_1m));
        assert!(can_make_ryanmen(&tile_counts, tile_id_2m));
        assert!(can_make_ryanmen(&tile_counts, tile_id_3m));
        assert!(!can_make_ryanmen(&tile_counts, tile_id_4m));

        // valid ryanmen 5s-6s
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_4s = mahjong_tile::get_id_from_tile_text("4s").unwrap();
        let tile_id_5s = mahjong_tile::get_id_from_tile_text("5s").unwrap();
        tile_counts[usize::from(tile_id_5s)] = 1;
        let tile_id_6s = mahjong_tile::get_id_from_tile_text("6s").unwrap();
        tile_counts[usize::from(tile_id_6s)] = 1;
        let tile_id_7s = mahjong_tile::get_id_from_tile_text("7s").unwrap();

        assert!(!can_make_ryanmen(&tile_counts, tile_id_4s));
        assert!(can_make_ryanmen(&tile_counts, tile_id_5s));
        assert!(can_make_ryanmen(&tile_counts, tile_id_6s));
        assert!(!can_make_ryanmen(&tile_counts, tile_id_7s));

        // ryanmen cannot include terminal (8m-9m is not a ryanmen)
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_8m = mahjong_tile::get_id_from_tile_text("8m").unwrap();
        tile_counts[usize::from(tile_id_8m)] = 1;
        let tile_id_9m = mahjong_tile::get_id_from_tile_text("9m").unwrap();
        tile_counts[usize::from(tile_id_9m)] = 1;

        assert!(!can_make_ryanmen(&tile_counts, tile_id_8m));
        assert!(!can_make_ryanmen(&tile_counts, tile_id_9m));

        // ryanmen cannot wrap in tile_id (9p-1s is not a ryanmen)
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_9p = mahjong_tile::get_id_from_tile_text("9p").unwrap();
        tile_counts[usize::from(tile_id_9p)] = 1;
        let tile_id_1s = tile_id_9p + 1;
        tile_counts[usize::from(tile_id_1s)] = 1;

        assert!(!can_make_ryanmen(&tile_counts, tile_id_9p));
        assert!(!can_make_ryanmen(&tile_counts, tile_id_1s));

        // ryanmen must be in the same suit (7m-8p is not a ryanmen)
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_7m = mahjong_tile::get_id_from_tile_text("7m").unwrap();
        tile_counts[usize::from(tile_id_7m)] = 1;
        let tile_id_8p = mahjong_tile::get_id_from_tile_text("8p").unwrap();
        tile_counts[usize::from(tile_id_8p)] = 1;

        assert!(!can_make_ryanmen(&tile_counts, tile_id_7m));
        assert!(!can_make_ryanmen(&tile_counts, tile_id_8p));

        // honor tiles cannot form a ryanmen
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_2z = mahjong_tile::get_id_from_tile_text("2z").unwrap();
        tile_counts[usize::from(tile_id_2z)] = 1;
        let tile_id_3z = mahjong_tile::get_id_from_tile_text("3z").unwrap();
        tile_counts[usize::from(tile_id_3z)] = 1;
        assert!(!can_make_ryanmen(&tile_counts, tile_id_2z));
        assert!(!can_make_ryanmen(&tile_counts, tile_id_3z));
    }

    #[test]
    fn test_can_make_penchan() {
        // valid penchan 1m-2m
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_1m = mahjong_tile::get_id_from_tile_text("1m").unwrap();
        tile_counts[usize::from(tile_id_1m)] = 1;
        let tile_id_2m = mahjong_tile::get_id_from_tile_text("2m").unwrap();
        tile_counts[usize::from(tile_id_2m)] = 1;
        let tile_id_3m = mahjong_tile::get_id_from_tile_text("3m").unwrap();

        assert!(can_make_penchan(&tile_counts, tile_id_1m));
        assert!(can_make_penchan(&tile_counts, tile_id_2m));
        assert!(!can_make_penchan(&tile_counts, tile_id_3m));

        // valid penchan 8s-9s
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_8s = mahjong_tile::get_id_from_tile_text("8s").unwrap();
        tile_counts[usize::from(tile_id_8s)] = 1;
        let tile_id_9s = mahjong_tile::get_id_from_tile_text("9s").unwrap();
        tile_counts[usize::from(tile_id_9s)] = 1;
        let tile_id_7s = mahjong_tile::get_id_from_tile_text("7s").unwrap();

        assert!(!can_make_penchan(&tile_counts, tile_id_7s));
        assert!(can_make_penchan(&tile_counts, tile_id_8s));
        assert!(can_make_penchan(&tile_counts, tile_id_9s));

        // 7p-8p is not a penchan
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_8p = mahjong_tile::get_id_from_tile_text("8p").unwrap();
        tile_counts[usize::from(tile_id_8p)] = 1;
        let tile_id_7p = mahjong_tile::get_id_from_tile_text("7p").unwrap();
        tile_counts[usize::from(tile_id_7p)] = 1;

        assert!(!can_make_penchan(&tile_counts, tile_id_7p));
        assert!(!can_make_penchan(&tile_counts, tile_id_8p));

        // penchan cannot wrap in tile_id (9p-1s is not a penchan)
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_9p = mahjong_tile::get_id_from_tile_text("9p").unwrap();
        tile_counts[usize::from(tile_id_9p)] = 1;
        let tile_id_1s = tile_id_9p + 1;
        tile_counts[usize::from(tile_id_1s)] = 1;

        assert!(!can_make_penchan(&tile_counts, tile_id_9p));
        assert!(!can_make_penchan(&tile_counts, tile_id_1s));

        // penchan must be in the same suit (8p-9m is not a penchan)
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_8p = mahjong_tile::get_id_from_tile_text("8p").unwrap();
        tile_counts[usize::from(tile_id_8p)] = 1;
        let tile_id_9m = mahjong_tile::get_id_from_tile_text("9m").unwrap();
        tile_counts[usize::from(tile_id_9m)] = 1;

        assert!(!can_make_penchan(&tile_counts, tile_id_8p));
        assert!(!can_make_penchan(&tile_counts, tile_id_9m));

        // honor tiles cannot form a penchan
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_6z = mahjong_tile::get_id_from_tile_text("6z").unwrap();
        tile_counts[usize::from(tile_id_6z)] = 1;
        let tile_id_7z = mahjong_tile::get_id_from_tile_text("7z").unwrap();
        tile_counts[usize::from(tile_id_7z)] = 1;
        assert!(!can_make_penchan(&tile_counts, tile_id_6z));
        assert!(!can_make_penchan(&tile_counts, tile_id_7z));
    }

    #[test]
    fn test_can_make_kanchan() {
        // valid kanchan 1m-3m
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_1m = mahjong_tile::get_id_from_tile_text("1m").unwrap();
        tile_counts[usize::from(tile_id_1m)] = 1;
        let tile_id_2m = mahjong_tile::get_id_from_tile_text("2m").unwrap();
        let tile_id_3m = mahjong_tile::get_id_from_tile_text("3m").unwrap();
        tile_counts[usize::from(tile_id_3m)] = 1;
        let tile_id_4m = mahjong_tile::get_id_from_tile_text("4m").unwrap();

        assert!(can_make_kanchan(&tile_counts, tile_id_1m));
        assert!(!can_make_kanchan(&tile_counts, tile_id_2m));
        assert!(can_make_kanchan(&tile_counts, tile_id_3m));
        assert!(!can_make_kanchan(&tile_counts, tile_id_4m));

        // valid kanchan 4s-6s (from 4s-5s-6s)
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_4s = mahjong_tile::get_id_from_tile_text("4s").unwrap();
        tile_counts[usize::from(tile_id_4s)] = 1;
        let tile_id_5s = mahjong_tile::get_id_from_tile_text("5s").unwrap();
        tile_counts[usize::from(tile_id_5s)] = 1;
        let tile_id_6s = mahjong_tile::get_id_from_tile_text("6s").unwrap();
        tile_counts[usize::from(tile_id_6s)] = 1;
        let tile_id_7s = mahjong_tile::get_id_from_tile_text("7s").unwrap();

        assert!(can_make_kanchan(&tile_counts, tile_id_4s));
        assert!(!can_make_kanchan(&tile_counts, tile_id_5s));
        assert!(can_make_kanchan(&tile_counts, tile_id_6s));
        assert!(!can_make_kanchan(&tile_counts, tile_id_7s));

        // kanchan cannot wrap around (8m-1m is not a kanchan)
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_8m = mahjong_tile::get_id_from_tile_text("8m").unwrap();
        tile_counts[usize::from(tile_id_8m)] = 1;
        let tile_id_1m = mahjong_tile::get_id_from_tile_text("1m").unwrap();
        tile_counts[usize::from(tile_id_1m)] = 1;

        assert!(!can_make_kanchan(&tile_counts, tile_id_8m));
        assert!(!can_make_kanchan(&tile_counts, tile_id_1m));

        // kanchan cannot wrap in tile_id (9m-2p is not a kanchan)
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_9m = mahjong_tile::get_id_from_tile_text("9m").unwrap();
        tile_counts[usize::from(tile_id_9m)] = 1;
        let tile_id_2p = tile_id_9m + 2;
        tile_counts[usize::from(tile_id_2p)] = 1;

        assert!(!can_make_kanchan(&tile_counts, tile_id_9m));
        assert!(!can_make_kanchan(&tile_counts, tile_id_2p));

        // kanchan must be in the same suit (7m-9p is not kanchan)
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_7m = mahjong_tile::get_id_from_tile_text("7m").unwrap();
        tile_counts[usize::from(tile_id_7m)] = 1;
        let tile_id_9p = mahjong_tile::get_id_from_tile_text("9p").unwrap();
        tile_counts[usize::from(tile_id_9p)] = 1;

        assert!(!can_make_kanchan(&tile_counts, tile_id_7m));
        assert!(!can_make_kanchan(&tile_counts, tile_id_9p));

        // honor tiles cannot form a kanchan
        let mut tile_counts: [u8; 34] = [0; 34];
        let tile_id_1z = mahjong_tile::get_id_from_tile_text("1z").unwrap();
        tile_counts[usize::from(tile_id_1z)] = 1;
        let tile_id_3z = mahjong_tile::get_id_from_tile_text("3z").unwrap();
        tile_counts[usize::from(tile_id_3z)] = 1;
        assert!(!can_make_kanchan(&tile_counts, tile_id_1z));
        assert!(!can_make_kanchan(&tile_counts, tile_id_3z));
    }

    #[test]
    fn build_shapes_complete_hand() {
        let hand = complex_winning_shape_hand();
        let groupings = hand.build_shapes(0);
        println!("returned groupings:");
        for grouping in groupings.iter() {
            println!("{}", grouping);
        }

        let mut complete_groupings = vec![];
        for grouping in groupings.iter() {
            if grouping.num_complete_groups == 4
                && grouping.num_incomplete_groups == 0
                && grouping.num_pairs == 1
            {
                complete_groupings.push(grouping.clone());
            }
        }

        assert_eq!(complete_groupings.len(), 1);
        let returned_grouping = complete_groupings.get(0).unwrap();
        assert_eq!(returned_grouping.tile_count_array, [0; 34]);
        assert_eq!(returned_grouping.num_complete_groups, 4);
        assert_eq!(returned_grouping.num_incomplete_groups, 0);
        assert_eq!(returned_grouping.num_pairs, 1);

        let mut honor_group_counts = [0; 34];
        honor_group_counts[usize::from(mahjong_tile::get_id_from_tile_text("1z").unwrap())] = 3;
        assert!(returned_grouping
            .groups_tile_counts
            .contains(&honor_group_counts));

        let mut meld_group_counts = [0; 34];
        meld_group_counts[usize::from(mahjong_tile::get_id_from_tile_text("2m").unwrap())] = 3;
        assert!(returned_grouping
            .groups_tile_counts
            .contains(&meld_group_counts));

        let mut pair_group_counts = [0; 34];
        pair_group_counts[usize::from(mahjong_tile::get_id_from_tile_text("3m").unwrap())] = 2;
        assert!(returned_grouping
            .groups_tile_counts
            .contains(&pair_group_counts));

        let mut meld_group_counts = [0; 34];
        meld_group_counts[usize::from(mahjong_tile::get_id_from_tile_text("4m").unwrap())] = 3;
        assert!(returned_grouping
            .groups_tile_counts
            .contains(&meld_group_counts));

        let mut sequence_group_counts = [0; 34];
        sequence_group_counts[usize::from(mahjong_tile::get_id_from_tile_text("5m").unwrap())] = 1;
        sequence_group_counts[usize::from(mahjong_tile::get_id_from_tile_text("6m").unwrap())] = 1;
        sequence_group_counts[usize::from(mahjong_tile::get_id_from_tile_text("7m").unwrap())] = 1;
        assert!(returned_grouping
            .groups_tile_counts
            .contains(&sequence_group_counts));
    }

    fn nobetan_shape() -> MahjongHand {
        MahjongHand {
            // hand: 2m3m4m5m
            tiles: mahjong_tile::get_tiles_from_string("2345m"),
            ..Default::default()
        }
    }

    // TODO fix failing test
    #[test]
    fn build_shapes_nobetan_shape() {
        // test with a four-in-a-row shape (yonrenkei, aka nobetan)
        let hand = nobetan_shape();
        let groupings = hand.build_shapes(2);
        println!("returned groupings:");
        for grouping in groupings.iter() {
            println!("{}", grouping);
        }

        assert_eq!(groupings.len(), 3);
        // there are three main ways to parse this shape
        // 234-5, 2-345, and 23-45
        // other interpretations are unnecessarily wide/inefficient e.g. 24-35, or 2-34-5
    }

    fn embedded_ryankan_shape() -> MahjongHand {
        MahjongHand {
            // hand: 2m3m3m4m5m7m8m
            tiles: mahjong_tile::get_tiles_from_string("234578m"),
            ..Default::default()
        }
    }

    // TODO fix failing test
    #[test]
    fn build_shapes_embedded_ryankan_shape() {
        // test with a shape that has an embedded ryankan (3m-5m-7m)
        let hand = embedded_ryankan_shape();
        let groupings = hand.build_shapes(2);
        println!("returned groupings:");
        for grouping in groupings.iter() {
            println!("{}", grouping);
        }

        assert_eq!(groupings.len(), 3);
        // there are several main ways to parse this shape
        // 23-345-78, 234-357-8, 234-35-78
        // other interpretations are unnecessarily wide/inefficient
    }
}
