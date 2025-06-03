pub use crate::mahjong_hand;
pub use crate::mahjong_tile;
use std::cmp::min;
use std::collections::VecDeque;
use std::fmt;
use std::ops::Index;

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
    tile_ids: Vec<u8>,
}

fn tile_ids_are_all_same(tile_ids: &Vec<u8>) -> bool {
    if tile_ids.len() == 0 {
        return true;
    }
    let first_tile_id = tile_ids.get(0).unwrap();
    for &tile_id in tile_ids.iter() {
        if tile_id != *first_tile_id {
            return false;
        }
    }
    true
}

fn tile_ids_are_sequence(tile_ids: &Vec<u8>) -> bool {
    if tile_ids.len() != 3 {
        return false;
    }
    let mut sorted_tile_ids = tile_ids.clone();
    sorted_tile_ids.sort();
    let min_tile_id = *sorted_tile_ids.get(0).unwrap();
    let mid_tile_id = *sorted_tile_ids.get(1).unwrap();
    let max_tile_id = *sorted_tile_ids.get(2).unwrap();
    // all tiles must not be honor tiles
    if max_tile_id >= mahjong_tile::FIRST_HONOR_ID {
        return false;
    }

    let min_tile_rank = mahjong_tile::get_num_tile_rank(min_tile_id).unwrap();
    let max_tile_rank = mahjong_tile::get_num_tile_rank(max_tile_id).unwrap();
    // tile ids must be sequential, but cannot wrap around the ends of a suit
    (max_tile_id == mid_tile_id + 1)
        && (mid_tile_id == min_tile_id + 1)
        && (min_tile_rank <= 7 && max_tile_rank >= 3)
}

fn tile_ids_are_ryanmen(tile_ids: &Vec<u8>) -> bool {
    if tile_ids.len() != 2 {
        return false;
    }
    let mut sorted_tile_ids = tile_ids.clone();
    sorted_tile_ids.sort();
    let min_tile_id = *sorted_tile_ids.get(0).unwrap();
    let max_tile_id = *sorted_tile_ids.get(1).unwrap();
    // all tiles must not be honor tiles
    if max_tile_id >= mahjong_tile::FIRST_HONOR_ID {
        return false;
    }

    let min_tile_rank = mahjong_tile::get_num_tile_rank(min_tile_id).unwrap();
    max_tile_id == min_tile_id + 1 && (min_tile_rank >= 2 && min_tile_rank <= 7)
}

fn tile_ids_are_kanchan(tile_ids: &Vec<u8>) -> bool {
    if tile_ids.len() != 2 {
        return false;
    }
    let mut sorted_tile_ids = tile_ids.clone();
    sorted_tile_ids.sort();
    let min_tile_id = *sorted_tile_ids.get(0).unwrap();
    let max_tile_id = *sorted_tile_ids.get(1).unwrap();
    // all tiles must not be honor tiles
    if max_tile_id >= mahjong_tile::FIRST_HONOR_ID {
        return false;
    }

    let min_tile_rank = mahjong_tile::get_num_tile_rank(min_tile_id).unwrap();
    let max_tile_rank = mahjong_tile::get_num_tile_rank(max_tile_id).unwrap();
    max_tile_id == min_tile_id + 2 && (min_tile_rank <= 7 && max_tile_rank >= 3)
}

fn tile_ids_are_penchan(tile_ids: &Vec<u8>) -> bool {
    if tile_ids.len() != 2 {
        return false;
    }
    let mut sorted_tile_ids = tile_ids.clone();
    sorted_tile_ids.sort();
    let min_tile_id = *sorted_tile_ids.get(0).unwrap();
    let max_tile_id = *sorted_tile_ids.get(1).unwrap();
    // all tiles must not be honor tiles
    if max_tile_id >= mahjong_tile::FIRST_HONOR_ID {
        return false;
    }

    let min_tile_rank = mahjong_tile::get_num_tile_rank(min_tile_id).unwrap();
    max_tile_id == min_tile_id + 1 && (min_tile_rank == 1 || min_tile_rank == 8)
}

impl TileMeld {
    fn new(tile_ids: Vec<u8>) -> Self {
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
        let mut sorted_tile_ids = tile_ids.clone();
        sorted_tile_ids.sort();
        TileMeld {
            meld_type: meld_type,
            tile_ids: sorted_tile_ids,
        }
    }

    fn is_complete(&self) -> bool {
        self.meld_type.is_complete()
    }

    fn tile_ids_to_complete_group(&self) -> Vec<u8> {
        // assumes the tile_ids are sorted
        match self.meld_type {
            MeldType::SingleTile => {
                let tile_id = self.tile_ids.get(0).unwrap();
                let tile_id = *tile_id;
                let mut tile_ids = vec![tile_id];
                match mahjong_tile::get_num_tile_rank(tile_id) {
                    Some(tile_rank) => {
                        if tile_rank <= 8 {
                            tile_ids.push(tile_id + 1);
                        }
                        if tile_rank <= 7 {
                            tile_ids.push(tile_id + 2);
                        }
                        if tile_rank >= 2 {
                            tile_ids.push(tile_id - 1);
                        }
                        if tile_rank >= 3 {
                            tile_ids.push(tile_id - 2);
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
                vec![*min_tile_id - 1, *max_tile_id + 1]
            }
            MeldType::Kanchan => {
                let min_tile_id = self.tile_ids.get(0).unwrap();
                vec![*min_tile_id + 1]
            }
            MeldType::Penchan => {
                let min_tile_id = self.tile_ids.get(0).unwrap();
                let min_tile_rank = mahjong_tile::get_num_tile_rank(*min_tile_id).unwrap();
                if min_tile_rank == 1 {
                    vec![*min_tile_id + 2]
                } else if min_tile_rank == 8 {
                    vec![*min_tile_id - 1]
                } else {
                    panic!("invalid penchan! min tile rank should be 1 or 8")
                }
            }
            t if t.is_complete() => Vec::new(),
            _ => panic!("unexpected meld type"),
        }
    }
}

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

impl HandInterpretation {
    fn num_tiles(&self) -> u8 {
        let mut total_num_tiles = 0;
        for &tile_count in self.total_tile_count_array.iter() {
            total_num_tiles += tile_count;
        }
        total_num_tiles
    }

    fn get_num_complete_groups(&self) -> i8 {
        let mut num_complete_groups = 0;
        for group in self.groups.iter() {
            if group.is_complete() {
                num_complete_groups += 1;
            }
        }
        num_complete_groups
    }

    fn get_num_incomplete_groups(&self) -> i8 {
        let mut num_incomplete_groups = 0; // this is taatsu + pairs
        for group in self.groups.iter() {
            if !group.is_complete() && group.tile_ids.len() == 2 {
                // exclude single / isolated tiles, as incomplete groups means it only
                // needs one more tile to become a complete group
                num_incomplete_groups += 1; // note that this includes taatsu and pairs!
            }
        }
        num_incomplete_groups
    }

    fn get_pair_tile_ids(&self) -> Vec<u8> {
        let mut pair_tile_ids = vec![];
        for group in self.groups.iter() {
            if group.meld_type == MeldType::Pair {
                let tile_id = group.tile_ids.get(0).unwrap();
                pair_tile_ids.push(*tile_id);
            }
        }
        pair_tile_ids
    }

    fn get_single_tile_ids(&self) -> Vec<u8> {
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

        let mut shanten = 8i8;
        // first, only count up to 4 groups (either complete or incomplete)
        shanten -= 2 * num_complete_groups;
        shanten -= min(num_incomplete_groups, 4 - num_complete_groups);
        // then reduce by 1 if there is a pair and at least 5 groups (one of the pairs can count towards the 5)
        if has_pair && num_complete_groups + num_incomplete_groups >= 5 {
            shanten -= 1;
        }

        // println!(
        //     "hand interpretation {} has {} complete groups, {} incomplete groups, has_pair={} => shanten = {}",
        //     self, num_complete_groups, num_incomplete_groups, has_pair, shanten
        // );
        shanten
    }

    fn get_ukiere(&self) -> Vec<u8> {
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

    // then, for each numbered suit (manzu, pinzu, souzu), recursively build subgroups
    // let mut manzu_suit_tile_counts: [u8; 9] = [0; 9];
    // let mut pinzu_suit_tile_counts: [u8; 9] = [0; 9];
    // let mut souzu_suit_tile_counts: [u8; 9] = [0; 9];

    // let manzu_meld_interpretations = get_suit_melds(manzu_suit_tile_counts);
    // let pinzu_meld_interpretations = get_suit_melds(pinzu_suit_tile_counts);
    // let souzu_meld_interpretations = get_suit_melds(souzu_suit_tile_counts);
    // combine all possible meld-combinations from each suit (and the honors)
    // let mut hand_interpretations = Vec::new();
    // for &manzu_melds in manzu_meld_interpretations.iter() {
    //     for &pinzu_melds in pinzu_meld_interpretations.iter() {
    //         for &souzu_melds in souzu_meld_interpretations.iter() {
    //             let mut all_melds = honor_tile_melds.clone();
    //             all_melds.extend(manzu_meld);
    //             all_melds.extend(pinzu_meld);
    //             all_melds.extend(souzu_meld);

    //             let hand_interpretation = HandInterpretation {
    //                 total_tile_count_array: tile_count_array,
    //                 groups: all_melds,
    //             };
    //             hand_interpretations.push(hand_interpretation);
    //         }
    //     }
    // }

    let mut hand_interpretations = Vec::new();
    let meld_interpretations = get_suit_melds(updated_tile_count_array);
    for melds in meld_interpretations {
        // iterate on the Vec directly to consume the vector
        let mut all_melds = honor_tile_melds.clone();
        all_melds.extend(melds);
        let hand_interpretation = HandInterpretation {
            total_tile_count_array: original_tile_count_array,
            groups: all_melds,
        };
        hand_interpretations.push(hand_interpretation);
    }

    hand_interpretations
}

#[derive(Clone)]
struct PartialMeldInterpretation {
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

pub fn get_suit_melds(suit_tile_count_array: [u8; 34]) -> Vec<Vec<TileMeld>> {
    let mut meld_interpretations = Vec::new();
    let mut queue: VecDeque<PartialMeldInterpretation> = VecDeque::new();
    queue.push_back(PartialMeldInterpretation {
        remaining_tile_count_array: suit_tile_count_array,
        groups: Vec::new(),
    });

    while !queue.is_empty() {
        let partial_interpretation = queue.pop_front().unwrap();
        // println!("current state: {}", partial_interpretation);

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
            meld_interpretations.push(partial_interpretation.groups);
            continue;
        }

        if tile_count_array[tile_idx] >= 3 {
            // break out a triplet or a pair
            let mut new_state_after_triplet = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id, tile_id]);
            new_state_after_triplet.remaining_tile_count_array[tile_idx] = num_tile_count - 3;
            new_state_after_triplet.groups.push(tile_meld);
            queue.push_back(new_state_after_triplet);
            // println!(
            //     "will recursively try forming a triplet from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );

            let mut new_state_after_pair = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id]);
            new_state_after_pair.remaining_tile_count_array[tile_idx] = num_tile_count - 2;
            new_state_after_pair.groups.push(tile_meld);
            queue.push_back(new_state_after_pair);
            // println!(
            //     "will recursively try forming a pair from {}",
            //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            // );

            continue;
        }

        if tile_count_array[tile_idx] == 2 {
            // break out a pair and then let it continue trying to add as a single
            let mut new_state_after_pair = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id]);
            new_state_after_pair.remaining_tile_count_array[tile_idx] = num_tile_count - 2;
            new_state_after_pair.groups.push(tile_meld);
            queue.push_back(new_state_after_pair);
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
            queue.push_back(new_state_after_sequence);
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
            queue.push_back(new_state_after_ryanmen);
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
            queue.push_back(new_state_after_penchan);
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
            queue.push_back(new_state_after_kanchan);
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
        queue.push_back(new_state_after_isolated);
        // println!(
        //     "will recursively try forming an isolated tile from {}",
        //     mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
        // );
    }

    meld_interpretations
}

pub fn get_shanten(tile_count_array: [u8; 34]) -> i8 {
    let interpretations = get_hand_interpretations(tile_count_array);
    // TODO also include chiitoitsu shanten (6 - (# tile counts >= 2)) and kokushi shanten
    get_shanten_helper(&interpretations)
}

pub fn get_shanten_helper(hand_interpretations: &Vec<HandInterpretation>) -> i8 {
    hand_interpretations
        .iter()
        .map(|i| i.get_standard_shanten())
        .min()
        .unwrap()
}

pub fn get_ukiere(tile_count_array: [u8; 34]) -> Vec<u8> {
    let interpretations = get_hand_interpretations(tile_count_array);
    let shanten = get_shanten_helper(&interpretations);
    get_ukiere_helper(&interpretations, shanten)
}

pub fn get_ukiere_helper(hand_interpretations: &Vec<HandInterpretation>, shanten: i8) -> Vec<u8> {
    let mut ukiere_tiles = Vec::new();
    println!("looking for hand interpretations with shanten {}", shanten);
    for interpretation in hand_interpretations.iter() {
        if interpretation.get_standard_shanten() != shanten {
            continue;
        }

        println!(
            "finding ukiere tiles for hand interpretation {} with shanten {}",
            interpretation, shanten
        );
        let new_tile_ids = interpretation.get_ukiere();
        for &tile_id in new_tile_ids.iter() {
            print!(
                "ukiere tile: {}",
                mahjong_tile::get_tile_text_from_id(tile_id).unwrap()
            );
            if !ukiere_tiles.contains(&tile_id) {
                ukiere_tiles.push(tile_id);
            }
            print!("\n");
        }
    }
    ukiere_tiles
}

pub fn get_chiitoi_shanten(_tile_count_array: [u8; 34]) -> i8 {
    todo!()
}

pub fn get_kokushi_shanten(_tile_count_array: [u8; 34]) -> i8 {
    todo!()
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

pub fn tiles_to_tile_ids(tiles_string: &str) -> Vec<u8> {
    let mut tile_ids = Vec::new();
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
                tile_ids.push(tile_id);
            }
            rank_chars = Vec::new();
        } else {
            rank_chars.push(char);
        }
    }
    tile_ids
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_tile_ids_match(tile_ids: &Vec<u8>, expected_tile_ids: &Vec<u8>) {
        assert_eq!(tile_ids.len(), expected_tile_ids.len());
        let mut sorted_tile_ids = tile_ids.clone();
        sorted_tile_ids.sort();
        let mut sorted_expected_tile_ids = expected_tile_ids.clone();
        sorted_expected_tile_ids.sort();
        assert_eq!(sorted_tile_ids, sorted_expected_tile_ids);
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
        let tile_ids = tiles_to_tile_ids("1234m");
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
        let tile_ids = tiles_to_tile_ids("46p255567s33478m4s");
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
        let expected_ukiere_tile_ids = tiles_to_tile_ids("123m");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let single_middle_tile =
            TileMeld::new(vec![mahjong_tile::get_id_from_tile_text("8p").unwrap()]);
        let ukiere_tile_ids = single_middle_tile.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = tiles_to_tile_ids("6789p");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let single_middle_tile =
            TileMeld::new(vec![mahjong_tile::get_id_from_tile_text("3s").unwrap()]);
        let ukiere_tile_ids = single_middle_tile.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = tiles_to_tile_ids("12345s");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let single_honor_tile =
            TileMeld::new(vec![mahjong_tile::get_id_from_tile_text("6z").unwrap()]);
        let ukiere_tile_ids = single_honor_tile.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = tiles_to_tile_ids("6z");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let pair = TileMeld::new(vec![
            mahjong_tile::get_id_from_tile_text("2z").unwrap(),
            mahjong_tile::get_id_from_tile_text("2z").unwrap(),
        ]);
        let ukiere_tile_ids = pair.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = tiles_to_tile_ids("2z");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let ryanmen = TileMeld::new(vec![
            mahjong_tile::get_id_from_tile_text("3p").unwrap(),
            mahjong_tile::get_id_from_tile_text("4p").unwrap(),
        ]);
        let ukiere_tile_ids = ryanmen.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = tiles_to_tile_ids("25p");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let kanchan = TileMeld::new(vec![
            mahjong_tile::get_id_from_tile_text("7m").unwrap(),
            mahjong_tile::get_id_from_tile_text("9m").unwrap(),
        ]);
        let ukiere_tile_ids = kanchan.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = tiles_to_tile_ids("8m");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);

        let penchan = TileMeld::new(vec![
            mahjong_tile::get_id_from_tile_text("8p").unwrap(),
            mahjong_tile::get_id_from_tile_text("9p").unwrap(),
        ]);
        let ukiere_tile_ids = penchan.tile_ids_to_complete_group();
        let expected_ukiere_tile_ids = tiles_to_tile_ids("7p");
        assert_tile_ids_match(&ukiere_tile_ids, &expected_ukiere_tile_ids);
    }

    #[test]
    fn hand_two_shanten_and_ukiere() {
        let tiles = tiles_to_count_array("46p455567s33478m");
        // hand is 2-shanten: 46p - 455s - 567s - 334m - 78m
        assert_eq!(get_shanten(tiles), 2);

        // ukiere tiles: 5p3568s23569m
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("5p3568s23569m");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn hand_one_shanten_and_ukiere() {
        let tiles = tiles_to_count_array("56m23346778p234s");
        // hand is 1-shanten: 56m - 234p - 678p - 3p - 7p - 234s
        assert_eq!(get_shanten(tiles), 1);

        // ukiere tiles: 47m37p
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("47m37p");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn complex_souzu_one_shanten() {
        let tiles = tiles_to_count_array("12234455s345p11z");
        // hand is 1-shanten: 123s - 24s - 455s - 345p - 11z
        assert_eq!(get_shanten(tiles), 1);

        // ukiere tiles: 23456s1z
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("23456s1z");
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

        // ukiere tiles: 47p45s
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("47p45s");
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

        // ukiere tiles: 12469s2p
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("12469s2p");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn hand_headless_one_shanten_with_ankou() {
        // headless 1-shanten is characterized by 3 complete groups + no pair (must have at least 1 incomplete group)
        // https://riichi.wiki/Iishanten#Atamanashi
        let tiles = tiles_to_count_array("23s678s56p888p888m");
        // hand is 1-shanten: 23s - 678s - 56p - 888p - 888m
        assert_eq!(get_shanten(tiles), 1);

        // ukiere tiles: completing either ryanmen group or pairing a tile in the ryanmen group
        // total: 1234s4567p
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("1234s4567p");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn hand_kutsuki_one_shanten() {
        // kutsuki 1-shanten is characterized by 3 complete groups + 1 pair: https://riichi.wiki/Iishanten#Kuttsuki
        let tiles = tiles_to_count_array("2344567m556p678s");
        // hand is 1-shanten: 234m - 456m - 7m - 556p - 678s (but the 4m could be considered floating as well, or as part of 2344m shape)
        assert_eq!(get_shanten(tiles), 1);

        // ukiere tiles: anything within 2 of a floating tile (4m7m6p), there are also
        // the manzu complex shape can accept 147m (2344m-567m or 234m-4567m), 2358m (24m-34567m)
        // total: 123456789m45678p
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("123456789m45678p");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn complete_hand_shanten() {
        let tiles = tiles_to_count_array("55588m234s11p666z1p");
        assert_eq!(get_shanten(tiles), -1);

        let ukiere_tiles = get_ukiere(tiles);
        assert_eq!(ukiere_tiles.len(), 0);
    }

    #[test]
    fn complete_honitsu_hand_shanten() {
        let tiles = tiles_to_count_array("3335577899m111z8m");
        assert_eq!(get_shanten(tiles), -1);

        let ukiere_tiles = get_ukiere(tiles);
        assert_eq!(ukiere_tiles.len(), 0);
    }

    #[test]
    fn six_block_hand_shanten() {
        // example from: https://pathofhouou.blogspot.com/2019/05/calculating-shanten-and-ukeire.html
        let tiles = tiles_to_count_array("12346m6799p1268s");
        assert_eq!(get_shanten(tiles), 2);

        // ukiere tiles: 5m58p37s
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("5m58p37s");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn three_shanten_hand() {
        // example from: https://pathofhouou.blogspot.com/2019/05/calculating-shanten-and-ukeire.html
        let tiles = tiles_to_count_array("12588m27789p889s");
        assert_eq!(get_shanten(tiles), 3);

        // ukiere tiles: 345678m123456789p6789s
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("345678m123456789p6789s");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_tanki_wait() {
        let tiles = tiles_to_count_array("123999m5558p666z");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 8p
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("8p");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_kanchan_wait() {
        let tiles = tiles_to_count_array("13p456777999s33z");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 2p
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("2p");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_penchan_wait() {
        let tiles = tiles_to_count_array("12777m345p67899s");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 3m
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("3m");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_shanpon_wait() {
        let tiles = tiles_to_count_array("123999m44p99s777z");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 4p9s
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("4p9s");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_ryanmen_wait() {
        let tiles = tiles_to_count_array("666m78p666789s22z");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 69p
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("69p");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_nobetan_wait() {
        //e.g. 3456p - https://riichi.wiki/Nobetan
        let tiles = tiles_to_count_array("777m3456p555666s");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 36p
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("36p");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_aryanmen_wait() {
        // e.g. 3455s - https://riichi.wiki/Aryanmen
        let tiles = tiles_to_count_array("567m123456p3455s");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 25s
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("25s");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_kantan_wait() {
        // 2-sided wait e.g. 6888m - https://riichi.wiki/Ryantan#Kantan
        let tiles = tiles_to_count_array("2226888m444p111z");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 67m
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("67m");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_pentan_wait() {
        // 2-sided wait e.g. 1222m - https://riichi.wiki/Kantan#Pentan
        let tiles = tiles_to_count_array("1222m678p345789s");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 13m
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("13m");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_standard_sanmenchan_wait() {
        // 3-sided wait e.g. 34567s - https://riichi.wiki/Sanmenchan
        let tiles = tiles_to_count_array("666m33678p34567s");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 258s
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("258s");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_ryantan_wait() {
        // 3-sided wait e.g. 5556m - https://riichi.wiki/Kantan#Ryantan
        let tiles = tiles_to_count_array("5556m234789p666z");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 467m
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("467m");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_entotsu_wait() {
        // 3-sided wait e.g. 11m45666s - https://riichi.wiki/Sanmenchan#Entotsu
        let tiles = tiles_to_count_array("11m222456p45666s");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 1m36s
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("1m36s");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }

    #[test]
    fn tenpai_sanmentan_wait() {
        // 3-sided wait e.g. 2345678p -  https://riichi.wiki/Sanmenchan#Sanmentan
        let tiles = tiles_to_count_array("233445m2345678p");
        assert_eq!(get_shanten(tiles), 0);

        // ukiere tiles: 258p
        let ukiere_tiles = get_ukiere(tiles);
        let expected_ukiere_tiles = tiles_to_tile_ids("258p");
        assert_tile_ids_match(&ukiere_tiles, &expected_ukiere_tiles);
    }
}
