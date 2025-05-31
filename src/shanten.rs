pub use crate::mahjong_tile;
use std::cmp::max;

#[derive(Clone, Copy, PartialEq)]
enum MeldType {
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
struct TileMeld {
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
                let min_tile_rank = mahjong_tile::get_num_tile_rank(min_tile_id);
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

struct HandInterpretation {
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
        if num_complete_groups + num_incomplete_groups >= 5 {
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
        for group in self.groups {
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

pub fn get_hand_interpretations(tile_count_array: [u8; 34]) -> Vec<HandInterpretation> {
    // TODO handle declared melds (which are locked)
    let mut honor_tile_melds: Vec<TileMeld> = Vec::new();

    // start with handling honor tiles: all copies of each honor tile must build one group
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

        tile_id += 1;
    }

    // then, for each numbered suit (manzu, pinzu, souzu), recursively build subgroups
    let mut manzu_suit_tile_counts: [u8; 9] = [0; 9];
    let mut pinzu_suit_tile_counts: [u8; 9] = [0; 9];
    let mut souzu_suit_tile_counts: [u8; 9] = [0; 9];

    let manzu_melds = get_suit_melds(manzu_suit_tile_counts);
    let pinzu_melds = get_suit_melds(pinzu_suit_tile_counts);
    let souzu_melds = get_suit_melds(souzu_suit_tile_counts);

    // combine all possible meld-combinations from each suit (and the honors)
    let mut hand_interpretations = Vec::new();
    for &manzu_meld in manzu_melds.iter() {
        for &pinzu_meld in pinzu_melds.iter() {
            for &souzu_meld in souzu_melds.iter() {
                let mut all_melds = honor_tile_melds.clone();
                all_melds.extend(&manzu_meld);
                all_melds.extend(&pinzu_meld);
                all_melds.extend(&souzu_meld);

                let hand_interpretation = HandInterpretation {
                    total_tile_count_array: tile_count_array,
                    groups: all_melds,
                };
                hand_interpretations.push(hand_interpretation);
            }
        }
    }
    hand_interpretations
}

pub fn get_suit_melds(suit_tile_count_array: [u8; 9]) -> Vec<TileMeld> {
    let mut melds = Vec::new();
    for tile_idx in 0..suit_tile_count_array.len() {
        if suit_tile_count_array[tile_idx] == 0 {
            continue;
        }

        // TODO finish recursive implementation here
        todo!()
    }
    melds
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

pub fn get_chiitoi_shanten(tile_count_array: [u8; 34]) -> i8 {
    todo!()
}

pub fn get_kokushi_shanten(tile_count_array: [u8; 34]) -> i8 {
    todo!()
}

pub fn tiles_to_count_array(tiles_string: &str) -> [u8; 34] {
    todo!()
}
