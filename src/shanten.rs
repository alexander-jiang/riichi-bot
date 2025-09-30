extern crate test;

pub use crate::mahjong_hand;
pub use crate::mahjong_tile;
use crate::mahjong_tile::{MahjongTileId, get_distinct_tile_ids_from_count_array, get_total_tiles_from_count_array};
use std::cmp::min;
use std::collections::{HashMap, VecDeque};
use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum MeldType {
    Triplet,    // e.g. 888
    Quadruplet, // e.g. 5555
    Sequence,   // e.g. 123 or 567
    Pair,
    Ryanmen,    // open wait e.g. 34 or 67
    Kanchan,    // closed wait e.g. 35 or 79
    Penchan,    // edge wait e.g. 12 or 89
    SingleTile, // for tanki wait or isolated tiles
}

impl MeldType {
    fn is_complete(&self) -> bool {
        match self {
            Self::Quadruplet | Self::Sequence | Self::Triplet => true,
            Self::Pair | Self::Ryanmen | Self::Kanchan | Self::Penchan | Self::SingleTile => false,
        }
    }
}

#[derive(Clone)]
pub struct TileMeld {
    meld_type: MeldType,
    tile_ids: Vec<MahjongTileId>,
}

fn tile_ids_are_all_same<T: Into<MahjongTileId> + Clone>(tile_ids: &Vec<T>) -> bool {
    if tile_ids.len() == 0 {
        return true;
    }
    let first_tile_id: MahjongTileId = tile_ids.get(0).unwrap().clone().into();
    for tile_id in tile_ids.iter() {
        let tile_id: MahjongTileId = tile_id.clone().into();
        if tile_id != first_tile_id {
            return false;
        }
    }
    true
}

fn tile_ids_are_sequence<T: Into<MahjongTileId> + Clone>(tile_ids: &Vec<T>) -> bool {
    if tile_ids.len() != 3 {
        return false;
    }
    let mut sorted_tile_ids: Vec<MahjongTileId> = tile_ids.iter().cloned().map(|t| t.into()).collect();
    sorted_tile_ids.sort();
    let min_tile_id = *sorted_tile_ids.get(0).unwrap();
    let mid_tile_id = *sorted_tile_ids.get(1).unwrap();
    let max_tile_id = *sorted_tile_ids.get(2).unwrap();
    // all tiles must not be honor tiles
    if max_tile_id.0 >= mahjong_tile::FIRST_HONOR_ID {
        return false;
    }

    let min_tile_rank = mahjong_tile::get_num_tile_rank(min_tile_id).unwrap();
    let max_tile_rank = mahjong_tile::get_num_tile_rank(max_tile_id).unwrap();
    // tile ids must be sequential, but cannot wrap around the ends of a suit
    (max_tile_id.0 == mid_tile_id.0 + 1)
        && (mid_tile_id.0 == min_tile_id.0 + 1)
        && (min_tile_rank <= 7 && max_tile_rank >= 3)
}

fn tile_ids_are_ryanmen<T: Into<MahjongTileId> + Clone>(tile_ids: &Vec<T>) -> bool {
    if tile_ids.len() != 2 {
        return false;
    }
    let mut sorted_tile_ids: Vec<MahjongTileId> = tile_ids.iter().cloned().map(|t| t.into()).collect();
    sorted_tile_ids.sort();
    let min_tile_id = *sorted_tile_ids.get(0).unwrap();
    let max_tile_id = *sorted_tile_ids.get(1).unwrap();
    // all tiles must not be honor tiles
    if max_tile_id.0 >= mahjong_tile::FIRST_HONOR_ID {
        return false;
    }

    let min_tile_rank = mahjong_tile::get_num_tile_rank(min_tile_id).unwrap();
    max_tile_id.0 == min_tile_id.0 + 1 && (min_tile_rank >= 2 && min_tile_rank <= 7)
}

fn tile_ids_are_kanchan<T: Into<MahjongTileId> + Clone>(tile_ids: &Vec<T>) -> bool {
    if tile_ids.len() != 2 {
        return false;
    }
    let mut sorted_tile_ids: Vec<MahjongTileId> = tile_ids.iter().cloned().map(|t| t.into()).collect();
    sorted_tile_ids.sort();
    let min_tile_id = *sorted_tile_ids.get(0).unwrap();
    let max_tile_id = *sorted_tile_ids.get(1).unwrap();
    // all tiles must not be honor tiles
    if max_tile_id.0 >= mahjong_tile::FIRST_HONOR_ID {
        return false;
    }

    let min_tile_rank = mahjong_tile::get_num_tile_rank(min_tile_id).unwrap();
    let max_tile_rank = mahjong_tile::get_num_tile_rank(max_tile_id).unwrap();
    max_tile_id.0 == min_tile_id.0 + 2 && (min_tile_rank <= 7 && max_tile_rank >= 3)
}

fn tile_ids_are_penchan<T: Into<MahjongTileId> + Clone>(tile_ids: &Vec<T>) -> bool {
    if tile_ids.len() != 2 {
        return false;
    }
    let mut sorted_tile_ids: Vec<MahjongTileId> = tile_ids.iter().cloned().map(|t| t.into()).collect();
    sorted_tile_ids.sort();
    let min_tile_id = *sorted_tile_ids.get(0).unwrap();
    let max_tile_id = *sorted_tile_ids.get(1).unwrap();
    // all tiles must not be honor tiles
    if max_tile_id.0 >= mahjong_tile::FIRST_HONOR_ID {
        return false;
    }

    let min_tile_rank = mahjong_tile::get_num_tile_rank(min_tile_id).unwrap();
    max_tile_id.0 == min_tile_id.0 + 1 && (min_tile_rank == 1 || min_tile_rank == 8)
}

impl TileMeld {
    /// Constructor for TileMeld which also validates the meld and sorts the tile_ids,
    /// which is useful for optimizing some future steps.
    fn new<T: Into<MahjongTileId> + Clone>(tile_ids: Vec<T>) -> Self {
        let meld_type = match tile_ids.len() {
            1 => MeldType::SingleTile,
            2 => {
                // check either the tiles are the same (pair), or can form a ryanmen, kanchan, or penchan
                if tile_ids_are_all_same(&tile_ids) {
                    MeldType::Pair
                } else if tile_ids_are_ryanmen(&tile_ids) {
                    MeldType::Ryanmen
                } else if tile_ids_are_kanchan(&tile_ids) {
                    MeldType::Kanchan
                } else if tile_ids_are_penchan(&tile_ids) {
                    MeldType::Penchan
                } else {
                    panic!("invalid meld with 2 tiles")
                }
            }
            3 => {
                // check all tiles are the same (triplet) or can form a sequence
                if tile_ids_are_all_same(&tile_ids) {
                    MeldType::Triplet
                } else if tile_ids_are_sequence(&tile_ids) {
                    MeldType::Sequence
                } else {
                    panic!("invalid meld with 3 tiles")
                }
            }
            4 => {
                // check all tiles are the same (quadruplet)
                if tile_ids_are_all_same(&tile_ids) {
                    MeldType::Quadruplet
                } else {
                    panic!("invalid meld with 4 tiles")
                }
            }
            0 => panic!("cannot form a meld with no tiles"),
            _ => panic!("invalid meld: too many tiles"),
        };
        let mut sorted_tile_ids: Vec<MahjongTileId> = tile_ids.iter().cloned().map(|t| t.into()).collect();
        sorted_tile_ids.sort();
        TileMeld {
            meld_type: meld_type,
            tile_ids: sorted_tile_ids,
        }
    }

    fn is_complete(&self) -> bool {
        self.meld_type.is_complete()
    }

    fn tile_ids_to_complete_group(&self) -> Vec<MahjongTileId> {
        // assumes the tile_ids are sorted
        match self.meld_type {
            MeldType::SingleTile => {
                let tile_id = self.tile_ids.get(0).unwrap();
                let tile_id = *tile_id;
                let mut tile_ids = vec![tile_id];
                match mahjong_tile::get_num_tile_rank(tile_id) {
                    Some(tile_rank) => {
                        if tile_rank <= 8 {
                            tile_ids.push(MahjongTileId(tile_id.0 + 1));
                        }
                        if tile_rank <= 7 {
                            tile_ids.push(MahjongTileId(tile_id.0 + 2));
                        }
                        if tile_rank >= 2 {
                            tile_ids.push(MahjongTileId(tile_id.0 - 1));
                        }
                        if tile_rank >= 3 {
                            tile_ids.push(MahjongTileId(tile_id.0 - 2));
                        }
                    }
                    None => {}
                };
                tile_ids
            }
            MeldType::Pair => {
                let tile_id = self.tile_ids.get(0).unwrap();
                vec![*tile_id]
            }
            MeldType::Ryanmen => {
                let min_tile_id = self.tile_ids.get(0).unwrap();
                let max_tile_id = self.tile_ids.get(1).unwrap();
                vec![MahjongTileId(min_tile_id.0 - 1), MahjongTileId(max_tile_id.0 + 1)]
            }
            MeldType::Kanchan => {
                let min_tile_id = self.tile_ids.get(0).unwrap();
                vec![MahjongTileId(min_tile_id.0 + 1)]
            }
            MeldType::Penchan => {
                let min_tile_id = self.tile_ids.get(0).unwrap();
                let min_tile_rank = mahjong_tile::get_num_tile_rank(*min_tile_id).unwrap();
                if min_tile_rank == 1 {
                    vec![MahjongTileId(min_tile_id.0 + 2)]
                } else if min_tile_rank == 8 {
                    vec![MahjongTileId(min_tile_id.0 - 1)]
                } else {
                    panic!("invalid penchan! min tile rank should be 1 or 8")
                }
            }
            t if t.is_complete() => Vec::new(),
            _ => panic!("unexpected meld type"),
        }
    }
}

#[derive(Clone)]
pub struct HandInterpretation {
    total_tile_count_array: [u8; 34],
    groups: Vec<TileMeld>,
}

impl fmt::Display for HandInterpretation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tile_groups_string_vec = vec![];
        for tile_group_count_array in self.groups.iter() {
            let mut meld_tile_ids_string = String::new();
            for &tile_id in tile_group_count_array.tile_ids.iter() {
                meld_tile_ids_string
                    .push_str(&mahjong_tile::get_tile_text_from_id(tile_id).unwrap());
            }
            tile_groups_string_vec.push(meld_tile_ids_string);
        }
        let tile_groups_string = tile_groups_string_vec.join(", ");
        write!(
            f,
            "tiles={}, tile_groups=[{}]",
            mahjong_hand::tile_count_array_to_string(&self.total_tile_count_array),
            tile_groups_string,
        )
    }
}

fn get_num_complete_groups(groups: &Vec<TileMeld>) -> i8 {
    let mut num_complete_groups = 0;
    for group in groups.iter() {
        if group.is_complete() {
            num_complete_groups += 1;
        }
    }
    num_complete_groups
}

fn get_num_incomplete_groups(groups: &Vec<TileMeld>) -> i8 {
    let mut num_incomplete_groups = 0; // this is taatsu + pairs
    for group in groups.iter() {
        if !group.is_complete() && group.tile_ids.len() == 2 {
            // exclude single / isolated tiles, as incomplete groups means it only
            // needs one more tile to become a complete group
            num_incomplete_groups += 1; // note that this includes taatsu and pairs!
        }
    }
    num_incomplete_groups
}

fn get_total_num_tiles(groups: &Vec<TileMeld>) -> usize {
    let mut num_tiles = 0;
    for group in groups.iter() {
        num_tiles += group.tile_ids.len();
    }
    num_tiles
}

fn get_pair_tile_ids(groups: &Vec<TileMeld>) -> Vec<MahjongTileId> {
    let mut pair_tile_ids = vec![];
    for group in groups.iter() {
        if group.meld_type == MeldType::Pair {
            let tile_id = group.tile_ids.get(0).unwrap();
            pair_tile_ids.push(*tile_id);
        }
    }
    pair_tile_ids
}



fn standard_shanten_formula(
    num_complete_groups: i8,
    num_incomplete_groups: i8,
    has_pair: bool,
) -> i8 {
    let mut shanten = 8i8;
    // first, only count up to 4 groups (either complete or incomplete)
    shanten -= 2 * num_complete_groups;
    shanten -= min(num_incomplete_groups, 4 - num_complete_groups);
    // then reduce by 1 if there is a pair and at least 5 groups (one of the pairs can count towards the 5)
    if has_pair && num_complete_groups + num_incomplete_groups >= 5 {
        shanten -= 1;
    }
    shanten
}

impl HandInterpretation {
    fn num_tiles(&self) -> u8 {
        let mut total_num_tiles = 0;
        for &tile_count in self.total_tile_count_array.iter() {
            total_num_tiles += tile_count;
        }
        total_num_tiles
    }

    fn get_num_complete_groups(&self) -> i8 {
        get_num_complete_groups(&self.groups)
    }

    fn get_num_incomplete_groups(&self) -> i8 {
        get_num_incomplete_groups(&self.groups)
    }

    fn get_pair_tile_ids(&self) -> Vec<MahjongTileId> {
        get_pair_tile_ids(&self.groups)
    }

    fn get_single_tile_ids(&self) -> Vec<MahjongTileId> {
        let mut single_tile_ids = vec![];
        for group in self.groups.iter() {
            if group.meld_type == MeldType::SingleTile {
                let tile_id = group.tile_ids.get(0).unwrap();
                single_tile_ids.push(*tile_id);
            }
        }
        single_tile_ids
    }

    fn get_standard_shanten(&self) -> i8 {
        if self.num_tiles() != 13 && self.num_tiles() != 14 {
            // TODO eventually will need to handle the case when there are more tiles due to quads
            panic!("invalid number of tiles")
        }
        // compute standard shanten: count complete groups, incomplete groups, and pairs
        let num_complete_groups = self.get_num_complete_groups();
        let num_incomplete_groups = self.get_num_incomplete_groups(); // this is taatsu + pairs
        let has_pair = !self.get_pair_tile_ids().is_empty();

        let shanten =
            standard_shanten_formula(num_complete_groups, num_incomplete_groups, has_pair);

        // println!(
        //     "hand interpretation {} has {} complete groups, {} incomplete groups, has_pair={} => shanten = {}",
        //     self, num_complete_groups, num_incomplete_groups, has_pair, shanten
        // );
        shanten
    }

    fn get_ukiere(&self) -> Vec<MahjongTileId> {
        if self.get_standard_shanten() == -1 {
            return Vec::new();
        }
        if self.num_tiles() != 13 {
            // TODO eventually will need to handle the case when there are more tiles due to quads
            panic!("invalid number of tiles")
        }

        let num_complete_groups = self.get_num_complete_groups();
        let num_incomplete_groups = self.get_num_incomplete_groups(); // this is taatsu + pairs
        let pair_tile_ids = self.get_pair_tile_ids();
        let single_tile_ids = self.get_single_tile_ids();
        let has_pair = !pair_tile_ids.is_empty();
        let total_groups = num_complete_groups + num_incomplete_groups;

        let mut ukiere_tile_ids = Vec::new();
        for group in &self.groups {
            let mut tile_ids = group.tile_ids_to_complete_group();
            if group.meld_type == MeldType::SingleTile {
                // for isolated tile, it only adds to the ukiere if making another
                // incomplete group would reduce shanten
                if total_groups >= 5 {
                    continue;
                }
                let single_tile_id = *(group.tile_ids.get(0).unwrap());
                if total_groups == 4 && !has_pair {
                    // in this case, you can decrease shanten, but only by drawing the same tile
                    // to form a new group which is the only pair
                    tile_ids = vec![single_tile_id];
                }

                // edge case - if the hand already contains a pair of the same value as
                // the isolated tile, then drawing another copy of the isolated tile doesn't reduce shanten
                if pair_tile_ids.contains(&single_tile_id) {
                    let tile_index = tile_ids.iter().find(|&&x| x == single_tile_id);
                    if tile_index.is_some() {
                        tile_ids.swap_remove(usize::from(*tile_index.unwrap()));
                    }
                }
            } else if group.meld_type == MeldType::Pair {
                let pair_tile_id = group.tile_ids.get(0).unwrap();
                // if there are 4 complete groups (and this is a pair), then we have a complete hand
                // if there are 5 total groups (complete groups + incomplete groups) with only one pair,
                // then completing the pair into a triplet won't decrease shanten
                // (as completing the triplet is offset by losing the pair)
                if num_complete_groups == 4 || (total_groups >= 5 && pair_tile_ids.len() == 1) {
                    continue;
                }

                // if there's an isolated tile of the same value as the pair, completing the pair into a triplet
                // doesn't decrease shanten
                if single_tile_ids.contains(&pair_tile_id) {
                    continue;
                }
            }
            for &tile_id in tile_ids.iter() {
                if !ukiere_tile_ids.contains(&tile_id) {
                    ukiere_tile_ids.push(tile_id);
                }
            }
        }
        ukiere_tile_ids
    }
}

/// takes ownership of tile_count_array (to mutate it)
pub fn get_hand_interpretations(tile_count_array: [u8; 34]) -> Vec<HandInterpretation> {
    let mut original_tile_count_array: [u8; 34] = [0; 34];
    original_tile_count_array.copy_from_slice(&tile_count_array);

    // TODO handle declared melds (which are locked)
    let mut honor_tile_melds: Vec<TileMeld> = Vec::new();

    // start with handling honor tiles: all copies of each honor tile must build one group
    let mut updated_tile_count_array = tile_count_array;
    let mut tile_id = mahjong_tile::FIRST_HONOR_ID;
    while usize::from(tile_id) < tile_count_array.len() {
        let tile_idx = usize::from(tile_id);
        let honor_tile_count = tile_count_array[tile_idx];
        if honor_tile_count == 0 {
            tile_id += 1;
            continue;
        }

        // build a meld with that many copies of the tile id
        let mut meld_tile_ids = Vec::new();
        for _i in 0..honor_tile_count {
            meld_tile_ids.push(tile_id);
        }
        let new_meld = TileMeld::new(meld_tile_ids);
        honor_tile_melds.push(new_meld);
        updated_tile_count_array[tile_idx] = 0;

        tile_id += 1;
    }

    let mut hand_interpretations = Vec::new();
    let meld_interpretations = get_suit_melds(updated_tile_count_array);
    // println!("hand_interpretations:");
    for melds in meld_interpretations {
        // iterate on the Vec directly to consume the vector
        let mut all_melds = honor_tile_melds.clone();
        all_melds.extend(melds);
        let hand_interpretation = HandInterpretation {
            total_tile_count_array: original_tile_count_array,
            groups: all_melds,
        };
        // println!("{}", hand_interpretation);
        hand_interpretations.push(hand_interpretation);
    }

    hand_interpretations
}

#[derive(Clone)]
pub struct PartialMeldInterpretation {
    remaining_tile_count_array: [u8; 34],
    groups: Vec<TileMeld>,
}

impl fmt::Display for PartialMeldInterpretation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tile_groups_string_vec = vec![];
        for tile_group_count_array in self.groups.iter() {
            let mut meld_tile_ids_string = String::new();
            for &tile_id in tile_group_count_array.tile_ids.iter() {
                meld_tile_ids_string
                    .push_str(&mahjong_tile::get_tile_text_from_id(tile_id).unwrap());
            }
            tile_groups_string_vec.push(meld_tile_ids_string);
        }
        let tile_groups_string = tile_groups_string_vec.join(", ");
        write!(
            f,
            "remaining tiles={}, tile_groups=[{}]",
            mahjong_hand::tile_count_array_to_string(&self.remaining_tile_count_array),
            tile_groups_string,
        )
    }
}

impl PartialMeldInterpretation {
    fn best_possible_shanten(&self) -> i8 {
        // treat the already existing groups as locked: without looking at the remaining tiles,
        // what is the best possible shanten we could achieve?
        let num_complete_groups = get_num_complete_groups(&self.groups);
        let num_incomplete_groups = get_num_incomplete_groups(&self.groups);

        // assume that, of the remaining tiles, we can form complete groups
        let remaining_tiles = get_total_tiles_from_count_array(self.remaining_tile_count_array);
        let ideal_remaining_complete_groups = i8::try_from(remaining_tiles / 3).ok().unwrap();
        let ideal_remaining_incomplete_groups = if remaining_tiles % 3 == 2 { 1 } else { 0 };
        let best_possible_shanten = standard_shanten_formula(
            num_complete_groups + ideal_remaining_complete_groups,
            num_incomplete_groups + ideal_remaining_incomplete_groups,
            true, // assume that we can get a pair if we were to draw the right tiles
        );
        best_possible_shanten
    }
}

pub fn get_suit_melds(suit_tile_count_array: [u8; 34]) -> Vec<Vec<TileMeld>> {
    let mut meld_interpretations = Vec::new();
    let mut queue: VecDeque<PartialMeldInterpretation> = VecDeque::new();
    queue.push_front(PartialMeldInterpretation {
        remaining_tile_count_array: suit_tile_count_array,
        groups: Vec::new(),
    });

    while !queue.is_empty() {
        let partial_interpretation = queue.pop_front().unwrap();
        // println!("current state: {}", partial_interpretation);

        let tile_count_array = partial_interpretation.remaining_tile_count_array;

        // find the first tile id that is not empty
        let mut tile_id = 0u8;
        while tile_id < mahjong_tile::FIRST_HONOR_ID {
            let tile_idx = usize::from(tile_id);
            if tile_count_array[tile_idx] != 0 {
                break;
            }
            tile_id += 1;
        }

        let tile_idx = usize::from(tile_id);
        let num_tile_count = tile_count_array[tile_idx];
        if tile_id == mahjong_tile::FIRST_HONOR_ID {
            // if we reached this point, then we have a complete interpretation:
            meld_interpretations.push(partial_interpretation.groups);
            continue;
        }

        if tile_count_array[tile_idx] >= 3 {
            // break out a triplet or a pair
            let mut new_state_after_triplet = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id, tile_id]);
            new_state_after_triplet.remaining_tile_count_array[tile_idx] = num_tile_count - 3;
            new_state_after_triplet.groups.push(tile_meld);
            queue.push_front(new_state_after_triplet);
            // println!(
            //     "will recursively try forming a triplet from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );

            let mut new_state_after_pair = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id]);
            new_state_after_pair.remaining_tile_count_array[tile_idx] = num_tile_count - 2;
            new_state_after_pair.groups.push(tile_meld);
            queue.push_front(new_state_after_pair);
            // println!(
            //     "will recursively try forming a pair from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );

            // however, it's possible that these three tiles are used as multiple sequences / incomplete groups e.g. 666778s can be 678s-67s-6s
        } else if tile_count_array[tile_idx] == 2 {
            // break out a pair and then let it continue trying to add as a single
            let mut new_state_after_pair = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id]);
            new_state_after_pair.remaining_tile_count_array[tile_idx] = num_tile_count - 2;
            new_state_after_pair.groups.push(tile_meld);
            queue.push_front(new_state_after_pair);
            // println!(
            //     "will recursively try forming a pair from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );
        }

        let can_make_sequence = mahjong_hand::can_make_sequence(&tile_count_array, tile_id);
        let can_make_ryanmen = mahjong_hand::can_make_ryanmen(&tile_count_array, tile_id);
        let can_make_penchan = mahjong_hand::can_make_penchan(&tile_count_array, tile_id);
        let can_make_kanchan = mahjong_hand::can_make_kanchan(&tile_count_array, tile_id);

        if can_make_sequence {
            let mut new_state_after_sequence = partial_interpretation.clone();
            // if not iterating through the tile_ids from low to high, these tile ids may not be the correct ones to form the sequence
            let tile_meld = TileMeld::new(vec![tile_id, tile_id + 1, tile_id + 2]);
            new_state_after_sequence.remaining_tile_count_array[tile_idx] =
                tile_count_array[tile_idx] - 1;
            new_state_after_sequence.remaining_tile_count_array[tile_idx + 1] =
                tile_count_array[tile_idx + 1] - 1;
            new_state_after_sequence.remaining_tile_count_array[tile_idx + 2] =
                tile_count_array[tile_idx + 2] - 1;
            new_state_after_sequence.groups.push(tile_meld);
            queue.push_front(new_state_after_sequence);
            // println!(
            //     "will recursively try forming a sequence from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );
        }

        if can_make_ryanmen {
            let mut new_state_after_ryanmen = partial_interpretation.clone();
            // if not iterating through the tile_ids from low to high, these tile ids may not be the correct ones to form the sequence
            let tile_meld = TileMeld::new(vec![tile_id, tile_id + 1]);
            new_state_after_ryanmen.remaining_tile_count_array[tile_idx] =
                tile_count_array[tile_idx] - 1;
            new_state_after_ryanmen.remaining_tile_count_array[tile_idx + 1] =
                tile_count_array[tile_idx + 1] - 1;
            new_state_after_ryanmen.groups.push(tile_meld);
            queue.push_front(new_state_after_ryanmen);
            // println!(
            //     "will recursively try forming a ryanmen from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );
        }

        if can_make_penchan {
            let mut new_state_after_penchan = partial_interpretation.clone();
            // if not iterating through the tile_ids from low to high, these tile ids may not be the correct ones to form the sequence
            let tile_meld = TileMeld::new(vec![tile_id, tile_id + 1]);
            new_state_after_penchan.remaining_tile_count_array[tile_idx] =
                tile_count_array[tile_idx] - 1;
            new_state_after_penchan.remaining_tile_count_array[tile_idx + 1] =
                tile_count_array[tile_idx + 1] - 1;
            new_state_after_penchan.groups.push(tile_meld);
            queue.push_front(new_state_after_penchan);
            // println!(
            //     "will recursively try forming a penchan from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );
        }

        // is it true that we should not try to make a kanchan if there is a possible sequence?
        // e.g. 2344
        if can_make_kanchan && !can_make_sequence {
            let mut new_state_after_kanchan = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id + 2]);
            new_state_after_kanchan.remaining_tile_count_array[tile_idx] =
                tile_count_array[tile_idx] - 1;
            new_state_after_kanchan.remaining_tile_count_array[tile_idx + 2] =
                tile_count_array[tile_idx + 2] - 1;
            new_state_after_kanchan.groups.push(tile_meld);
            queue.push_front(new_state_after_kanchan);
            // println!(
            //     "will recursively try forming a kanchan from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );
        }

        let mut new_state_after_isolated = partial_interpretation.clone();
        let tile_meld = TileMeld::new(vec![tile_id]);
        new_state_after_isolated.remaining_tile_count_array[tile_idx] =
            tile_count_array[tile_idx] - 1;
        new_state_after_isolated.groups.push(tile_meld);
        // how to indicate this is a floating tile vs. a taatsu / protogroup that is only one away
        queue.push_front(new_state_after_isolated);
        // println!(
        //     "will recursively try forming an isolated tile from {}",
        //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
        // );
    }

    meld_interpretations
}

pub fn get_hand_interpretations_min_shanten(
    tile_count_array: [u8; 34],
    shanten_cutoff: i8,
) -> Vec<HandInterpretation> {
    let mut original_tile_count_array: [u8; 34] = [0; 34];
    original_tile_count_array.copy_from_slice(&tile_count_array);

    // TODO handle declared melds (which are locked)
    let mut honor_tile_melds: Vec<TileMeld> = Vec::new();

    // start with handling honor tiles: all copies of each honor tile must build one group
    let mut updated_tile_count_array = tile_count_array;
    let mut tile_id = mahjong_tile::FIRST_HONOR_ID;
    while usize::from(tile_id) < tile_count_array.len() {
        let tile_idx = usize::from(tile_id);
        let honor_tile_count = tile_count_array[tile_idx];
        if honor_tile_count == 0 {
            tile_id += 1;
            continue;
        }

        // build a meld with that many copies of the tile id
        let mut meld_tile_ids = Vec::new();
        for _i in 0..honor_tile_count {
            meld_tile_ids.push(tile_id);
        }
        let new_meld = TileMeld::new(meld_tile_ids);
        honor_tile_melds.push(new_meld);
        updated_tile_count_array[tile_idx] = 0;

        tile_id += 1;
    }

    // construct initial interpretation (all possible interpretations must be based off of this starting point)
    let partial_hand_interpretation = PartialMeldInterpretation {
        remaining_tile_count_array: updated_tile_count_array,
        groups: honor_tile_melds,
    };

    let hand_interpretations = get_suit_melds_min_shanten(
        original_tile_count_array,
        partial_hand_interpretation,
        shanten_cutoff,
    );

    hand_interpretations
}

fn add_to_queue(
    queue: &mut VecDeque<PartialMeldInterpretation>,
    new_state: PartialMeldInterpretation,
) {
    let num_melded_tiles_next_in_queue = queue.get(0).map(|i| get_total_num_tiles(&i.groups));
    // if we've melded more tiles, we want to explore that state first, so place it at the front of the queue
    // (kind of a janky priority queue based on ascending number of tiles remaining)
    if get_total_num_tiles(&new_state.groups) >= num_melded_tiles_next_in_queue.unwrap_or(0) {
        queue.push_front(new_state);
    } else {
        queue.push_back(new_state);
    }
}

fn print_queue(queue: &VecDeque<PartialMeldInterpretation>) {
    println!("current queue state:");
    for interpretation in queue.iter() {
        println!("-> {}", interpretation);
    }
}

pub fn get_suit_melds_min_shanten(
    original_tile_count_array: [u8; 34],
    initial_partial_interpretation: PartialMeldInterpretation,
    shanten_cutoff: i8,
) -> Vec<HandInterpretation> {
    let mut hand_interpretations: HashMap<i8, Vec<HandInterpretation>> = HashMap::new();
    let mut queue: VecDeque<PartialMeldInterpretation> = VecDeque::new();
    queue.push_back(initial_partial_interpretation);

    let mut best_shanten_so_far = shanten_cutoff;
    // println!("best shanten so far: {}", best_shanten_so_far);

    while !queue.is_empty() {
        let partial_interpretation = queue.pop_front().unwrap();
        let best_possible_shanten = partial_interpretation.best_possible_shanten();
        // println!(
        //     "current state: {}, best_possible_shanten: {}",
        //     partial_interpretation, best_possible_shanten
        // );
        if best_possible_shanten > best_shanten_so_far {
            // cannot possibly make a hand with shanten that is at least as good as the best so far,
            // so don't waste your time
            // println!("can't at least get to the best shanten so far");
            continue;
        }

        let tile_count_array = partial_interpretation.remaining_tile_count_array;

        // find the first tile id that is not empty
        // TODO we could be more clever - considering each suit separately, for example
        let mut tile_id = 0u8;
        while tile_id < mahjong_tile::FIRST_HONOR_ID {
            let tile_idx = usize::from(tile_id);
            if tile_count_array[tile_idx] != 0 {
                break;
            }
            tile_id += 1;
        }

        let tile_idx = usize::from(tile_id);
        let num_tile_count = tile_count_array[tile_idx];
        if tile_id == mahjong_tile::FIRST_HONOR_ID {
            // if we reached this point, then we have a complete interpretation:
            let new_hand_interpretation = HandInterpretation {
                total_tile_count_array: original_tile_count_array,
                groups: partial_interpretation.groups,
            };
            // update best shanten so far if we found something promising
            let new_interpretation_standard_shanten =
                new_hand_interpretation.get_standard_shanten();
            if new_interpretation_standard_shanten < best_shanten_so_far {
                best_shanten_so_far = new_interpretation_standard_shanten;
                // println!(
                //     "found new best shanten: {} from interpretation {}",
                //     best_shanten_so_far, new_hand_interpretation
                // );
            }
            if !hand_interpretations.contains_key(&new_interpretation_standard_shanten) {
                hand_interpretations.insert(
                    new_interpretation_standard_shanten,
                    vec![new_hand_interpretation],
                );
            } else {
                let this_shanten_interpretations = hand_interpretations
                    .get_mut(&new_interpretation_standard_shanten)
                    .unwrap();
                (*this_shanten_interpretations).push(new_hand_interpretation);
            }
            continue;
        }

        if tile_count_array[tile_idx] >= 3 {
            // break out a triplet or a pair
            let mut new_state_after_triplet = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id, tile_id]);
            new_state_after_triplet.remaining_tile_count_array[tile_idx] = num_tile_count - 3;
            new_state_after_triplet.groups.push(tile_meld);
            add_to_queue(&mut queue, new_state_after_triplet);
            // println!(
            //     "will recursively try forming a triplet from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );

            let mut new_state_after_pair = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id]);
            new_state_after_pair.remaining_tile_count_array[tile_idx] = num_tile_count - 2;
            new_state_after_pair.groups.push(tile_meld);
            add_to_queue(&mut queue, new_state_after_pair);
            // println!(
            //     "will recursively try forming a pair from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );

            // however, it's possible that these three tiles are used as multiple sequences / incomplete groups e.g. 666778s can be 678s-67s-6s
        } else if tile_count_array[tile_idx] == 2 {
            // break out a pair and then let it continue trying to add as a single
            let mut new_state_after_pair = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id]);
            new_state_after_pair.remaining_tile_count_array[tile_idx] = num_tile_count - 2;
            new_state_after_pair.groups.push(tile_meld);
            add_to_queue(&mut queue, new_state_after_pair);
            // println!(
            //     "will recursively try forming a pair from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );
        }

        let can_make_sequence = mahjong_hand::can_make_sequence(&tile_count_array, tile_id);
        let can_make_ryanmen = mahjong_hand::can_make_ryanmen(&tile_count_array, tile_id);
        let can_make_penchan = mahjong_hand::can_make_penchan(&tile_count_array, tile_id);
        let can_make_kanchan = mahjong_hand::can_make_kanchan(&tile_count_array, tile_id);

        if can_make_sequence {
            let mut new_state_after_sequence = partial_interpretation.clone();
            // if not iterating through the tile_ids from low to high, these tile ids may not be the correct ones to form the sequence
            let tile_meld = TileMeld::new(vec![tile_id, tile_id + 1, tile_id + 2]);
            new_state_after_sequence.remaining_tile_count_array[tile_idx] =
                tile_count_array[tile_idx] - 1;
            new_state_after_sequence.remaining_tile_count_array[tile_idx + 1] =
                tile_count_array[tile_idx + 1] - 1;
            new_state_after_sequence.remaining_tile_count_array[tile_idx + 2] =
                tile_count_array[tile_idx + 2] - 1;
            new_state_after_sequence.groups.push(tile_meld);
            add_to_queue(&mut queue, new_state_after_sequence);
            // println!(
            //     "will recursively try forming a sequence from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );
        }

        if can_make_ryanmen {
            let mut new_state_after_ryanmen = partial_interpretation.clone();
            // if not iterating through the tile_ids from low to high, these tile ids may not be the correct ones to form the sequence
            let tile_meld = TileMeld::new(vec![tile_id, tile_id + 1]);
            new_state_after_ryanmen.remaining_tile_count_array[tile_idx] =
                tile_count_array[tile_idx] - 1;
            new_state_after_ryanmen.remaining_tile_count_array[tile_idx + 1] =
                tile_count_array[tile_idx + 1] - 1;
            new_state_after_ryanmen.groups.push(tile_meld);
            add_to_queue(&mut queue, new_state_after_ryanmen);
            // println!(
            //     "will recursively try forming a ryanmen from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );
        }

        if can_make_penchan {
            let mut new_state_after_penchan = partial_interpretation.clone();
            // if not iterating through the tile_ids from low to high, these tile ids may not be the correct ones to form the sequence
            let tile_meld = TileMeld::new(vec![tile_id, tile_id + 1]);
            new_state_after_penchan.remaining_tile_count_array[tile_idx] =
                tile_count_array[tile_idx] - 1;
            new_state_after_penchan.remaining_tile_count_array[tile_idx + 1] =
                tile_count_array[tile_idx + 1] - 1;
            new_state_after_penchan.groups.push(tile_meld);
            add_to_queue(&mut queue, new_state_after_penchan);
            // println!(
            //     "will recursively try forming a penchan from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );
        }

        // is it true that we should not try to make a kanchan if there is a possible sequence?
        // e.g. 2344
        if can_make_kanchan && !can_make_sequence {
            let mut new_state_after_kanchan = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id + 2]);
            new_state_after_kanchan.remaining_tile_count_array[tile_idx] =
                tile_count_array[tile_idx] - 1;
            new_state_after_kanchan.remaining_tile_count_array[tile_idx + 2] =
                tile_count_array[tile_idx + 2] - 1;
            new_state_after_kanchan.groups.push(tile_meld);
            add_to_queue(&mut queue, new_state_after_kanchan);
            // println!(
            //     "will recursively try forming a kanchan from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );
        }

        let mut new_state_after_isolated = partial_interpretation.clone();
        let tile_meld = TileMeld::new(vec![tile_id]);
        new_state_after_isolated.remaining_tile_count_array[tile_idx] =
            tile_count_array[tile_idx] - 1;
        new_state_after_isolated.groups.push(tile_meld);
        // how to indicate this is a floating tile vs. a taatsu / protogroup that is only one away
        add_to_queue(&mut queue, new_state_after_isolated);
        // println!(
        //     "will recursively try forming an isolated tile from {}",
        //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
        // );
        // print_queue(&queue);
    }

    // println!(
    //     "getting hand interpretations for shanten {}",
    //     best_shanten_so_far
    // );
    let flattened_hand_interpretations = hand_interpretations.get(&best_shanten_so_far);
    match flattened_hand_interpretations {
        Some(interpretations) => interpretations.clone(),
        None => Vec::new(),
    }
}

pub fn get_shanten(tile_count_array: [u8; 34]) -> i8 {
    let chiitoi_shanten = get_chiitoi_shanten(tile_count_array);
    let kokushi_shanten = get_kokushi_shanten(tile_count_array);
    let mut shanten = min(chiitoi_shanten, kokushi_shanten);

    let interpretations = get_hand_interpretations(tile_count_array);
    let standard_shanten = get_shanten_helper(&interpretations);
    if standard_shanten < shanten {
        shanten = standard_shanten;
    }
    shanten
}

pub fn get_shanten_optimized(tile_count_array: [u8; 34]) -> i8 {
    let chiitoi_shanten = get_chiitoi_shanten(tile_count_array);
    let kokushi_shanten = get_kokushi_shanten(tile_count_array);
    let mut shanten = min(chiitoi_shanten, kokushi_shanten);

    let interpretations = get_hand_interpretations_min_shanten(tile_count_array, shanten);
    // println!("hand interpretations:");
    // for hand_interpretation in interpretations.iter() {
    //     println!("{}", hand_interpretation);
    // }
    let standard_shanten = get_shanten_helper(&interpretations);
    if standard_shanten < shanten {
        shanten = standard_shanten;
    }
    shanten
}

/// Gets the possible shanten for each possible discard from the given set of tiles
pub fn get_shanten_after_each_discard(
    tile_count_array: [u8; 34],
    shanten_function: &dyn Fn([u8; 34]) -> i8,
) -> HashMap<i8, Vec<MahjongTileId>> {
    if get_total_tiles_from_count_array(tile_count_array) != 14 {
        // TODO eventually will need to handle the case when there are more tiles due to quads
        panic!("invalid number of tiles")
    }
    // naive implementation: just try every possible discard -> call `get_shanten` -> pick the best overall
    // but is there a more clever way?
    let mut tile_id = 0u8;
    let mut shanten_by_discard_tile_id = HashMap::new();
    while usize::from(tile_id) < tile_count_array.len() {
        let tile_idx = usize::from(tile_id);
        if tile_count_array[tile_idx] > 0 {
            let mut tile_count_after_discard = tile_count_array;
            tile_count_after_discard[tile_idx] -= 1;
            let shanten_after_discard = shanten_function(tile_count_after_discard);
            if !shanten_by_discard_tile_id.contains_key(&shanten_after_discard) {
                shanten_by_discard_tile_id.insert(shanten_after_discard, vec![MahjongTileId(tile_id)]);
            } else {
                let tile_ids_for_shanten = shanten_by_discard_tile_id
                    .get_mut(&shanten_after_discard)
                    .unwrap();
                tile_ids_for_shanten.push(MahjongTileId(tile_id));
            }
        }
        tile_id += 1;
    }
    // println!(
    //     "shanten after each discard: {:?}",
    //     shanten_by_discard_tile_id
    //         .iter()
    //         .map(|(shanten, tile_ids)| (*shanten, tile_ids_to_string(tile_ids)))
    //         .collect::<HashMap<i8, String>>()
    // );
    shanten_by_discard_tile_id
}

/// Gets the possible shanten after discarding any tile from the given set of tiles
pub fn get_best_shanten_after_discard(
    tile_count_array: [u8; 34],
    shanten_function: &dyn Fn([u8; 34]) -> i8,
) -> i8 {
    if get_total_tiles_from_count_array(tile_count_array) != 14 {
        // TODO eventually will need to handle the case when there are more tiles due to quads
        panic!("invalid number of tiles")
    }
    let shanten_by_discard_tile_id =
        get_shanten_after_each_discard(tile_count_array, shanten_function);
    let mut best_shanten = 99i8;
    for shanten in shanten_by_discard_tile_id.keys() {
        if *shanten < best_shanten {
            best_shanten = *shanten;
        }
    }
    best_shanten
}

/// returns all discard options (discard tile id, ukiere tile ids, number of ukiere tiles) with the best shanten
/// and that maximizes the count of ukiere
pub fn get_most_ukiere_after_discard<T: Into<MahjongTileId> + Clone>(
    tile_count_array: [u8; 34],
    best_shanten: i8,
    shanten_function: &dyn Fn([u8; 34]) -> i8,
    ukiere_function: &dyn Fn([u8; 34]) -> Vec<MahjongTileId>,
    other_visible_tiles: &Vec<T>,
) -> Vec<(MahjongTileId, Vec<MahjongTileId>, u16)> {
    if get_total_tiles_from_count_array(tile_count_array) != 14 {
        // TODO eventually will need to handle the case when there are more tiles due to quads
        panic!("invalid number of tiles")
    }
    let shanten_by_discard_tile_id =
        get_shanten_after_each_discard(tile_count_array, shanten_function);
    let best_shanten_discard_tile_ids = shanten_by_discard_tile_id.get(&best_shanten).unwrap();

    let mut ukiere_discard_options = Vec::new();
    let mut highest_ukiere_count_so_far = 0;
    for discard_tile_id in best_shanten_discard_tile_ids {
        let ukiere_tiles_after_discard =
            get_ukiere_after_discard(tile_count_array, *discard_tile_id, ukiere_function);

        let new_count_array = remove_tile_id_from_count_array(tile_count_array, *discard_tile_id);
        let mut visible_tiles_after_discard =
            combine_tile_ids_from_count_array_and_vec(new_count_array, other_visible_tiles);
        visible_tiles_after_discard.push(*discard_tile_id);

        let ukiere_count =
            get_num_tiles_remaining(&ukiere_tiles_after_discard, &visible_tiles_after_discard);
        let new_entry = (*discard_tile_id, ukiere_tiles_after_discard, ukiere_count);
        if ukiere_count > highest_ukiere_count_so_far {
            highest_ukiere_count_so_far = ukiere_count;
            ukiere_discard_options = vec![new_entry];
        } else if ukiere_count == highest_ukiere_count_so_far {
            ukiere_discard_options.push(new_entry);
        }
    }

    ukiere_discard_options
}

/// returns all discard options (discard tile id, ukiere tile ids, number of ukiere tiles) with the best shanten
/// regardless of the ukiere
pub fn get_all_ukiere_after_discard<T: Into<MahjongTileId> + Clone>(
    tile_count_array: [u8; 34],
    best_shanten: i8,
    shanten_function: &dyn Fn([u8; 34]) -> i8,
    ukiere_function: &dyn Fn([u8; 34]) -> Vec<MahjongTileId>,
    other_visible_tiles: &Vec<T>,
) -> Vec<(MahjongTileId, Vec<MahjongTileId>, u16)> {
    if get_total_tiles_from_count_array(tile_count_array) != 14 {
        // TODO eventually will need to handle the case when there are more tiles due to quads
        panic!("invalid number of tiles")
    }
    let shanten_by_discard_tile_id =
        get_shanten_after_each_discard(tile_count_array, shanten_function);
    let best_shanten_discard_tile_ids = shanten_by_discard_tile_id.get(&best_shanten).unwrap();

    let mut ukiere_discard_options = Vec::new();
    for discard_tile_id in best_shanten_discard_tile_ids {
        let ukiere_tiles_after_discard =
            get_ukiere_after_discard(tile_count_array, *discard_tile_id, ukiere_function);

        let new_count_array = remove_tile_id_from_count_array(tile_count_array, *discard_tile_id);
        let mut visible_tiles_after_discard =
            combine_tile_ids_from_count_array_and_vec(new_count_array, other_visible_tiles);
        visible_tiles_after_discard.push(*discard_tile_id);

        let ukiere_count =
            get_num_tiles_remaining(&ukiere_tiles_after_discard, &visible_tiles_after_discard);
        let new_entry = (*discard_tile_id, ukiere_tiles_after_discard, ukiere_count);
        ukiere_discard_options.push(new_entry);
    }
    ukiere_discard_options
}

pub fn get_shanten_ukiere_after_each_discard<T: Into<MahjongTileId> + Clone>(
    tile_count_array: [u8; 34],
    shanten_function: &dyn Fn([u8; 34]) -> i8,
    ukiere_function: &dyn Fn([u8; 34]) -> Vec<MahjongTileId>,
    other_visible_tiles: &Vec<T>,
) -> Vec<(MahjongTileId, i8, Vec<MahjongTileId>, u16)> {
    if get_total_tiles_from_count_array(tile_count_array) != 14 {
        // TODO eventually will need to handle the case when there are more tiles due to quads
        panic!("invalid number of tiles")
    }
    let shanten_by_discard_tile_id =
        get_shanten_after_each_discard(tile_count_array, shanten_function);

    let mut ukiere_discard_options = Vec::new();
    for (shanten, discard_tile_ids) in shanten_by_discard_tile_id {
        for discard_tile_id in discard_tile_ids {
            let ukiere_tiles_after_discard =
                get_ukiere_after_discard(tile_count_array, discard_tile_id, ukiere_function);

            let new_count_array =
                remove_tile_id_from_count_array(tile_count_array, discard_tile_id);
            let mut visible_tiles_after_discard =
                combine_tile_ids_from_count_array_and_vec(new_count_array, other_visible_tiles);
            visible_tiles_after_discard.push(discard_tile_id);

            let ukiere_count =
                get_num_tiles_remaining(&ukiere_tiles_after_discard, &visible_tiles_after_discard);
            let new_entry = (
                discard_tile_id,
                shanten,
                ukiere_tiles_after_discard,
                ukiere_count,
            );
            ukiere_discard_options.push(new_entry);
        }
    }

    ukiere_discard_options
}

/// utility function: useful for combining tile ids from a hand (in the form of a tile count array) and a vector of other visible tile ids (e.g. discard pools, dora indicator, visible melds from opponents, etc.)
pub fn combine_tile_ids_from_count_array_and_vec<T: Into<MahjongTileId> + Clone>(
    tile_count_array: [u8; 34],
    tile_ids: &Vec<T>,
) -> Vec<MahjongTileId> {
    let mut new_tile_ids: Vec<MahjongTileId> = tile_ids.iter().cloned().map(|t| t.into()).collect();
    let mut hand_tile_ids = tile_count_array_to_tile_ids(tile_count_array);
    new_tile_ids.append(&mut hand_tile_ids);
    new_tile_ids
}

/// returns a map of (upgrade tile to draw -> map of (tile to discard, resulting ukiere tile ids))
pub fn get_upgrade_tiles<T: Into<MahjongTileId> + Clone>(
    tile_count_array: [u8; 34],
    shanten_function: &dyn Fn([u8; 34]) -> i8,
    ukiere_function: &dyn Fn([u8; 34]) -> Vec<MahjongTileId>,
    other_visible_tiles: &Vec<T>,
) -> HashMap<MahjongTileId, HashMap<MahjongTileId, (Vec<MahjongTileId>, u16)>> {
    if get_total_tiles_from_count_array(tile_count_array) != 13 {
        // TODO eventually will need to handle the case when there are more tiles due to quads
        panic!("invalid number of tiles")
    }
    let mut upgrades = HashMap::new();
    let starting_shanten = shanten_function(tile_count_array);
    let starting_ukiere_tiles = ukiere_function(tile_count_array);
    let visible_tile_ids =
        combine_tile_ids_from_count_array_and_vec(tile_count_array, other_visible_tiles);
    let starting_num_ukiere_tiles =
        get_num_tiles_remaining(&starting_ukiere_tiles, &visible_tile_ids);

    for draw_tile_id in 0..mahjong_tile::NUM_DISTINCT_TILE_VALUES {
        if starting_ukiere_tiles.contains(&Into::<MahjongTileId>::into(draw_tile_id)) {
            // drawing a ukiere tile is not an upgrade (upgrade = same shanten, but)
            continue;
        }
        let new_count_array = add_tile_id_to_count_array(tile_count_array, draw_tile_id);
        let new_tiles = get_distinct_tile_ids_from_count_array(new_count_array);
        let visible_tile_ids =
            combine_tile_ids_from_count_array_and_vec(new_count_array, other_visible_tiles);

        let mut discard_to_ukiere = HashMap::new();
        for discard_tile_id in new_tiles {
            let tile_count_array_after_discard =
                remove_tile_id_from_count_array(new_count_array, discard_tile_id);
            let shanten_after_discard = shanten_function(tile_count_array_after_discard);
            if shanten_after_discard > starting_shanten {
                // increasing shanten -> not an upgrade
                continue;
            } else if shanten_after_discard < starting_shanten {
                panic!("shanten after discard is {} (lower than starting shanten {}), but that means this is an ukiere tile, not an upgrade tile", shanten_after_discard, starting_shanten);
            }

            let ukiere_after_discard =
                get_ukiere_after_discard(new_count_array, discard_tile_id, ukiere_function);
            let num_ukiere_tiles_after_discard =
                get_num_tiles_remaining(&ukiere_after_discard, &visible_tile_ids);
            if num_ukiere_tiles_after_discard > starting_num_ukiere_tiles {
                // only an upgrade if the total number of ukiere tiles remaining increases
                discard_to_ukiere.insert(
                    discard_tile_id,
                    (ukiere_after_discard, num_ukiere_tiles_after_discard),
                );
            }
        }
        if !discard_to_ukiere.is_empty() {
            upgrades.insert(MahjongTileId(draw_tile_id), discard_to_ukiere);
        }
    }
    upgrades
}

pub fn add_tile_id_to_count_array<T: Into<MahjongTileId>>(tile_count_array: [u8; 34], new_tile_id: T) -> [u8; 34] {
    let new_tile_id: MahjongTileId = new_tile_id.into();
    assert!(
        usize::from(new_tile_id) < tile_count_array.len(),
        "invalid tile id"
    );
    let mut new_count_array = [0; 34];
    for tile_id in 0..tile_count_array.len() {
        new_count_array[tile_id] = tile_count_array[tile_id];
    }
    let new_tile_idx = usize::from(new_tile_id);
    new_count_array[new_tile_idx] += 1;
    new_count_array
}

pub fn remove_tile_id_from_count_array<T: Into<MahjongTileId>>(
    tile_count_array: [u8; 34],
    discard_tile_id: T,
) -> [u8; 34] {
    let discard_tile_id: MahjongTileId = discard_tile_id.into();
    assert!(
        usize::from(discard_tile_id) < tile_count_array.len(),
        "invalid tile id"
    );
    let mut new_count_array = [0; 34];
    for tile_id in 0..tile_count_array.len() {
        new_count_array[tile_id] = tile_count_array[tile_id];
    }
    let discard_tile_idx = usize::from(discard_tile_id);
    if new_count_array[discard_tile_idx] > 0 {
        new_count_array[discard_tile_idx] -= 1;
    }
    new_count_array
}

pub fn get_ukiere_after_discard<T: Into<MahjongTileId>>(
    tile_count_array: [u8; 34],
    discard_tile_id: T,
    ukiere_function: &dyn Fn([u8; 34]) -> Vec<MahjongTileId>,
) -> Vec<MahjongTileId> {
    if get_total_tiles_from_count_array(tile_count_array) != 14 {
        // TODO eventually will need to handle the case when there are more tiles due to quads
        panic!("invalid number of tiles")
    }
    let new_count_array = remove_tile_id_from_count_array(tile_count_array, discard_tile_id);
    ukiere_function(new_count_array)
}

pub fn get_num_tiles_remaining<T>(target_tile_ids: &Vec<T>, visible_tile_ids: &Vec<T>) -> u16
where
    T: Into<MahjongTileId> + Clone + PartialEq,
{
    let mut base_tile_count = [4u16; 34];
    let mut remaining_tile_count: u16 = (4 * target_tile_ids.len()).try_into().unwrap();
    for tile_id in visible_tile_ids {
        if !target_tile_ids.contains(tile_id) {
            continue;
        }
        let mahjong_tile_id: MahjongTileId = tile_id.clone().into();
        let tile_idx = usize::from(mahjong_tile_id);
        if base_tile_count[tile_idx] > 0 && remaining_tile_count > 0 {
            base_tile_count[tile_idx] -= 1;
            remaining_tile_count -= 1;
        } else {
            panic!(
                "tried to remove too many copies of tile {} (tile_id {})",
                mahjong_tile::get_tile_text_from_id(mahjong_tile_id).unwrap(),
                mahjong_tile_id
            );
        }
    }
    remaining_tile_count
}

pub fn get_shanten_helper(hand_interpretations: &Vec<HandInterpretation>) -> i8 {
    hand_interpretations
        .iter()
        .map(|i| i.get_standard_shanten())
        .min()
        .unwrap_or(8) // worst possible shanten by standard shape is 8-shanten
}

fn generate_ukiere_tiles(
    tile_count_array: [u8; 34],
    standard_shanten: i8,
    chiitoi_shanten: i8,
    kokushi_shanten: i8,
    hand_interpretations: &Vec<HandInterpretation>,
) -> Vec<MahjongTileId> {
    let special_shanten = min(chiitoi_shanten, kokushi_shanten);
    let min_shanten = min(special_shanten, standard_shanten);

    let mut ukiere_tile_ids = Vec::new();
    if standard_shanten == min_shanten {
        let standard_ukiere_tiles = get_ukiere_helper(&hand_interpretations, standard_shanten);
        for tile_id in standard_ukiere_tiles {
            if !ukiere_tile_ids.contains(&tile_id) {
                ukiere_tile_ids.push(tile_id);
            }
        }
    }
    if chiitoi_shanten == min_shanten {
        let chiitoi_ukiere_tiles = get_chiitoi_ukiere(tile_count_array);
        for tile_id in chiitoi_ukiere_tiles {
            if !ukiere_tile_ids.contains(&tile_id) {
                ukiere_tile_ids.push(tile_id);
            }
        }
    }
    if kokushi_shanten == min_shanten {
        let kokushi_ukiere_tiles = get_kokushi_ukiere(tile_count_array);
        for tile_id in kokushi_ukiere_tiles {
            if !ukiere_tile_ids.contains(&tile_id) {
                ukiere_tile_ids.push(tile_id);
            }
        }
    }
    ukiere_tile_ids
}

pub fn get_ukiere(tile_count_array: [u8; 34]) -> Vec<MahjongTileId> {
    let interpretations = get_hand_interpretations(tile_count_array);
    let standard_shanten = get_shanten_helper(&interpretations);
    let chiitoi_shanten = get_chiitoi_shanten(tile_count_array);
    let kokushi_shanten = get_kokushi_shanten(tile_count_array);

    generate_ukiere_tiles(
        tile_count_array,
        standard_shanten,
        chiitoi_shanten,
        kokushi_shanten,
        &interpretations,
    )
}

pub fn get_ukiere_optimized(tile_count_array: [u8; 34]) -> Vec<MahjongTileId> {
    let chiitoi_shanten = get_chiitoi_shanten(tile_count_array);
    let kokushi_shanten = get_kokushi_shanten(tile_count_array);
    let special_shanten = min(chiitoi_shanten, kokushi_shanten);

    let interpretations = get_hand_interpretations_min_shanten(tile_count_array, special_shanten);
    let standard_shanten = get_shanten_helper(&interpretations);
    generate_ukiere_tiles(
        tile_count_array,
        standard_shanten,
        chiitoi_shanten,
        kokushi_shanten,
        &interpretations,
    )
}

pub fn get_ukiere_helper(hand_interpretations: &Vec<HandInterpretation>, shanten: i8) -> Vec<MahjongTileId> {
    let mut ukiere_tiles = Vec::new();
    // println!("looking for hand interpretations with shanten {}", shanten);
    for interpretation in hand_interpretations.iter() {
        if interpretation.get_standard_shanten() != shanten {
            continue;
        }

        // println!(
        //     "finding ukiere tiles for hand interpretation {} with shanten {}",
        //     interpretation, shanten
        // );
        let new_tile_ids = interpretation.get_ukiere();
        for &tile_id in new_tile_ids.iter() {
            // print!(
            //     "ukiere tile: {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );
            if !ukiere_tiles.contains(&tile_id) {
                ukiere_tiles.push(tile_id);
            }
            // print!("\n");
        }
    }
    ukiere_tiles
}

pub fn get_chiitoi_shanten(tile_count_array: [u8; 34]) -> i8 {
    let mut tile_id = 0u8;
    let mut num_pairs = 0;
    while usize::from(tile_id) < tile_count_array.len() {
        let tile_idx = usize::from(tile_id);
        if tile_count_array[tile_idx] >= 2 {
            num_pairs += 1;
        }
        tile_id += 1;
    }
    6 - num_pairs
}

pub fn get_chiitoi_ukiere(tile_count_array: [u8; 34]) -> Vec<MahjongTileId> {
    let mut tile_id = 0u8;
    let mut ukiere_tile_ids = Vec::new();
    while usize::from(tile_id) < tile_count_array.len() {
        let tile_idx = usize::from(tile_id);
        if tile_count_array[tile_idx] == 1 {
            ukiere_tile_ids.push(MahjongTileId(tile_id));
        }
        tile_id += 1;
    }
    ukiere_tile_ids
}

pub fn get_kokushi_shanten(tile_count_array: [u8; 34]) -> i8 {
    let kokushi_tile_ids = mahjong_tile::tiles_to_tile_ids("19m19p19s1234567z");
    let mut num_kokushi_tiles = 0;
    let mut has_kokushi_pair = false;
    for kokushi_tile_id in kokushi_tile_ids.iter() {
        let kokushi_tile_idx = usize::from(*kokushi_tile_id);
        let kokushi_tile_count = tile_count_array[kokushi_tile_idx];
        if kokushi_tile_count >= 1 {
            num_kokushi_tiles += 1;
        }
        if kokushi_tile_count >= 2 {
            has_kokushi_pair = true;
        }
    }
    let mut kokushi_shanten = 13 - num_kokushi_tiles;
    if has_kokushi_pair {
        kokushi_shanten -= 1;
    }
    kokushi_shanten
}

pub fn get_kokushi_ukiere(tile_count_array: [u8; 34]) -> Vec<MahjongTileId> {
    let mut missing_kokushi_tiles = Vec::new();
    let kokushi_tile_ids = mahjong_tile::tiles_to_tile_ids("19m19p19s1234567z");
    let mut has_kokushi_pair = false;
    for kokushi_tile_id in kokushi_tile_ids.iter() {
        let kokushi_tile_idx = usize::from(*kokushi_tile_id);
        let kokushi_tile_count = tile_count_array[kokushi_tile_idx];
        if kokushi_tile_count == 0 {
            missing_kokushi_tiles.push(*kokushi_tile_id);
        } else if kokushi_tile_count >= 2 {
            has_kokushi_pair = true;
        }
    }

    if !has_kokushi_pair {
        // can improve on any kokushi tile: it will either form a pair or be a new kokushi that wasn't in the hand already
        kokushi_tile_ids
    } else {
        missing_kokushi_tiles
    }
}

pub fn tiles_to_count_array(tiles_string: &str) -> [u8; 34] {
    let mut tile_count_array: [u8; 34] = [0; 34];
    let mut rank_chars: Vec<char> = Vec::new();
    for char in tiles_string.chars() {
        if char == 'm' || char == 's' || char == 'p' || char == 'z' {
            if rank_chars.is_empty() {
                panic!("expected some numbers/ranks to come before the suit character")
            }
            for rank_char in rank_chars {
                let mut tile_string = String::new();
                // println!("found tile {}{}", rank_char, char);
                tile_string.push(rank_char);
                tile_string.push(char);
                let tile_id = mahjong_tile::get_id_from_tile_text(&tile_string).unwrap();
                tile_count_array[usize::from(tile_id)] += 1;
            }
            rank_chars = Vec::new();
        } else {
            rank_chars.push(char);
        }
    }
    tile_count_array
}

fn tile_count_array_to_tile_ids(tile_count_array: [u8; 34]) -> Vec<MahjongTileId> {
    let mut tile_ids = Vec::new();
    for tile_id in 0..mahjong_tile::NUM_DISTINCT_TILE_VALUES {
        let tile_idx = usize::from(tile_id);
        if tile_count_array[tile_idx] == 0 {
            continue;
        }
        for _i in 0..tile_count_array[tile_idx] {
            tile_ids.push(MahjongTileId(tile_id));
        }
    }
    tile_ids
}

pub fn tile_ids_to_string<T: Into<MahjongTileId> + Clone>(tile_ids: &Vec<T>) -> String {
    let mut tiles_string = String::new();
    for tile_id in tile_ids.iter() {
        let tile_id: MahjongTileId = tile_id.clone().into();
        let tile_string = mahjong_tile::get_tile_text_from_id(tile_id).unwrap();
        tiles_string.push_str(&tile_string);
    }
    tiles_string
}

// TODO figure out how to use generics for this
pub fn print_ukiere_after_discard(options_after_discard: &Vec<(MahjongTileId, Vec<MahjongTileId>, u16)>) {
    let mut options = options_after_discard.clone();
    options
        .iter_mut()
        .for_each(|(_, ukiere_after_upgrade_discard, _)| {
            ukiere_after_upgrade_discard.sort();
        });

    // sort discard options (after drawing improvement tile) by descending number of ukiere tiles
    // let mut options_after_improve_sorted = options.clone();
    options.sort_by(
        |(_, _, num_ukiere_tiles_after_improve1), (_, _, num_ukiere_tiles_after_improve2)| {
            num_ukiere_tiles_after_improve2.cmp(num_ukiere_tiles_after_improve1)
        },
    );

    let options_str: Vec<String> = options
        .into_iter()
        .map(
            |(discard_tile_id, ukiere_tile_ids_after_discard, num_ukiere_tiles_after_discard)| {
                format!(
                    "cut {} => {} ukiere: {}",
                    mahjong_tile::get_tile_text_from_id(discard_tile_id.clone()).unwrap(),
                    num_ukiere_tiles_after_discard,
                    tile_ids_to_string(&ukiere_tile_ids_after_discard)
                )
            },
        )
        .collect();
    println!("{}", options_str.join("; "));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;
    use std::collections::HashSet;
    use test::Bencher;

    macro_rules! hashmap {
        ($( $key: expr => $val: expr ),*) => {{
            let mut map = ::std::collections::HashMap::new();
            $( map.insert($key, $val); )*
            map
        }}
    }

    fn assert_tile_ids_match<T: Into<MahjongTileId> + Clone>(tile_ids: &Vec<T>, expected_tile_ids: &Vec<T>) {
        // assert_eq!(tile_ids.len(), expected_tile_ids.len());
        let mut sorted_tile_ids: Vec<MahjongTileId> = tile_ids.iter().cloned().map(|t| t.into()).collect();
        sorted_tile_ids.sort();
        let mut sorted_expected_tile_ids: Vec<MahjongTileId> = expected_tile_ids.iter().cloned().map(|t| t.into()).collect();
        sorted_expected_tile_ids.sort();
        assert_eq!(
            sorted_tile_ids,
            sorted_expected_tile_ids,
            "got {} but expected {}",
            tile_ids_to_string(&sorted_tile_ids),
            tile_ids_to_string(&sorted_expected_tile_ids)
        );
    }

    fn assert_ukiere_tiles_after_discard_match<T: Into<MahjongTileId> + Clone>(
        tile_count_array: [u8; 34],
        expected_ukiere_after_discard: &HashMap<&'static str, Vec<T>>,
    ) {
        for (discard_tile_str, expected_ukiere_tiles) in expected_ukiere_after_discard {
            let ukiere_tiles = get_ukiere_after_discard(
                tile_count_array,
                mahjong_tile::get_id_from_tile_text(discard_tile_str).unwrap(),
                &get_ukiere,
            );
            let expected_ukiere_tiles: Vec<MahjongTileId> = expected_ukiere_tiles.iter().cloned().map(|t| t.into()).collect();
            assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

            let ukiere_tiles = get_ukiere_after_discard(
                tile_count_array,
                mahjong_tile::get_id_from_tile_text(discard_tile_str).unwrap(),
                &get_ukiere_optimized,
            );
            assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
        }
    }

    fn assert_upgrade_tiles_match<T: Into<MahjongTileId> + Clone>(
        tile_count_array: [u8; 34],
        expected_upgrades: &HashMap<&'static str, HashMap<&'static str, Vec<T>>>,
        other_visible_tiles: &Vec<T>,
    ) {
        // println!(
        //     "asserting upgrade tiles for {}",
        //     tile_ids_to_string(&tile_count_array_to_tile_ids(tile_count_array))
        // );
        if get_total_tiles_from_count_array(tile_count_array) != 13 {
            // TODO eventually will need to handle the case when there are more tiles due to quads
            panic!("invalid number of tiles");
        }

        for (draw_tile_str, discard_tile_str_to_ukiere_tiles) in expected_upgrades {
            let new_count_array = add_tile_id_to_count_array(
                tile_count_array,
                mahjong_tile::get_id_from_tile_text(&draw_tile_str).unwrap(),
            );
            // println!("considering ukiere tiles after discard for {}", tile_ids_to_string(&tile_count_array_to_tile_ids(new_count_array)));

            for (discard_tile_str, new_ukiere_tiles) in discard_tile_str_to_ukiere_tiles {
                let ukiere_tiles = get_ukiere_after_discard(
                    new_count_array,
                    mahjong_tile::get_id_from_tile_text(*discard_tile_str).unwrap(),
                    &get_ukiere,
                );
                let new_ukiere_tiles: Vec<MahjongTileId> = new_ukiere_tiles.iter().cloned().map(|t| t.into()).collect();
                assert_tile_ids_match(&ukiere_tiles, &new_ukiere_tiles);

                let ukiere_tiles = get_ukiere_after_discard(
                    new_count_array,
                    mahjong_tile::get_id_from_tile_text(*discard_tile_str).unwrap(),
                    &get_ukiere_optimized,
                );
                assert_tile_ids_match(&ukiere_tiles, &new_ukiere_tiles);
            }

            // check that the discard tile ids (after upgrade draw) mapping in `expected_upgrades` are correct:
            // we shouldn't have missed anything!
            let best_shanten =
                get_best_shanten_after_discard(new_count_array, &get_shanten_optimized);
            let best_ukiere_discards = get_most_ukiere_after_discard(
                new_count_array,
                best_shanten,
                &get_shanten_optimized,
                &get_ukiere_optimized,
                &other_visible_tiles,
            );

            let discard_tiles_after_upgrade_draw: HashSet<String> =
                HashSet::from_iter(best_ukiere_discards.into_iter().map(
                    |(discard_tile_id, _, _)| {
                        mahjong_tile::get_tile_text_from_id(discard_tile_id).unwrap()
                    },
                ));
            let expected_best_discard_tiles_after_upgrade_draw: HashSet<String> =
                HashSet::from_iter((*discard_tile_str_to_ukiere_tiles).keys().map(|&s| {
                    let mut new_str = String::new();
                    new_str.push_str(s);
                    new_str
                }));
            assert_eq!(
                discard_tiles_after_upgrade_draw,
                expected_best_discard_tiles_after_upgrade_draw,
                "expected {:?} to be the best discard(s) (by ukiere tile count) after an upgrade {}, but instead got the following tiles as best discard(s): {:?}",
                expected_best_discard_tiles_after_upgrade_draw,
                draw_tile_str,
                discard_tiles_after_upgrade_draw
            );
        }

        // println!(
        //     "getting shanten for {}",
        //     tile_ids_to_string(&tile_count_array_to_tile_ids(tile_count_array))
        // );
        let shanten = get_shanten_optimized(tile_count_array);
        // println!(
        //     "getting ukiere for {}",
        //     tile_ids_to_string(&tile_count_array_to_tile_ids(tile_count_array))
        // );
        let ukiere_tile_ids = get_ukiere_optimized(tile_count_array);

        let visible_tile_ids =
            combine_tile_ids_from_count_array_and_vec(tile_count_array, other_visible_tiles);
        let num_ukiere = get_num_tiles_remaining(&ukiere_tile_ids, &visible_tile_ids);

        // check non-upgrade & non-ukiere tiles: drawing these should not change the max possible ukiere
        // -> the max num of ukiere of the discard options should be the same as the num ukiere before drawing the non-upgrade tile
        for potential_tile_id in 0..mahjong_tile::NUM_DISTINCT_TILE_VALUES {
            let potential_tile_string =
                mahjong_tile::get_tile_text_from_id(potential_tile_id).unwrap();
            let potential_tile_str = potential_tile_string.as_str();
            if expected_upgrades.contains_key(&potential_tile_str)
                || ukiere_tile_ids.contains(&MahjongTileId(potential_tile_id))
            {
                continue;
            }
            let new_count_array = add_tile_id_to_count_array(tile_count_array, potential_tile_id);

            println!(
                "checking ukiere after draw {} -> discard for {}",
                potential_tile_string,
                tile_ids_to_string(&tile_count_array_to_tile_ids(new_count_array))
            );
            // check that the best discard from this new hand is the same tile that was just added - and we don't get any more ukiere
            let ukiere_after_nonupgrade = get_most_ukiere_after_discard(
                new_count_array,
                shanten,
                &get_shanten_optimized,
                &get_ukiere_optimized,
                other_visible_tiles,
            );
            let mut best_num_ukiere_after_nonupgrade = 0;
            let mut discard_tile_id_to_ukiere = HashMap::new();
            for (discard_tile_id, ukiere_tiles_after_nonupgrade, num_ukiere_after_nonupgrade) in
                ukiere_after_nonupgrade
            {
                if num_ukiere_after_nonupgrade > best_num_ukiere_after_nonupgrade {
                    best_num_ukiere_after_nonupgrade = num_ukiere_after_nonupgrade;
                    discard_tile_id_to_ukiere = HashMap::new();
                    discard_tile_id_to_ukiere
                        .insert(discard_tile_id, ukiere_tiles_after_nonupgrade);
                } else if num_ukiere_after_nonupgrade == best_num_ukiere_after_nonupgrade {
                    discard_tile_id_to_ukiere
                        .insert(discard_tile_id, ukiere_tiles_after_nonupgrade);
                }
            }

            // format for display
            let mut discard_tile_str_to_ukiere = HashMap::new();
            for (discard_tile_id, ukiere_tiles_after_discard) in discard_tile_id_to_ukiere {
                let discard_tile_str =
                    mahjong_tile::get_tile_text_from_id(discard_tile_id).unwrap();
                let mut sorted_ukiere_tiles_after_discard = ukiere_tiles_after_discard.clone();
                sorted_ukiere_tiles_after_discard.sort();
                let ukiere_tiles_str = tile_ids_to_string(&sorted_ukiere_tiles_after_discard);
                discard_tile_str_to_ukiere.insert(discard_tile_str, ukiere_tiles_str);
            }
            assert_eq!(
                best_num_ukiere_after_nonupgrade,
                num_ukiere,
                "tile {} was not in expected_upgrade tiles, but ukiere has increased to {} tiles: {:?}",
                potential_tile_str,
                best_num_ukiere_after_nonupgrade,
                discard_tile_str_to_ukiere,
            );
        }
    }

    // TODO use generics here instead of only accepting MahjongTileId in input types
    fn assert_discards_ukiere_match(
        discards_ukiere: &Vec<(MahjongTileId, Vec<MahjongTileId>, u16)>,
        expected_discards_ukiere: &Vec<(MahjongTileId, Vec<MahjongTileId>, u16)>,
    ) {
        let mut sorted_discards_ukiere = discards_ukiere.clone();
        for (_, ukiere_tile_ids_after_discard, _) in sorted_discards_ukiere.iter_mut() {
            ukiere_tile_ids_after_discard.sort();
        }
        sorted_discards_ukiere.sort();

        let mut sorted_expected_discards_ukiere = expected_discards_ukiere.clone();
        for (_, ukiere_tile_ids_after_discard, _) in sorted_expected_discards_ukiere.iter_mut() {
            ukiere_tile_ids_after_discard.sort();
        }
        sorted_expected_discards_ukiere.sort();
        assert_eq!(sorted_discards_ukiere, sorted_expected_discards_ukiere);
    }

    #[test]
    fn tile_counts_from_string() {
        let tiles = tiles_to_count_array("1234m");
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("1m").unwrap())],
            1
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("2m").unwrap())],
            1
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("3m").unwrap())],
            1
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("4m").unwrap())],
            1
        );
    }

    #[test]
    fn tile_ids_from_string() {
        let tile_ids = mahjong_tile::tiles_to_tile_ids("1234m");
        assert_eq!(tile_ids.len(), 4);
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("1m").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("2m").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("3m").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("4m").unwrap()));
    }

    #[test]
    fn hand_tile_counts_from_string() {
        let tiles = tiles_to_count_array("46p255567s33478m4s");
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("4p").unwrap())],
            1
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("6p").unwrap())],
            1
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("2s").unwrap())],
            1
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("4s").unwrap())],
            1
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("5s").unwrap())],
            3
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("6s").unwrap())],
            1
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("7s").unwrap())],
            1
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("3m").unwrap())],
            2
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("4m").unwrap())],
            1
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("7m").unwrap())],
            1
        );
        assert_eq!(
            tiles[usize::from(mahjong_tile::get_id_from_tile_text("8m").unwrap())],
            1
        );
    }

    #[test]
    fn hand_tile_ids_from_string() {
        let tile_ids = mahjong_tile::tiles_to_tile_ids("46p255567s33478m4s");
        assert_eq!(tile_ids.len(), 14);
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("4p").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("6p").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("2s").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("4s").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("5s").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("6s").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("7s").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("3m").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("4m").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("7m").unwrap()));
        assert!(tile_ids.contains(&mahjong_tile::get_id_from_tile_text("8m").unwrap()));
    }

    #[test]
    fn test_tile_ids_to_complete_group() {
        let single_terminal_tile =
            TileMeld::new(vec![mahjong_tile::get_id_from_tile_text("1m").unwrap()]);
        let ukiere_tile_ids = single_terminal_tile.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = mahjong_tile::tiles_to_tile_ids("123m");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let single_middle_tile =
            TileMeld::new(vec![mahjong_tile::get_id_from_tile_text("8p").unwrap()]);
        let ukiere_tile_ids = single_middle_tile.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = mahjong_tile::tiles_to_tile_ids("6789p");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let single_middle_tile =
            TileMeld::new(vec![mahjong_tile::get_id_from_tile_text("3s").unwrap()]);
        let ukiere_tile_ids = single_middle_tile.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = mahjong_tile::tiles_to_tile_ids("12345s");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let single_honor_tile =
            TileMeld::new(vec![mahjong_tile::get_id_from_tile_text("6z").unwrap()]);
        let ukiere_tile_ids = single_honor_tile.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = mahjong_tile::tiles_to_tile_ids("6z");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let pair = TileMeld::new(vec![
            mahjong_tile::get_id_from_tile_text("2z").unwrap(),
            mahjong_tile::get_id_from_tile_text("2z").unwrap(),
        ]);
        let ukiere_tile_ids = pair.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = mahjong_tile::tiles_to_tile_ids("2z");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let ryanmen = TileMeld::new(vec![
            mahjong_tile::get_id_from_tile_text("3p").unwrap(),
            mahjong_tile::get_id_from_tile_text("4p").unwrap(),
        ]);
        let ukiere_tile_ids = ryanmen.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = mahjong_tile::tiles_to_tile_ids("25p");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let kanchan = TileMeld::new(vec![
            mahjong_tile::get_id_from_tile_text("7m").unwrap(),
            mahjong_tile::get_id_from_tile_text("9m").unwrap(),
        ]);
        let ukiere_tile_ids = kanchan.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = mahjong_tile::tiles_to_tile_ids("8m");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let penchan = TileMeld::new(vec![
            mahjong_tile::get_id_from_tile_text("8p").unwrap(),
            mahjong_tile::get_id_from_tile_text("9p").unwrap(),
        ]);
        let ukiere_tile_ids = penchan.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = mahjong_tile::tiles_to_tile_ids("7p");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);
    }

    #[test]
    fn hand_two_shanten_and_ukiere() {
        let tiles = tiles_to_count_array("46p455567s33478m");
        // hand is 2-shanten: 46p - 455s - 567s - 334m - 78m
        assert_eq!(get_shanten(tiles), 2);
        assert_eq!(get_shanten_optimized(tiles), 2);

        // ukiere tiles: 5p3568s23569m
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("5p3568s23569m");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn hand_one_shanten_and_ukiere() {
        let tiles = tiles_to_count_array("56m23346778p234s");
        // hand is 1-shanten: 56m - 234p - 678p - 3p - 7p - 234s
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: 47m37p
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("47m37p");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn complex_souzu_one_shanten() {
        let tiles = tiles_to_count_array("12234455s345p11z");
        // hand is 1-shanten: 123s - 24s - 455s - 345p - 11z
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: 23456s1z
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("23456s1z");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn floating_one_shanten() {
        // floating 1-shanten is characterized by 2 complete groups + 1 pair + 2 incomplete groups
        // plus 1 unused/floating tile
        // https://riichi.wiki/Iishanten#Yojouhai
        let tiles = tiles_to_count_array("233445m56p4455s7z");
        // hand is 1-shanten: 234m - 345m - 56p - 44s - 55s - 7z
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: 47p45s
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("47p45s");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn complete_one_shanten() {
        // complete 1-shanten is characterized by 2 complete groups + 1 pair + 2 incomplete groups,
        // but the last tile is also part of one of the incomplete groups e.g. 223s
        // https://riichi.wiki/Iishanten#Kanzenkei
        let tiles = tiles_to_count_array("234m22378s22567p");
        // hand is 1-shanten: 234m - 223s - 78s - 22p - 567p
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: 12469s2p
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("12469s2p");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn hand_headless_one_shanten_with_ankou() {
        // headless 1-shanten is characterized by 3 complete groups + no pair (must have at least 1 incomplete group)
        // https://riichi.wiki/Iishanten#Atamanashi
        let tiles = tiles_to_count_array("23s678s56p888p888m");
        // hand is 1-shanten: 23s - 678s - 56p - 888p - 888m
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: completing either ryanmen group or pairing a tile in the ryanmen group
        // total: 1234s4567p
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("1234s4567p");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn hand_kutsuki_one_shanten() {
        // kutsuki 1-shanten is characterized by 3 complete groups + 1 pair: https://riichi.wiki/Iishanten#Kuttsuki
        // example from wiki:
        let tiles = tiles_to_count_array("234678m37s22567p");
        // hand is 1-shanten: 234m - 678m - 3s - 7s - 22p - 567p
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: anything within 2 of a floating tile (3s7s), and 2p
        // total: 123456789s2p
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("123456789s2p");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        // example from youtube video: https://youtu.be/TulF31VKJ94?si=DlCBB2flwB-Y7P18 (timestamp 1:33:59)
        let tiles = tiles_to_count_array("2344567m556p678s");
        // hand is 1-shanten: 234m - 456m - 7m - 556p - 678s (but the 4m could be considered floating as well, or as part of 2344m shape)
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: anything within 2 of a floating tile (4m7m6p), and
        // the manzu complex shape can accept 147m (2344m-567m or 234m-4567m), 2358m (24m-34567m)
        // total: 123456789m45678p
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("123456789m45678p");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn complete_hand_shanten() {
        let tiles = tiles_to_count_array("55588m234s11p666z1p");
        assert_eq!(get_shanten(tiles), -1);
        assert_eq!(get_shanten_optimized(tiles), -1);

        let ukiere_tiles = get_ukiere(tiles);
        assert_eq!(ukiere_tiles.len(), 0);
        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_eq!(ukiere_tiles.len(), 0);
    }

    #[test]
    fn complete_honitsu_hand_shanten() {
        let tiles = tiles_to_count_array("3335577899m111z8m");
        assert_eq!(get_shanten(tiles), -1);
        assert_eq!(get_shanten_optimized(tiles), -1);

        let ukiere_tiles = get_ukiere(tiles);
        assert_eq!(ukiere_tiles.len(), 0);
        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_eq!(ukiere_tiles.len(), 0);
    }

    #[test]
    fn six_block_hand_two_shanten() {
        // example from: https://pathofhouou.blogspot.com/2019/05/calculating-shanten-and-ukeire.html
        let tiles = tiles_to_count_array("12346m6799p1268s");
        assert_eq!(get_shanten(tiles), 2);
        assert_eq!(get_shanten_optimized(tiles), 2);

        // ukiere tiles: 5m58p37s
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("5m58p37s");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn three_shanten_hand() {
        // example from: https://pathofhouou.blogspot.com/2019/05/calculating-shanten-and-ukeire.html
        let tiles = tiles_to_count_array("12588m27789p889s");
        assert_eq!(get_shanten(tiles), 3);
        assert_eq!(get_shanten_optimized(tiles), 3);

        // ukiere tiles: 12345678m123456789p6789s (12m-5m-88m-2p-7p-789p-889s; also 3-shanten from chiitoi)
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("12345678m123456789p6789s");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_tanki_wait() {
        let tiles = tiles_to_count_array("123999m5558p666z");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 8p
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("8p");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_kanchan_wait() {
        let tiles = tiles_to_count_array("13p456777999s33z");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 2p
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("2p");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_penchan_wait() {
        let tiles = tiles_to_count_array("12777m345p67899s");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 3m
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("3m");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_shanpon_wait() {
        let tiles = tiles_to_count_array("123999m44p99s777z");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 4p9s
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("4p9s");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_ryanmen_wait() {
        let tiles = tiles_to_count_array("666m78p666789s22z");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 69p
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("69p");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_nobetan_wait() {
        //e.g. 3456p - https://riichi.wiki/Nobetan
        let tiles = tiles_to_count_array("777m3456p555666s");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 36p
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("36p");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_aryanmen_wait() {
        // e.g. 3455s - https://riichi.wiki/Aryanmen
        let tiles = tiles_to_count_array("567m123456p3455s");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 25s
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("25s");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_kantan_wait() {
        // 2-sided wait e.g. 6888m - https://riichi.wiki/Ryantan#Kantan
        let tiles = tiles_to_count_array("2226888m444p111z");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 67m
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("67m");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_pentan_wait() {
        // 2-sided wait e.g. 1222m - https://riichi.wiki/Kantan#Pentan
        let tiles = tiles_to_count_array("1222m678p345789s");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 13m
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("13m");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_standard_sanmenchan_wait() {
        // 3-sided wait e.g. 34567s - https://riichi.wiki/Sanmenchan
        let tiles = tiles_to_count_array("666m33678p34567s");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 258s
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("258s");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_ryantan_wait() {
        // 3-sided wait e.g. 5556m - https://riichi.wiki/Kantan#Ryantan
        let tiles = tiles_to_count_array("5556m234789p666z");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 467m
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("467m");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_entotsu_wait() {
        // 3-sided wait e.g. 11m45666s - https://riichi.wiki/Sanmenchan#Entotsu
        let tiles = tiles_to_count_array("11m222456p45666s");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 1m36s
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("1m36s");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_sanmentan_wait() {
        // 3-sided wait e.g. 2345678p -  https://riichi.wiki/Sanmenchan#Sanmentan
        let tiles = tiles_to_count_array("233445m2345678p");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 258p
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("258p");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn chiitoi_one_shanten() {
        // https://riichi.wiki/Iishanten#Chiitoitsu
        // example with no triplets
        let tiles = tiles_to_count_array("1166m4499s667p25z");
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: 7p25z
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("7p25z");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        // example with a triplet
        let tiles = tiles_to_count_array("3388m229p111s337z");
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: 9p7z
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("9p7z");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        // example that is 1-shanten for both chiitoi and standard hand structure
        let tiles = tiles_to_count_array("233m11223399p56s");
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: 2m56s (to form a sixth pair), 134m9p47s (to complete a group)
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("1234m9p4567s");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn chiitoi_tenpai() {
        let tiles = tiles_to_count_array("1166m4499s667p55z");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 7p
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("7p");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn chiitoi_complete_hand() {
        let tiles = tiles_to_count_array("1166m4499s6677p55z");
        assert_eq!(get_shanten(tiles), -1);
        assert_eq!(get_shanten_optimized(tiles), -1);

        let ukiere_tiles = get_ukiere(tiles);
        assert_eq!(ukiere_tiles.len(), 0);
        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_eq!(ukiere_tiles.len(), 0);
    }

    #[test]
    fn kokushi_musou_one_shanten() {
        // https://riichi.wiki/Iishanten#Kokushi_musou
        // example with a pair
        let tiles = tiles_to_count_array("1m139p19s1234566z");
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: 9m7z
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("9m7z");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        // example with no pair
        let tiles = tiles_to_count_array("19m139p19s123456z");
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: 19m19p19s1234567z (in particular, drawing 7z gives a 13-sided wait)
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("19m19p19s1234567z");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn kokushi_musou_tenpai() {
        // aka thirteen orphans - https://riichi.wiki/Kokushi_musou
        // 1-sided wait
        let tiles = tiles_to_count_array("1m19p19s12345667z");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 9m
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("9m");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        // 13-sided wait
        let tiles = tiles_to_count_array("19m19p19s1234567z");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 19m19p19s1234567z (any terminal or honor tile)
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("19m19p19s1234567z");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn kokushi_musou_complete() {
        // aka thirteen orphans - https://riichi.wiki/Kokushi_musou
        let tiles = tiles_to_count_array("19m19p199s1234567z");
        assert_eq!(get_shanten(tiles), -1);
        assert_eq!(get_shanten_optimized(tiles), -1);

        let ukiere_tiles = get_ukiere(tiles);
        assert_eq!(ukiere_tiles.len(), 0);
        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_eq!(ukiere_tiles.len(), 0);
    }

    #[bench]
    fn bench_standard_shanten(b: &mut Bencher) {
        let tiles = tiles_to_count_array("12234455s345p11z");
        // hand is 1-shanten: 123s - 24s - 455s - 345p - 11z
        // ukiere tiles: 23456s1z
        b.iter(|| {
            get_shanten(tiles);
            get_ukiere(tiles)
        });
    }

    // #[test]
    // fn debug_get_hand_interpretations() {
    //     let tiles = tiles_to_count_array("12234455s345p11z");
    //     let hand_interpretations = get_hand_interpretations(tiles);
    //     for hand_interpretation in hand_interpretations {
    //         println!(
    //             "{} -> ukiere tiles {}",
    //             hand_interpretation,
    //             tile_ids_to_string(&hand_interpretation.get_ukiere())
    //         );
    //     }
    // }

    // #[test]
    // fn debug_get_hand_interpretations_min_shanten() {
    //     // let tiles = tiles_to_count_array("23s678s56p888p888m");
    //     let tiles = tiles_to_count_array("12234455s345p11z");
    //     let hand_interpretations = get_hand_interpretations_min_shanten(tiles, 3);
    //     for hand_interpretation in hand_interpretations {
    //         println!(
    //             "{} -> ukiere tiles {}",
    //             hand_interpretation,
    //             tile_ids_to_string(&hand_interpretation.get_ukiere())
    //         );
    //     }
    // }

    #[bench]
    fn bench_standard_shanten_optimized(b: &mut Bencher) {
        let tiles = tiles_to_count_array("12234455s345p11z");
        // hand is 1-shanten: 123s - 24s - 455s - 345p - 11z
        // ukiere tiles: 23456s1z
        b.iter(|| {
            get_shanten_optimized(tiles);
            get_ukiere_optimized(tiles)
        });
    }

    #[test]
    fn wwyd_efficiency_problem_example1() {
        // examples from https://justanotherjapanesemahjongblog.blogspot.com/2012/01/choosing-multi-wait-tenpai-problem.html
        let tiles = tiles_to_count_array("123667m6889p1278s");
        assert_eq!(get_best_shanten_after_discard(tiles, &get_shanten), 2);
        assert_eq!(
            get_best_shanten_after_discard(tiles, &get_shanten_optimized),
            2
        );
        // best discard 9p (or 6p) -> 2-shanten

        // ukiere tiles after discard 9p (or 6p): 568m78p369s
        let mut expected_ukiere_tiles_after_discard = HashMap::new();
        expected_ukiere_tiles_after_discard.insert("6p", mahjong_tile::tiles_to_tile_ids("568m78p369s"));
        expected_ukiere_tiles_after_discard.insert("9p", mahjong_tile::tiles_to_tile_ids("568m78p369s"));
        expected_ukiere_tiles_after_discard.insert("6m", mahjong_tile::tiles_to_tile_ids("58m369s"));
        expected_ukiere_tiles_after_discard.insert("7m", mahjong_tile::tiles_to_tile_ids("6m78p369s"));
        expected_ukiere_tiles_after_discard.insert("8p", mahjong_tile::tiles_to_tile_ids("7p369s"));
        expected_ukiere_tiles_after_discard.insert("1s", mahjong_tile::tiles_to_tile_ids("7p69s"));
        expected_ukiere_tiles_after_discard.insert("2s", mahjong_tile::tiles_to_tile_ids("7p69s"));
        expected_ukiere_tiles_after_discard.insert("7s", mahjong_tile::tiles_to_tile_ids("7p3s"));
        expected_ukiere_tiles_after_discard.insert("8s", mahjong_tile::tiles_to_tile_ids("7p3s"));

        for (discard_tile_str, expected_ukiere_tiles) in expected_ukiere_tiles_after_discard {
            let ukiere_tiles = get_ukiere_after_discard(
                tiles,
                mahjong_tile::get_id_from_tile_text(discard_tile_str).unwrap(),
                &get_ukiere,
            );
            assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

            let ukiere_tiles = get_ukiere_after_discard(
                tiles,
                mahjong_tile::get_id_from_tile_text(discard_tile_str).unwrap(),
                &get_ukiere_optimized,
            );
            assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
        }
    }

    #[test]
    fn wwyd_efficiency_problem_example2() {
        // examples from https://justanotherjapanesemahjongblog.blogspot.com/2012/01/choosing-multi-wait-tenpai-problem.html
        let tiles = tiles_to_count_array("2334578m34p66999s");
        assert_eq!(get_best_shanten_after_discard(tiles, &get_shanten), 1);
        assert_eq!(
            get_best_shanten_after_discard(tiles, &get_shanten_optimized),
            1
        );
        // best discard 8m -> 1-shanten

        // ukiere tiles after discard 8m: 146m25p
        let mut expected_ukiere_tiles_after_discard = HashMap::new();
        expected_ukiere_tiles_after_discard.insert("8m", mahjong_tile::tiles_to_tile_ids("146m25p"));
        expected_ukiere_tiles_after_discard.insert("2m", mahjong_tile::tiles_to_tile_ids("69m25p"));
        expected_ukiere_tiles_after_discard.insert("3m", mahjong_tile::tiles_to_tile_ids("69m25p"));
        expected_ukiere_tiles_after_discard.insert("5m", mahjong_tile::tiles_to_tile_ids("69m25p"));
        expected_ukiere_tiles_after_discard.insert("7m", mahjong_tile::tiles_to_tile_ids("14m25p"));
        expected_ukiere_tiles_after_discard.insert("3p", mahjong_tile::tiles_to_tile_ids("1469m"));
        expected_ukiere_tiles_after_discard.insert("4p", mahjong_tile::tiles_to_tile_ids("1469m"));

        for (discard_tile_str, expected_ukiere_tiles) in expected_ukiere_tiles_after_discard {
            let ukiere_tiles = get_ukiere_after_discard(
                tiles,
                mahjong_tile::get_id_from_tile_text(discard_tile_str).unwrap(),
                &get_ukiere,
            );
            assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

            let ukiere_tiles = get_ukiere_after_discard(
                tiles,
                mahjong_tile::get_id_from_tile_text(discard_tile_str).unwrap(),
                &get_ukiere_optimized,
            );
            assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
        }
    }

    #[test]
    fn wwyd_efficiency_problem_example3() {
        // examples from https://justanotherjapanesemahjongblog.blogspot.com/2012/01/choosing-multi-wait-tenpai-problem.html
        let tiles = tiles_to_count_array("2368m24888p33567s");
        assert_eq!(get_best_shanten_after_discard(tiles, &get_shanten), 1);
        assert_eq!(
            get_best_shanten_after_discard(tiles, &get_shanten_optimized),
            1
        );
        // best discard -> 1-shanten

        // ukiere tiles after discard 6m or 8m: 14m3p - but the argument is that, if you keep 68m (and cut 24p),
        // then if you were to draw 5m (upgrade to ryanmen wait), you have overlapping wait on 4m (between 23m and now 56m)
        let mut expected_ukiere_tiles_after_discard = HashMap::new();
        expected_ukiere_tiles_after_discard.insert("8m", mahjong_tile::tiles_to_tile_ids("14m3p"));
        expected_ukiere_tiles_after_discard.insert("6m", mahjong_tile::tiles_to_tile_ids("14m3p"));
        expected_ukiere_tiles_after_discard.insert("2p", mahjong_tile::tiles_to_tile_ids("147m"));
        expected_ukiere_tiles_after_discard.insert("4p", mahjong_tile::tiles_to_tile_ids("147m"));
        expected_ukiere_tiles_after_discard.insert("2m", mahjong_tile::tiles_to_tile_ids("7m3p"));
        expected_ukiere_tiles_after_discard.insert("3m", mahjong_tile::tiles_to_tile_ids("7m3p"));
        assert_ukiere_tiles_after_discard_match(tiles, &expected_ukiere_tiles_after_discard);
    }

    #[test]
    fn tenpai_and_n_shanten_examples_riichi_book_1() {
        // examples from riichi book 1: section 3.2.3 (ready and n-away)
        // tenpai hand
        let tiles = tiles_to_count_array("34588p23678s777z");
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        // ukiere tiles: 14s
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("14s");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        // 1-shanten hand:
        let tiles = tiles_to_count_array("35588p23678s777z");
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // ukiere tiles: 458p14s
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("458p14s");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        // 2-shanten hand
        let tiles = tiles_to_count_array("35588p23677s777z");
        assert_eq!(get_shanten(tiles), 2);
        assert_eq!(get_shanten_optimized(tiles), 2);

        // ukiere tiles: 3458p12345678s (3p236s will make the hand 1-shanten for chiitoi)
        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("3458p12345678s");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn wwyd_basic_efficiency_riichi_book_1() {
        // examples from riichi book 1: section 3.2.3 (ready and n-away > advancing your hand)
        let tiles = tiles_to_count_array("5677m34p45579s666z");
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);
        // 567m - 7m - 34p - 455s - 79s - 666z
        // (can't interpret as 45s-579s because you need the pair of 5s)

        let mut expected_ukiere_tiles_after_discard = HashMap::new();
        expected_ukiere_tiles_after_discard.insert("7m", mahjong_tile::tiles_to_tile_ids("25p8s"));
        expected_ukiere_tiles_after_discard.insert("4s", mahjong_tile::tiles_to_tile_ids("25p8s"));
        assert_ukiere_tiles_after_discard_match(tiles, &expected_ukiere_tiles_after_discard);
    }

    #[test]
    fn wwyd_kutsuki_iishanten() {
        // examples from youtube video (recording of a presentation/lesson): https://youtu.be/TulF31VKJ94?si=baGLLhyg-Mr2cbqd
        // 1st example (timestamp 1:33:59)
        let tiles = tiles_to_count_array("2344567m556p6678s");
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // discarding 6p, 6s, 4m, or 7m leaves with kutsuki iishanten:
        // discard 6p: 55p - 678s - 6s - 234m - 4567m -> floating 6s, 4m, and 7m; 1m will form an indirect incomplete group (123m-44m-567m) and 9s forms 66s-789s
        // discard 6s: 556p - 678s - 234m - 4567m -> floating 6p, 4m, and 7m; 1m will form incomplete group as well
        // discard 4m/7m: 556p - 678s - 6s (+ two sequences in manzu) -> floating 6p, 6s; 9s will form incomplete group as well
        // discard 5p (headless 1-shanten if 6678s is one group): 147m will form a pair in manzu, 47p leaves you with 6678s aryanmen, 69s forms pair in souzu
        let mut expected_ukiere_tiles_after_discard = HashMap::new();
        expected_ukiere_tiles_after_discard.insert("6p", mahjong_tile::tiles_to_tile_ids("123456789m5p456789s"));
        expected_ukiere_tiles_after_discard.insert("6s", mahjong_tile::tiles_to_tile_ids("123456789m45678p"));
        expected_ukiere_tiles_after_discard.insert("4m", mahjong_tile::tiles_to_tile_ids("45678p456789s"));
        expected_ukiere_tiles_after_discard.insert("7m", mahjong_tile::tiles_to_tile_ids("45678p456789s"));
        expected_ukiere_tiles_after_discard.insert("5p", mahjong_tile::tiles_to_tile_ids("147m47p69s"));
        assert_ukiere_tiles_after_discard_match(tiles, &expected_ukiere_tiles_after_discard);

        // 2nd example (timestamp 1:40:29)
        let tiles = tiles_to_count_array("2446888m4678999p");
        assert_eq!(get_shanten(tiles), 1);
        assert_eq!(get_shanten_optimized(tiles), 1);

        // discarding 2m, 6m, or 4p leaves with kutsuki iishanten:
        // discard 2m: 44m - 6m - 888m - 4p - 678p - 999p -> note: can accept 9p as well, which forms an incomplete group 46p-789p-999p
        // discard 6m: 44m - 2m - 888m - 4p - 678p - 999p -> note: can accept 9p as well, which forms an incomplete group 46p-789p-999p
        // discard 4p: 2m - 44m - 6m - 888m - 678p - 999p
        // discard 4m (headless + ankou): 246m - 888m - 4p - 678p - 999p -> can accept 7m (24m-678m-88m) and 5p (456p-789p-99p) as well
        let mut expected_ukiere_tiles_after_discard = HashMap::new();
        expected_ukiere_tiles_after_discard.insert("2m", mahjong_tile::tiles_to_tile_ids("45678m234569p"));
        expected_ukiere_tiles_after_discard.insert("6m", mahjong_tile::tiles_to_tile_ids("1234m234569p"));
        expected_ukiere_tiles_after_discard.insert("4m", mahjong_tile::tiles_to_tile_ids("23567m45p"));
        expected_ukiere_tiles_after_discard.insert("4p", mahjong_tile::tiles_to_tile_ids("12345678m"));
        // remaining options lock in a pair of 8m or 9p, which means the 4p doesn't count towards shanten and the 2446m needs to form 2 groups
        expected_ukiere_tiles_after_discard.insert("8m", mahjong_tile::tiles_to_tile_ids("35m"));
        expected_ukiere_tiles_after_discard.insert("6p", mahjong_tile::tiles_to_tile_ids("35m"));
        expected_ukiere_tiles_after_discard.insert("9p", mahjong_tile::tiles_to_tile_ids("35m"));
        assert_ukiere_tiles_after_discard_match(tiles, &expected_ukiere_tiles_after_discard);
    }

    #[test]
    fn tanki_tenpai_upgrade() {
        // example from riichi book 1, section 3.2.5 (Pairs (toitsu) > Building the head)
        let tiles = tiles_to_count_array("789m234567p3457s");
        // currently, the hand is tenpai, with tanki wait on 7s
        assert_eq!(get_shanten(tiles), 0);
        assert_eq!(get_shanten_optimized(tiles), 0);

        let expected_ukiere_tiles = mahjong_tile::tiles_to_tile_ids("7s");
        let ukiere_tiles = get_ukiere(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        let ukiere_tiles = get_ukiere_optimized(tiles);
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);

        // but if you were to draw 69m124578p2356s, the wait improves significantly
        // (you would discard 7s and have at least 6 ukiere, either nobetan or aryanmen)
        // even drawing 4s would slightly improve the wait: could discard 3s for a 4457s with a kanchan wait on 6s (4 ukiere)
        let mut expected_upgrade_tiles: HashMap<&'static str, HashMap<&'static str, Vec<MahjongTileId>>> =
            HashMap::new();
        expected_upgrade_tiles.insert("1p", hashmap!["7s" => mahjong_tile::tiles_to_tile_ids("147p")]);
        expected_upgrade_tiles.insert("2p", hashmap!["7s" => mahjong_tile::tiles_to_tile_ids("258p")]);
        expected_upgrade_tiles.insert("4p", hashmap!["7s" => mahjong_tile::tiles_to_tile_ids("147p")]);
        expected_upgrade_tiles.insert("5p", hashmap!["7s" => mahjong_tile::tiles_to_tile_ids("258p")]);
        expected_upgrade_tiles.insert("7p", hashmap!["7s" => mahjong_tile::tiles_to_tile_ids("147p")]);
        expected_upgrade_tiles.insert("8p", hashmap!["7s" => mahjong_tile::tiles_to_tile_ids("258p")]);
        expected_upgrade_tiles.insert("6m", hashmap!["7s" => mahjong_tile::tiles_to_tile_ids("69m")]);
        expected_upgrade_tiles.insert("9m", hashmap!["7s" => mahjong_tile::tiles_to_tile_ids("69m")]);
        expected_upgrade_tiles.insert("2s", hashmap!["7s" => mahjong_tile::tiles_to_tile_ids("25s")]);
        expected_upgrade_tiles.insert("3s", hashmap!["7s" => mahjong_tile::tiles_to_tile_ids("36s")]);
        expected_upgrade_tiles.insert("5s", hashmap!["7s" => mahjong_tile::tiles_to_tile_ids("25s")]);
        expected_upgrade_tiles.insert(
            "6s",
            hashmap!["7s" => mahjong_tile::tiles_to_tile_ids("36s"), "3s" => mahjong_tile::tiles_to_tile_ids("47s")],
        );
        expected_upgrade_tiles.insert("4s", hashmap!["3s" => mahjong_tile::tiles_to_tile_ids("6s")]);

        // calculate the tiles that, if drawn, upgrade the ukiere of the hand at the current shanten
        let other_visible_tiles = vec![];
        assert_upgrade_tiles_match(tiles, &expected_upgrade_tiles, &other_visible_tiles);

        // for example, test that the ukiere tiles count is correct after drawing 1p
        let ukiere_tiles = mahjong_tile::tiles_to_tile_ids("147p");
        // remove the original tiles in hand, plus the next tile that is drawn
        let mut visible_tile_ids = Vec::new();
        let tiles_as_ids = tile_count_array_to_tile_ids(tiles);
        visible_tile_ids.extend_from_slice(&tiles_as_ids.as_slice());
        visible_tile_ids.push(mahjong_tile::get_id_from_tile_text("1p").unwrap());
        // println!(
        //     "visible tile ids = {}",
        //     tile_ids_to_string(&visible_tile_ids)
        // );
        assert_eq!(9, get_num_tiles_remaining(&ukiere_tiles, &visible_tile_ids));

        // check the situation after drawing 1p: still in tenpai, but with improved wait after discarding 7s
        let tiles_after_draw =
            add_tile_id_to_count_array(tiles, mahjong_tile::get_id_from_tile_text("1p").unwrap());
        let shanten_after_discard =
            get_best_shanten_after_discard(tiles_after_draw, &get_shanten_optimized);
        assert_eq!(shanten_after_discard, 0);

        let other_visible_tiles = Vec::new();
        let best_ukiere_after_discard = get_most_ukiere_after_discard(
            tiles_after_draw,
            shanten_after_discard,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );
        let mut expected_discard_ukiere: Vec<(MahjongTileId, Vec<MahjongTileId>, u16)> = Vec::new();
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("7s").unwrap(),
            mahjong_tile::tiles_to_tile_ids("147p"),
            9,
        ));
        assert_discards_ukiere_match(&best_ukiere_after_discard, &expected_discard_ukiere);

        // for (discard_tile_id, ukiere_tile_ids_after_discard, num_ukiere_after_discard) in
        //     best_ukiere_after_discard
        // {
        //     println!(
        //         "discard {} -> {} ukiere tiles: {} ",
        //         mahjong_tile::get_tile_text_from_id(discard_tile_id).unwrap(),
        //         num_ukiere_after_discard,
        //         tile_ids_to_string(&ukiere_tile_ids_after_discard)
        //     );
        // }

        let shanten_ukiere_after_each_discard = get_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );
        print_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &shanten_ukiere_after_each_discard,
            &other_visible_tiles,
        );
    }

    fn print_shanten_ukiere_after_each_discard<T: Into<MahjongTileId> + Clone>(
        tile_count_array: [u8; 34],
        shanten_ukiere_after_each_discard: &Vec<(T, i8, Vec<T>, u16)>,
        other_visible_tiles: &Vec<T>,
    ) {
        if get_total_tiles_from_count_array(tile_count_array) != 14 {
            // TODO eventually will need to handle the case when there are more tiles due to quads
            panic!("invalid number of tiles")
        }
        let original_tiles = tile_count_array_to_tile_ids(tile_count_array);
        println!(
            "shanten + ukiere after each discard: {}",
            tile_ids_to_string(&original_tiles)
        );
        let mut sorted_shanten_ukiere_after_each_discard =
            shanten_ukiere_after_each_discard.clone();
        sorted_shanten_ukiere_after_each_discard.sort_by(
            |(_, shanten_1, _, ukiere_1), (_, shanten_2, _, ukiere_2)| match shanten_1
                .cmp(shanten_2)
            {
                // sort by shanten ascending, then by ukiere descending
                Ordering::Equal => ukiere_2.cmp(ukiere_1),
                Ordering::Less => Ordering::Less,
                Ordering::Greater => Ordering::Greater,
            },
        );

        let best_shanten = get_best_shanten_after_discard(tile_count_array, &get_shanten_optimized);

        // from the initial hand - try each discard
        for (
            discard_tile_id,
            shanten_after_discard,
            ukiere_tile_ids_after_discard,
            num_ukiere_after_discard,
        ) in sorted_shanten_ukiere_after_each_discard
        {
            let mut sorted_ukiere_tile_ids_after_discard: Vec<MahjongTileId> = ukiere_tile_ids_after_discard.iter().cloned().map(|t| t.into()).collect();
            sorted_ukiere_tile_ids_after_discard.sort();
            println!(
                "discard {} -> {} shanten, {} ukiere tiles: {} ",
                mahjong_tile::get_tile_text_from_id(discard_tile_id.clone()).unwrap(),
                shanten_after_discard,
                num_ukiere_after_discard,
                tile_ids_to_string(&sorted_ukiere_tile_ids_after_discard)
            );

            let new_count_array =
                remove_tile_id_from_count_array(tile_count_array, discard_tile_id);
            let new_shanten = get_shanten_optimized(new_count_array);
            // println!(
            //     "hand {} is {} shanten",
            //     mahjong_hand::tile_count_array_to_string(&new_count_array),
            //     new_shanten
            // );

            // for performance, only print out improve results (i.e. results after drawing an ukiere tile)
            // for the discards that result in best shanten (i.e. don't print out for suboptimal discards)
            if new_shanten == best_shanten {
                println!("  after advancing shanten:");
                let mut improve_options = Vec::new();
                for improve_tile_id in ukiere_tile_ids_after_discard {
                    let after_improve_draw_count_array =
                        add_tile_id_to_count_array(new_count_array, improve_tile_id.clone());
                    let mut options_after_ukiere_draw = get_most_ukiere_after_discard(
                        after_improve_draw_count_array,
                        new_shanten,
                        &get_shanten_optimized,
                        &get_ukiere_optimized,
                        other_visible_tiles,
                    );
                    options_after_ukiere_draw.iter_mut().for_each(
                        |(_, ukiere_after_upgrade_discard, _)| {
                            ukiere_after_upgrade_discard.sort();
                        },
                    );

                    // sort discard options (after drawing improvement tile) by descending number of ukiere tiles
                    // let mut options_after_improve_sorted = options_after_ukiere_draw.clone();
                    options_after_ukiere_draw.sort_by(
                        |(_, _, num_ukiere_tiles_after_improve1),
                         (_, _, num_ukiere_tiles_after_improve2)| {
                            num_ukiere_tiles_after_improve2.cmp(num_ukiere_tiles_after_improve1)
                        },
                    );
                    improve_options.push((improve_tile_id, options_after_ukiere_draw));
                }

                // sort improve options by max number of ukiere tiles after discard (descending)
                improve_options.sort_by(
                    |(_, options_after_improve_sorted1), (_, options_after_improve_sorted2)| {
                        let max_ukiere_after_improve1 =
                            options_after_improve_sorted1.get(0).unwrap().2;
                        let max_ukiere_after_improve2 =
                            options_after_improve_sorted2.get(0).unwrap().2;
                        max_ukiere_after_improve2.cmp(&max_ukiere_after_improve1)
                    },
                );

                let improve_options_str_parts: Vec<String> = improve_options
                    .into_iter()
                    .map(|(improve_tile_id, options_after_improve)| {
                        let options_after_improve_str_parts: Vec<String> = options_after_improve
                            .into_iter()
                            .map(
                                |(
                                    discard_after_improve_tile_id,
                                    ukiere_tiles_after_improve_discard,
                                    num_ukiere_tiles_after_improve_discard,
                                )| {
                                    format!(
                                        "cut {} => {} ukiere: {}",
                                        mahjong_tile::get_tile_text_from_id(
                                            discard_after_improve_tile_id
                                        )
                                        .unwrap(),
                                        num_ukiere_tiles_after_improve_discard,
                                        tile_ids_to_string(&ukiere_tiles_after_improve_discard)
                                    )
                                },
                            )
                            .collect();
                        format!(
                            "    draw {} -> {}",
                            mahjong_tile::get_tile_text_from_id(improve_tile_id).unwrap(),
                            options_after_improve_str_parts.join("; ")
                        )
                    })
                    .collect();
                println!("{}", improve_options_str_parts.join("\n"));
            }

            // for performance, only print out upgrades for the discards that result in best shanten
            // (i.e. don't print out for suboptimal discards)
            if new_shanten == best_shanten {
                let upgrades = get_upgrade_tiles(
                    new_count_array,
                    &get_shanten_optimized,
                    &get_ukiere_optimized,
                    other_visible_tiles,
                );
                let has_upgrades = !upgrades.is_empty();
                if has_upgrades {
                    println!("  upgrades:");
                    let mut upgrade_options = Vec::new();
                    for (upgrade_tile_id, discard_to_ukiere) in upgrades {
                        let mut discard_to_ukiere_options = Vec::new();
                        for (discard_tile_id, (ukiere_after_discard, num_ukiere_after_discard)) in
                            discard_to_ukiere
                        {
                            // sort the ukiere tiles by tile id
                            let mut sorted_ukiere_after_discard = ukiere_after_discard.clone();
                            sorted_ukiere_after_discard.sort();

                            discard_to_ukiere_options.push((
                                discard_tile_id,
                                sorted_ukiere_after_discard,
                                num_ukiere_after_discard,
                            ));
                        }
                        // sort discard options (after drawing upgrade) by num ukiere tiles after discard (descending)
                        discard_to_ukiere_options.sort_by(
                            |(_, _, num_ukiere_after_discard1),
                             (_, _, num_ukiere_after_discard2)| {
                                num_ukiere_after_discard2.cmp(num_ukiere_after_discard1)
                            },
                        );
                        upgrade_options.push((upgrade_tile_id, discard_to_ukiere_options));
                    }
                    // sort upgrade options by max num ukiere tiles after discard (descending)
                    upgrade_options.sort_by(
                        |(_, options_after_upgrade1), (_, options_after_upgrade2)| {
                            let max_ukiere_after_upgrade1 =
                                options_after_upgrade1.get(0).unwrap().2;
                            let max_ukiere_after_upgrade2 =
                                options_after_upgrade2.get(0).unwrap().2;
                            max_ukiere_after_upgrade2.cmp(&max_ukiere_after_upgrade1)
                        },
                    );

                    let upgrade_options_str_parts: Vec<String> = upgrade_options
                        .into_iter()
                        .map(|(upgrade_tile_id, discard_options)| {
                            let discard_to_ukiere_str_parts: Vec<String> = discard_options
                                .into_iter()
                                .map(
                                    |(
                                        discard_after_upgrade_tile_id,
                                        sorted_ukiere_after_upgrade_discard,
                                        num_ukiere_after_upgrade_discard,
                                    )| {
                                        format!(
                                            "cut {} => {} ukiere: {}",
                                            mahjong_tile::get_tile_text_from_id(
                                                discard_after_upgrade_tile_id
                                            )
                                            .unwrap(),
                                            num_ukiere_after_upgrade_discard,
                                            tile_ids_to_string(
                                                &sorted_ukiere_after_upgrade_discard
                                            )
                                        )
                                    },
                                )
                                .collect();
                            format!(
                                "    draw {} -> {}",
                                mahjong_tile::get_tile_text_from_id(upgrade_tile_id).unwrap(),
                                discard_to_ukiere_str_parts.join("; ")
                            )
                        })
                        .collect();
                    println!("{}", upgrade_options_str_parts.join("\n"));
                }
            }
        }
    }

    #[test]
    fn upgrade_analysis_tenhou_hand_example() {
        // hand: 345m1156p4666778s (in game: i had 345m11256p46778s6s, could discard 2p and draw 6s to reach this hand state)
        // my program's analysis outputs: cut 4s => 17 ukiere: 147p679s
        // but this is wrong: https://tenhou.net/2/?q=345m1156p4666778s - it's missing 58s as additional ukiere tiles
        // cut 4s => 24 ukiere: 147p56789s (my program's analysis of ukiere after cut 7s is correct: 147p569s)
        let tiles_after_draw = tiles_to_count_array("345m1156p4666778s");
        let shanten_after_discard =
            get_best_shanten_after_discard(tiles_after_draw, &get_shanten_optimized);
        assert_eq!(shanten_after_discard, 1);

        let tile_id_4s = mahjong_tile::get_id_from_tile_text("4s").unwrap();
        let tiles_after_discard_4s = remove_tile_id_from_count_array(tiles_after_draw, tile_id_4s);
        // this hand interpretation is missing from the hand interpretations, because it splits up the three 6s tiles into a sequence, an incomplete group, and a isolated tile
        let hand_interpretation = HandInterpretation {
            total_tile_count_array: tiles_after_discard_4s,
            groups: vec![
                TileMeld::new(mahjong_tile::tiles_to_tile_ids("345m")),
                TileMeld::new(mahjong_tile::tiles_to_tile_ids("11p")),
                TileMeld::new(mahjong_tile::tiles_to_tile_ids("56p")),
                TileMeld::new(mahjong_tile::tiles_to_tile_ids("6s")),
                TileMeld::new(mahjong_tile::tiles_to_tile_ids("67s")),
                TileMeld::new(mahjong_tile::tiles_to_tile_ids("678s")),
            ],
        };
        assert_tile_ids_match(
            &hand_interpretation.get_ukiere(),
            &mahjong_tile::tiles_to_tile_ids("47p58s"),
        );
        assert_eq!(hand_interpretation.get_standard_shanten(), 1);

        let expected_ukiere_after_discard_4s = mahjong_tile::tiles_to_tile_ids("147p56789s");
        let ukiere_after_discard_4s =
            get_ukiere_after_discard(tiles_after_draw, tile_id_4s, &get_ukiere);
        // println!("checking ukiere after discard 4s from 345m1156p4666778s (using get_ukiere):");
        assert_tile_ids_match(&ukiere_after_discard_4s, &expected_ukiere_after_discard_4s);
        // println!("get_ukiere after discard 4s from 345m1156p4666778s is correct");

        let ukiere_after_discard_4s = get_ukiere_after_discard(
            tiles_after_draw,
            mahjong_tile::get_id_from_tile_text("4s").unwrap(),
            &get_ukiere_optimized,
        );
        // println!(
        //     "checking ukiere after discard 4s from 345m1156p4666778s (using get_ukiere_optimized):"
        // );
        assert_tile_ids_match(&ukiere_after_discard_4s, &expected_ukiere_after_discard_4s);
        // println!("get_ukiere_optimized after discard 4s from 345m1156p4666778s is correct");
    }

    #[test]
    fn wwyd_hand_game_example() {
        // east-1, seat: east, (25k points all), dora indicator: 1m (dora: 2m)
        let tiles_after_draw = tiles_to_count_array("5789s357p34667m11z");
        let shanten_after_discard =
            get_best_shanten_after_discard(tiles_after_draw, &get_shanten_optimized);
        let other_visible_tiles = Vec::new();
        let best_ukiere_after_discard = get_most_ukiere_after_discard(
            tiles_after_draw,
            shanten_after_discard,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );

        let mut expected_discard_ukiere: Vec<(MahjongTileId, Vec<MahjongTileId>, u16)> = Vec::new();
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("5s").unwrap(),
            mahjong_tile::tiles_to_tile_ids("2568m46p1z"),
            24,
        ));
        assert_discards_ukiere_match(&best_ukiere_after_discard, &expected_discard_ukiere);

        // best discard = 5s -> 2 shanten, 24 ukiere: 2568m46p1z
        // for (discard_tile_id, ukiere_tile_ids_after_discard, num_ukiere_after_discard) in
        //     best_ukiere_after_discard
        // {
        //     println!(
        //         "discard {} -> {} ukiere tiles: {} ",
        //         mahjong_tile::get_tile_text_from_id(discard_tile_id).unwrap(),
        //         num_ukiere_after_discard,
        //         tile_ids_to_string(&ukiere_tile_ids_after_discard)
        //     );
        // }

        let shanten_ukiere_after_each_discard = get_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );
        print_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &shanten_ukiere_after_each_discard,
            &other_visible_tiles,
        );

        // next turn: after discarding 3p and calling 1z
        println!("check after discard 3p and calling 1z");
        let tiles_after_call = tiles_to_count_array("5789s57p34667m111z");
        let shanten_after_discard =
            get_best_shanten_after_discard(tiles_after_call, &get_shanten_optimized);
        let other_visible_tiles = mahjong_tile::tiles_to_tile_ids("3p");
        let best_ukiere_after_discard = get_most_ukiere_after_discard(
            tiles_after_call,
            shanten_after_discard,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );

        println!("checking if discard ukiere matches...");
        let mut expected_discard_ukiere: Vec<(MahjongTileId, Vec<MahjongTileId>, u16)> = Vec::new();
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("5s").unwrap(),
            mahjong_tile::tiles_to_tile_ids("25m6p"),
            12,
        ));
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("7m").unwrap(),
            mahjong_tile::tiles_to_tile_ids("25m6p"),
            12,
        ));
        assert_discards_ukiere_match(&best_ukiere_after_discard, &expected_discard_ukiere);
        println!("finished checking if discard ukiere matches!");

        let shanten_ukiere_after_each_discard = get_shanten_ukiere_after_each_discard(
            tiles_after_call,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );
        print_shanten_ukiere_after_each_discard(
            tiles_after_call,
            &shanten_ukiere_after_each_discard,
            &other_visible_tiles,
        );

        println!("checking if upgrade tiles matches...");
        // upgrades after discarding 5s
        let mut expected_upgrade_tiles: HashMap<&'static str, HashMap<&'static str, Vec<MahjongTileId>>> =
            HashMap::new();
        expected_upgrade_tiles.insert("6m", hashmap!["7m" => mahjong_tile::tiles_to_tile_ids("2345m567p")]);
        expected_upgrade_tiles.insert("8m", hashmap!["6m" => mahjong_tile::tiles_to_tile_ids("2345m567p")]);
        // the upgrade options below: 34m345789p are not included in the results from this efficiency trainer:
        // https://euophrys.itch.io/mahjong-efficiency-trainer
        // 34667m57p789s111z: 1 shanten, 12 ukiere: 25m6p
        // draw 3m is an upgrade -> cut 4m -> 33667m57p789s111z: 1 shanten, 16 ukiere: 3568m6p
        // or draw 3m -> cut 7m -> 33466m57p789s111z: 1 shanten, 16 ukiere: 2356m6p
        expected_upgrade_tiles.insert(
            "3m",
            hashmap!["4m" => mahjong_tile::tiles_to_tile_ids("3568m6p"), "7m" => mahjong_tile::tiles_to_tile_ids("2356m6p")],
        );
        // draw 4m is an upgrade -> cut 3m -> 44667m57p789s111z: 1 shanten, 16 ukiere: 4568m6p
        // or draw 4m -> cut 7m -> 34466m57p789s111z: 1 shanten, 16 ukiere: 2456m6p
        expected_upgrade_tiles.insert(
            "4m",
            hashmap!["3m" => mahjong_tile::tiles_to_tile_ids("4568m6p"), "7m" => mahjong_tile::tiles_to_tile_ids("2456m6p")],
        );
        // draw 3p is an upgrade (357p ryankan) -> cut 7m -> 3466m357p789s111z: 1 shanten, 16 ukiere: 25m46p
        expected_upgrade_tiles.insert("3p", hashmap!["7m" => mahjong_tile::tiles_to_tile_ids("25m46p")]);
        // draw 4p is an upgrade (57p kanchan -> 45p ryanmen) -> cut 7p -> 34667m45p789s111z: 1 shanten, 16 ukiere: 25m36p
        // or draw 4p -> cut 7m -> 3466m457p789s111z: 1 shanten, 16 ukiere: 256m56p
        expected_upgrade_tiles.insert(
            "4p",
            hashmap!["7p" => mahjong_tile::tiles_to_tile_ids("25m36p"), "7m" => mahjong_tile::tiles_to_tile_ids("25m36p")],
        );
        // draw 5p is an upgrade -> cut 7m -> 3466m557p789s111z: 1 shanten, 16 ukiere: 256m56p
        // or draw 5p -> cut 7p -> 34667m55p789s111z: 1 shanten, 16 ukiere: 2568m5p
        expected_upgrade_tiles.insert(
            "5p",
            hashmap!["7m" => mahjong_tile::tiles_to_tile_ids("256m56p"), "7p" => mahjong_tile::tiles_to_tile_ids("2568m5p")],
        );
        // draw 7p is an upgrade -> cut 7m -> 3466m577p789s111z: 1 shanten, 16 ukiere: 256m67p
        // or draw 7p -> cut 5p -> 34667m77p789s111z: 1 shanten, 16 ukiere: 2568m7p
        expected_upgrade_tiles.insert(
            "7p",
            hashmap!["7m" => mahjong_tile::tiles_to_tile_ids("256m67p"), "5p" => mahjong_tile::tiles_to_tile_ids("2568m7p")],
        );
        // draw 8p is an upgrade (57p kanchan -> 78p ryanmen) -> cut 5p -> 34667m78p789s111z: 1 shanten, 16 ukiere: 25m69p
        // or draw 8p -> cut 7m -> 3466m578p789s111z: 1 shanten, 16 ukiere: 25m69p
        expected_upgrade_tiles.insert(
            "8p",
            hashmap!["5p" => mahjong_tile::tiles_to_tile_ids("25m69p"), "7m" => mahjong_tile::tiles_to_tile_ids("25m69p")],
        );
        // draw 9p is an upgrade (579p ryankan) -> cut 7m -> 3466m579p789s111z: 1 shanten, 16 ukiere: 25m68p
        expected_upgrade_tiles.insert("9p", hashmap!["7m" => mahjong_tile::tiles_to_tile_ids("25m68p")]);

        // calculate the tiles that, if drawn, upgrade the ukiere of the hand at the current shanten
        let tiles_after_cut_5s = remove_tile_id_from_count_array(
            tiles_after_call,
            mahjong_tile::get_id_from_tile_text("5s").unwrap(),
        );
        assert_upgrade_tiles_match(
            tiles_after_cut_5s,
            &expected_upgrade_tiles,
            &other_visible_tiles,
        );
    }

    #[test]
    fn wwyd_tenhou_hand_east1_turn8() {
        // east-1, seat: east, (25k points all), dora indicator: 6s (dora: 7s)
        let tiles_after_draw = tiles_to_count_array("345m11256p46778s6s");
        let shanten_after_discard =
            get_best_shanten_after_discard(tiles_after_draw, &get_shanten_optimized);
        assert_eq!(shanten_after_discard, 1);
        let other_visible_tiles = Vec::new();
        let best_ukiere_after_discard = get_most_ukiere_after_discard(
            tiles_after_draw,
            shanten_after_discard,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );

        let mut expected_discard_ukiere: Vec<(MahjongTileId, Vec<MahjongTileId>, u16)> = Vec::new();
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("4s").unwrap(),
            mahjong_tile::tiles_to_tile_ids("47p58s"),
            15,
        ));
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("2p").unwrap(),
            mahjong_tile::tiles_to_tile_ids("47p58s"),
            15,
        ));
        // discard 7s -> also 1 shanten, but only 12 ukiere: 47p5s
        assert_discards_ukiere_match(&best_ukiere_after_discard, &expected_discard_ukiere);

        // for (discard_tile_id, ukiere_tile_ids_after_discard, num_ukiere_after_discard) in
        //     best_ukiere_after_discard
        // {
        //     println!(
        //         "discard {} -> {} ukiere tiles: {} ",
        //         mahjong_tile::get_tile_text_from_id(discard_tile_id).unwrap(),
        //         num_ukiere_after_discard,
        //         tile_ids_to_string(&ukiere_tile_ids_after_discard)
        //     );
        // }

        let shanten_ukiere_after_each_discard = get_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );
        print_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &shanten_ukiere_after_each_discard,
            &other_visible_tiles,
        );
    }

    #[test]
    fn wwyd_tenhou_hand_east1_turn7() {
        // I think there was a bigger mistake on the previous turn/discard decision in this same hand:
        // 345m11256p46778s6m
        let tiles_after_draw = tiles_to_count_array("345m11256p46778s6m");
        let shanten_after_discard =
            get_best_shanten_after_discard(tiles_after_draw, &get_shanten_optimized);
        assert_eq!(shanten_after_discard, 2);
        let other_visible_tiles = Vec::new();
        let best_ukiere_after_discard = get_most_ukiere_after_discard(
            tiles_after_draw,
            shanten_after_discard,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );
        let mut expected_discard_ukiere: Vec<(MahjongTileId, Vec<MahjongTileId>, u16)> = Vec::new();
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("7s").unwrap(),
            mahjong_tile::tiles_to_tile_ids("12345678m12347p234569s"),
            67,
        ));
        assert_discards_ukiere_match(&best_ukiere_after_discard, &expected_discard_ukiere);

        let shanten_ukiere_after_each_discard = get_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );
        print_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &shanten_ukiere_after_each_discard,
            &other_visible_tiles,
        );
    }

    #[test]
    fn wwyd_solitaire_hand_turn7() {
        // dora indicator: 5m (dora 6m) - discarded so far: 3z5z7z1z4z9p
        let tiles_after_draw = tiles_to_count_array("46p255567s33478m4s");
        let shanten_after_discard =
            get_best_shanten_after_discard(tiles_after_draw, &get_shanten_optimized);
        assert_eq!(shanten_after_discard, 2);
        let other_visible_tiles = Vec::new();
        let best_ukiere_after_discard = get_most_ukiere_after_discard(
            tiles_after_draw,
            shanten_after_discard,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );

        let mut expected_discard_ukiere: Vec<(MahjongTileId, Vec<MahjongTileId>, u16)> = Vec::new();
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("2s").unwrap(),
            mahjong_tile::tiles_to_tile_ids("5p3568s23569m"),
            34,
        ));
        // expected_discard_ukiere.push((
        //     mahjong_tile::get_id_from_tile_text("4p").unwrap(),
        //     mahjong_tile::tiles_to_tile_ids("358s23569m"),
        //     27,
        // ));
        // expected_discard_ukiere.push((
        //     mahjong_tile::get_id_from_tile_text("6p").unwrap(),
        //     mahjong_tile::tiles_to_tile_ids("358s23569m"),
        //     27,
        // ));
        assert_discards_ukiere_match(&best_ukiere_after_discard, &expected_discard_ukiere);

        let shanten_ukiere_after_each_discard = get_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );
        print_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &shanten_ukiere_after_each_discard,
            &other_visible_tiles,
        );
    }

    #[test]
    fn wwyd_tenhou_hand2_east1_turn1() {
        // 4557m2307p23067s6p - dora indicator 8s (I'm sitting west, dealer discarded 2z, south discarded 3z, and I just drew 6p -> 1 shanten)
        // TODO parse red fives
        // The main options are cut 4m vs cut 7m. Both lead to 1-shanten waiting on 14s14p, with upgrades to headless 1-shanten.
        // In game, I cut 4m to try to chase 567 sanshoku.
        // What is the loss in efficiency? Cutting 7m (instead of 4m) means my hand can also upgrade on 3m (in addition to 56m -> headless 1-shanten and 23s23p -> complete 1-shanten)
        // What is the value-speed tradeoff? My hand already has 2 dora locked in - the remaining potential yaku are:
        // tanyao (which isn't guaranteed due to the two 23 ryanmen shapes), pinfu (which is more likely but prevents the hand from calling),
        // and 567 sanshoku (but calling 6m to complete 567m leads to headless 1-shanten, which could result in tanki wait at tenpai).
        // -> can we open the hand at all? if we discard 4m and call 6m, that locks in sanshoku. But besides that, there isn't a completely
        // safe option to call and guarantee yaku. For example, if I discard 4m or 7m and then decide to call on 4s, the 23p shape
        // means I can't ron on 1p (and drawing 1p myself is worse: I don't have yaku since I opened the hand and I will be in furiten)
        // -> so if we can't safely open the hand (unless we can call on 6m after cut 4m), is that extra han worth it to not be able to upgrade on 3m?
        // with the call on 6m, the value is 3 han (and could be tanki wait)
        // but without call, the value could be 3-4 hand (with riichi and pinfu) and the wait could still be a tanki wait
        // the other thing is: it's so early in the hand, it seems unlikely that kamicha (player to my left) would discard a 6m for me to call
        // -> overall, I think cutting 7m is better - I cannot guarantee yaku if I open the hand (unless I specifically call 6m for sanshoku,
        // and kamicha probably shouldn't drop 6m this early into the hand)
        let tiles_after_draw = tiles_to_count_array("4557m2357p23567s6p");
        let shanten_after_discard =
        get_best_shanten_after_discard(tiles_after_draw, &get_shanten_optimized);
        assert_eq!(shanten_after_discard, 1);
        let other_visible_tiles = Vec::new();
        let best_ukiere_after_discard = get_most_ukiere_after_discard(
            tiles_after_draw,
            shanten_after_discard,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );
        let mut expected_discard_ukiere: Vec<(MahjongTileId, Vec<MahjongTileId>, u16)> = Vec::new();
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("4m").unwrap(),
            mahjong_tile::tiles_to_tile_ids("14p14s"),
            16,
        ));
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("7m").unwrap(),
            mahjong_tile::tiles_to_tile_ids("14p14s"),
            16,
        ));
        assert_discards_ukiere_match(&best_ukiere_after_discard, &expected_discard_ukiere);

        let shanten_ukiere_after_each_discard = get_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );
        print_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &shanten_ukiere_after_each_discard,
            &other_visible_tiles,
        );
    }


    #[test]
    fn wwyd_tenhou_hand3_east4_turn6() {
        // 2246778m234677p5p - dora indicator 2m (I'm sitting east, just drew 5p -> kutsuki 1-shanten, with 47m7p as floating tiles)
        // self  discards: 3z7z1m9s4p
        // south discards: 4z1p1s2p9p
        // west  discards: 9m8m9s6m2s
        // north discards: 3p8m5m8s2s
        let tiles_after_draw = tiles_to_count_array("2246778m234677p5p");
        let shanten_after_discard =
        get_best_shanten_after_discard(tiles_after_draw, &get_shanten_optimized);
        assert_eq!(shanten_after_discard, 1);
        let other_visible_tiles = Vec::new();
        let best_ukiere_after_discard = get_most_ukiere_after_discard(
            tiles_after_draw,
            shanten_after_discard,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );
        let mut expected_discard_ukiere: Vec<(MahjongTileId, Vec<MahjongTileId>, u16)> = Vec::new();
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("7m").unwrap(),
            mahjong_tile::tiles_to_tile_ids("234569m1456789p"),
            43,
        ));
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("4m").unwrap(),
            mahjong_tile::tiles_to_tile_ids("256789m1456789p"),
            41,
        ));
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("7p").unwrap(),
            mahjong_tile::tiles_to_tile_ids("23456789m"),
            25,
        ));
        expected_discard_ukiere.push((
            mahjong_tile::get_id_from_tile_text("2m").unwrap(),
            mahjong_tile::tiles_to_tile_ids("37m147p"),
            15,
        ));
        assert_discards_ukiere_match(&best_ukiere_after_discard, &expected_discard_ukiere);

        let shanten_ukiere_after_each_discard = get_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &get_shanten_optimized,
            &get_ukiere_optimized,
            &other_visible_tiles,
        );
        print_shanten_ukiere_after_each_discard(
            tiles_after_draw,
            &shanten_ukiere_after_each_discard,
            &other_visible_tiles,
        );
    }


    // hand: "234678s2345577p6z" - tenpai (57p shanpon) - but how many upgrade tiles? and how much value does it add?
}

// 3445799m13p3456s4m - 1-shanten, cut 3s/6s results in 15 ukiere (4689m2p)
// 345m11256p466778s - 1-shanten, should cut 4s (holding 2p gives more upgrade opportunities with 13p draw)

// from riichi book 1, section 6.3.3 (advanced scoring > examples)
// 22345m45567p777z -> tenpai on 36p
// win by ron on 3p or 6p (as non-dealer): 1 han 40 fu -> 1300
// win by tsumo on 3p (as non-dealer): 2 han 30 fu (+8 from ankou of 777z, +2 tsumo) -> 500/1000
// win by tsumo on 6p (as non-dealer): 2 han 40 fu (additional +2 from kanchan wait 456p-57p) -> 700/1300

// example in riichi book 1, section 3.3.1 (three-tile complex shapes > double closed (ryankan) shape)
// 455789m45667p77s2p
// discard 2p -> 1-shanten, accepts 356m58p7s (19 tiles)
// discard 5m -> 1-shanten, accepts 36m358p (19 tiles)
// discarding 5m means we have more options to reach pinfu (if discard 2p and then draw 5m or 7s, it would form an ankou -> not eligible for pinfu)

// 11345m678p246s77z2m
// should cut 2s (or 6s) - but why?
