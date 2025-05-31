pub use crate::mahjong_hand;
pub use crate::mahjong_tile;
use std::cmp::max;
use std::collections::VecDeque;

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
                    MeldType::Triplet
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
            MeldType::Pair | MeldType::SingleTile => {
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

impl HandInterpretation {
    fn num_tiles(&self) -> u8 {
        let mut total_num_tiles = 0;
        for &tile_count in self.total_tile_count_array.iter() {
            total_num_tiles += tile_count;
        }
        total_num_tiles
    }

    fn get_standard_shanten(&self) -> i8 {
        if self.num_tiles() != 13 {
            // TODO eventually will need to handle the case when there are more tiles due to quads
            panic!("invalid number of tiles")
        }
        // compute standard shanten: count complete groups, incomplete groups, and pairs
        let mut num_complete_groups = 0;
        let mut num_incomplete_groups = 0; // this is taatsu + pairs
        let mut has_pair = false;
        for group in self.groups.iter() {
            if group.is_complete() {
                num_complete_groups += 1;
            }
            if !group.is_complete() && group.tile_ids.len() == 2 {
                num_incomplete_groups += 1; // note that this includes taatsu and pairs!
            }
            if group.meld_type == MeldType::Pair {
                has_pair = true;
            }
        }
        // first, only count up to 4 groups (either complete or incomplete)
        let mut shanten = 8;
        shanten -= 2 * num_complete_groups;
        shanten -= max(num_incomplete_groups, 4 - num_complete_groups);
        // then reduce by 1 if there is a pair and at least 5 groups (one of the pairs can count towards the 5)
        if has_pair && num_complete_groups + num_incomplete_groups >= 5 {
            shanten -= 1;
        }
        shanten
    }

    fn get_ukiere(&self) -> Vec<u8> {
        if self.num_tiles() != 13 {
            // TODO eventually will need to handle the case when there are more tiles due to quads
            panic!("invalid number of tiles")
        }
        let mut ukiere_tile_ids = Vec::new();
        for group in &self.groups {
            let tile_ids = group.tile_ids_to_complete_group();
            for &tile_id in tile_ids.iter() {
                if ukiere_tile_ids.contains(&tile_id) {
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

pub fn get_suit_melds(suit_tile_count_array: [u8; 34]) -> Vec<Vec<TileMeld>> {
    let mut meld_interpretations = Vec::new();
    let mut queue: VecDeque<PartialMeldInterpretation> = VecDeque::new();
    queue.push_back(PartialMeldInterpretation {
        remaining_tile_count_array: suit_tile_count_array,
        groups: Vec::new(),
    });

    while !queue.is_empty() {
        let partial_interpretation = queue.pop_front().unwrap();
        let tile_count_array = partial_interpretation.remaining_tile_count_array;

        // find the first tile id that is not empty
        // TODO we could be more clever - considering each suit separately, for example
        let mut tile_id = 0u8;
        while tile_id < mahjong_tile::FIRST_HONOR_ID {
            let tile_idx = usize::from(tile_id);
            if tile_count_array[tile_idx] == 0 {
                tile_id += 1;
                continue;
            }
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

            let mut new_state_after_pair = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id]);
            new_state_after_pair.remaining_tile_count_array[tile_idx] = num_tile_count - 2;
            new_state_after_pair.groups.push(tile_meld);
            queue.push_back(new_state_after_pair);

            continue;
        }

        if tile_count_array[tile_idx] == 2 {
            // break out a pair and then let it continue trying to add as a single
            let mut new_state_after_pair = partial_interpretation.clone();
            let tile_meld = TileMeld::new(vec![tile_id, tile_id]);
            new_state_after_pair.remaining_tile_count_array[tile_idx] = num_tile_count - 2;
            new_state_after_pair.groups.push(tile_meld);
            queue.push_back(new_state_after_pair);
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
    get_ukiere_helper(&interpretations)
}

pub fn get_ukiere_helper(hand_interpretations: &Vec<HandInterpretation>) -> Vec<u8> {
    let mut ukiere_tiles = Vec::new();
    for interpretation in hand_interpretations.iter() {
        let new_tile_ids = interpretation.get_ukiere();
        for &tile_id in new_tile_ids.iter() {
            if !ukiere_tiles.contains(&tile_id) {
                ukiere_tiles.push(tile_id);
            }
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
                println!("found tile {}{}", rank_char, char);
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn hand_two_shanten_and_ukiere() {
        let tiles = tiles_to_count_array("46p455567s33478m");
        // hand is 2-shanten: 46p - 455s - 567s - 334m - 78m
        assert_eq!(get_shanten(tiles), 2);

        // ukiere tiles: 5p3568s23569m
        let ukiere_tiles = get_ukiere(tiles);
        assert_eq!(ukiere_tiles.len(), 10);
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("5p").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("3s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("5s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("6s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("8s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("2m").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("3m").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("5m").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("6m").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("9m").unwrap()));
    }

    #[test]
    fn hand_one_shanten_and_ukiere() {
        let tiles = tiles_to_count_array("233445m56p4455s7z");
        // hand is 1-shanten: 234m - 345m - 56p - 44s - 55s - 7z
        assert_eq!(get_shanten(tiles), 1);

        // ukiere tiles: 47p45s
        let ukiere_tiles = get_ukiere(tiles);
        assert_eq!(ukiere_tiles.len(), 4);
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("4p").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("7p").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("4s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("5s").unwrap()));

        let tiles = tiles_to_count_array("56m23346778p234s");
        // hand is 1-shanten: 56m - 234p - 678p - 3p - 7p - 234s
        assert_eq!(get_shanten(tiles), 1);

        // ukiere tiles: 47m37p
        let ukiere_tiles = get_ukiere(tiles);
        assert_eq!(ukiere_tiles.len(), 4);
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("4m").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("7m").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("3p").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("7p").unwrap()));
    }

    #[test]
    fn hand_headless_one_shanten_with_ankou() {
        let tiles = tiles_to_count_array("23s678s56p888p888m");
        // hand is 1-shanten: 23s - 678s - 56p - 888p - 888m
        assert_eq!(get_shanten(tiles), 1);

        // ukiere tiles: 1234s4567p
        let ukiere_tiles = get_ukiere(tiles);
        assert_eq!(ukiere_tiles.len(), 8);
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("1s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("2s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("3s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("4s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("4p").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("5p").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("6p").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("7p").unwrap()));
    }

    #[test]
    fn complex_souzu_one_shanten() {
        let tiles = tiles_to_count_array("12234455s345p11z");
        // hand is 1-shanten: 123s - 24s - 455s - 345p - 11z
        assert_eq!(get_shanten(tiles), 1);

        // ukiere tiles: 23456s1z
        let ukiere_tiles = get_ukiere(tiles);
        assert_eq!(ukiere_tiles.len(), 6);
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("2s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("3s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("4s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("5s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("6s").unwrap()));
        assert!(ukiere_tiles.contains(&mahjong_tile::get_id_from_tile_text("1z").unwrap()));
    }
}
