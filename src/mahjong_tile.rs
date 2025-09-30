// rethinking the Tile class:
// what is a tile?
// - there is a tile value: its suit + rank (e.g. 3-manzu, 9-souzu, west wind, white dragon)
// - there are additional properties that tiles can have (which do not affect the value):
// - tile can be a red tile or not (which does not affect its value)
// - tiles can be part of a open meld or closed kan (i.e. they are fixed)
// - tiles can be drawn from different sources (e.g. from the wall, from the dead wall, from the discard pool)

// what is the most efficient representation of a tile?
// - if you treat red tiles as equivalent to their normal counterparts, there are 34 different tile types.
// - Each red tile adds 1 additional possible tile type for display / scoring purposes (e.g. with one red five in each numbered suit, there are 37 different possible tiles)
// - Regardless of if you play with red tiles or not, we can represent each tile's value
// as a number from 1 to 34 (inclusive). Red tiles only impact the scoring of the hand, not the hand shapes, dora ordering, etc..
// - Therefore, for most purposes, we can represent a collection of tiles as an array of size 34,
// where each array element is the count of that tile type.
// - But when constructing the total pool of tiles, scoring hands, and displaying hands in the interface,
// we'll need to distinguish red tiles vs. their normal counterparts (if red tiles are used)

// relations between tiles:
// - tiles can be identical (i.e. same type, which treats red tiles and their normal counterparts as identical)
// and ordered (to form sequence melds, which only applies for number suits: manzu, souzu, pinzu)
// - tiles have a special ordering for dora indicator, which applies for honor tiles and which wraps around for number tiles
// (for the purposes of dora order, red tiles are the same as their normal counterparts)

// the Hand will be a collection of tiles, with additional info:
// - declared melds (open melds + closed kans) need to be distinguished from the closed tiles in your hand
// (as tiles in the declared melds cannot be discarded)
// - for open melds, the source of the tile needs to be identified (which player discarded the tile)
// (for display purposes and for some edge cases in scoring)
// - the open melds need to be ordered as well (edge cases in scoring)
// - which tile was just drawn / added to the hand (if any) - which is used for hand scoring
// - how the tile was added to the hand (drawn from the wall, drawn from the dead wall, taken from opponent discard, etc.) - used for hand scoring

use std::fmt;

use crate::mahjong_error;

/// One of the numbered mahjong tile suits (i.e. excludes Wind and Dragon suits)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MahjongTileNumberedSuit {
    Man,
    Pin,
    Sou,
}

/// One of the distinct Mahjong tile values (does not differentiate between red tiles and non-red tiles)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MahjongTileValue {
    /// 1-9 m, p or s
    Number(u8, MahjongTileNumberedSuit),
    /// 1-4z
    Wind(u8),
    /// 5-7z
    Dragon(u8),
}

pub const NUM_DISTINCT_TILE_VALUES: u8 = 34;
pub const FIRST_PINZU_ID: u8 = 9;
pub const FIRST_SOUZU_ID: u8 = 18;
pub const FIRST_WIND_ID: u8 = 27;
pub const FIRST_HONOR_ID: u8 = FIRST_WIND_ID;
pub const FIRST_DRAGON_ID: u8 = 31;

impl MahjongTileValue {
    /// returns an integer in range 0-33 (inclusive) that represents the distinct tile values (red tiles have the same value),
    /// which is used as the array index for an array of size 34 that stores the count of each tile value (in a hand, in a discard pool, etc.)
    /// If the input is invalid, returns a `MahjongError`
    pub fn to_id(&self) -> Result<MahjongTileId, mahjong_error::MahjongError> {
        match self {
            MahjongTileValue::Number(value, tile_suit) => {
                let val = *value;
                if val < 1 || val > 9 {
                    return Err(mahjong_error::MahjongError {
                        message: format!("Invalid number suit value {} (should be 1-9)", val),
                    });
                }
                match *tile_suit {
                    MahjongTileNumberedSuit::Man => Ok(MahjongTileId(val - 1)),
                    MahjongTileNumberedSuit::Pin => Ok(MahjongTileId(val - 1 + FIRST_PINZU_ID)),
                    MahjongTileNumberedSuit::Sou => Ok(MahjongTileId(val - 1 + FIRST_SOUZU_ID)),
                }
            }
            MahjongTileValue::Wind(value) => {
                let val = *value;
                if val < 1 || val > 4 {
                    return Err(mahjong_error::MahjongError {
                        message: format!("Invalid wind value {} (should be 1-4)", val),
                    });
                }
                Ok(MahjongTileId(val - 1 + FIRST_WIND_ID))
            }
            MahjongTileValue::Dragon(value) => {
                let val = *value;
                if val < 5 || val > 7 {
                    return Err(mahjong_error::MahjongError {
                        message: format!("Invalid dragon value {} (should be 5-7)", val),
                    });
                }
                Ok(MahjongTileId(val - 1 + FIRST_WIND_ID))
            }
        }
    }

    /// takes an integer in range 0-33 (inclusive) that represents the distinct tile values (red tiles have the same value),
    /// and returns the corresponding value (does not include red tiles)
    /// If the input is invalid, returns a `MahjongError`
    pub fn from_id<T: Into<MahjongTileId>>(tile_id: T) -> Result<Self, mahjong_error::MahjongError> {
        let tile_id: MahjongTileId = tile_id.into();
        let id: u8 = tile_id.into();
        if id >= NUM_DISTINCT_TILE_VALUES {
            return Err(mahjong_error::MahjongError {
                message: format!(
                    "Invalid id value {} (should be 0-{})",
                    id,
                    NUM_DISTINCT_TILE_VALUES - 1
                ),
            });
        }
        if id < FIRST_PINZU_ID {
            // 0-8 is manzu (1m-9m)
            return Ok(MahjongTileValue::Number(id + 1, MahjongTileNumberedSuit::Man));
        } else if id < FIRST_SOUZU_ID {
            // 9-17 is pinzu (1p-9p)
            return Ok(MahjongTileValue::Number(
                (id - FIRST_PINZU_ID) + 1,
                MahjongTileNumberedSuit::Pin,
            ));
        } else if id < FIRST_WIND_ID {
            // 18-26 is souzu (1s-9s)
            return Ok(MahjongTileValue::Number(
                (id - FIRST_SOUZU_ID) + 1,
                MahjongTileNumberedSuit::Sou,
            ));
        } else if id < FIRST_DRAGON_ID {
            // 27-30 is winds (1z-4z)
            return Ok(MahjongTileValue::Wind((id - FIRST_WIND_ID) + 1));
        } else {
            // 31-33 is dragons (5z-7z)
            return Ok(MahjongTileValue::Dragon((id - FIRST_WIND_ID) + 1));
        }
    }

    /// Returns a text representation of a tile e.g. "1m" or "7z" or "0p" (0 refers to a red five)
    pub fn to_text(&self) -> String {
        match self {
            Self::Number(rank, suit) => {
                let mut tile_string = String::new();
                tile_string.push_str(&(rank.to_string()));
                tile_string.push_str(match suit {
                    MahjongTileNumberedSuit::Man => "m",
                    MahjongTileNumberedSuit::Pin => "p",
                    MahjongTileNumberedSuit::Sou => "s",
                });
                tile_string
            }
            Self::Wind(rank) | Self::Dragon(rank) => {
                let mut tile_string = rank.to_string().to_owned();
                tile_string.push_str("z");
                tile_string
            }
        }
    }
}

/// An integer representation of a `MahjongTileValue` (maps 1-to-1 for more compact storage)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MahjongTileId(pub u8);

// implement From trait -> Into trait will be defined automatically
impl From<u8> for MahjongTileId {
    fn from(id: u8) -> Self {
        MahjongTileId(id)
    }
}
impl From<MahjongTileId> for u8 {
    fn from(id: MahjongTileId) -> Self {
        id.0
    }
}
impl From<MahjongTileId> for usize {
    fn from(id: MahjongTileId) -> Self {
        usize::from(id.0)
    }
}

impl fmt::Display for MahjongTileId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write out with the MSPZ notation (use Debug for the raw tile_id value)
        write!(f, "{}", MahjongTileValue::from_id(*self).and_then(|id| Ok(id.to_text())).unwrap_or(format!("Invalid tile id: {}", self.0)))
    }
}

/// Returns Some with the rank of the tile (a number from 1-9) if the tile
/// is in a numbered suit (man, pin, or sou). Returns None otherise.
pub fn get_num_tile_rank<T: Into<MahjongTileId>>(tile_id: T) -> Option<u8> {
    let tile_id: MahjongTileId = tile_id.into();
    let tile_id = tile_id.0;
    if tile_id >= FIRST_HONOR_ID {
        None
    } else {
        if tile_id < 9 {
            Some(tile_id + 1)
        } else if tile_id < 18 {
            Some((tile_id - 9) + 1)
        } else {
            Some((tile_id - 18) + 1)
        }
    }
}

/// Returns Some with the suit of the tile (man, pin, or sou) if the tile
/// is in a numbered suit. Returns None otherise.
pub fn get_num_tile_suit(tile_id: u8) -> Option<MahjongTileNumberedSuit> {
    if tile_id >= FIRST_HONOR_ID {
        None
    } else {
        if tile_id < 9 {
            Some(MahjongTileNumberedSuit::Man)
        } else if tile_id < 18 {
            Some(MahjongTileNumberedSuit::Pin)
        } else {
            Some(MahjongTileNumberedSuit::Sou)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MahjongTile {
    value: MahjongTileValue,
    is_red: bool,
}
// TODO is it valid to compare based on MahjongTile::get_id() (i.e. should a red-five be considered equal to a normal five?)

impl Default for MahjongTile {
    fn default() -> Self {
        Self {
            value: MahjongTileValue::Dragon(7),
            is_red: false,
        }
    }
}

impl MahjongTile {
    // TODO constructor for MahjongTile to validate the MahjongTileValue? how to prevent building the struct with `MahjongTile { (...) }`

    pub fn get_id(&self) -> Result<MahjongTileId, mahjong_error::MahjongError> {
        self.value.to_id()
    }

    pub fn from_id<T: Into<MahjongTileId>>(id: T) -> Result<Self, mahjong_error::MahjongError> {
        let id: MahjongTileId = id.into();
        MahjongTileValue::from_id(id).map(|tile_value| MahjongTile {
            value: tile_value,
            ..Default::default()
        })
    }

    /// Parse a text representation of a tile e.g. "1m" or "7z" or "0p" (0 refers to a red five)
    pub fn from_text(tile_string: &str) -> Result<Self, mahjong_error::MahjongError> {
        if tile_string.len() != 2 {
            return Err(mahjong_error::MahjongError::new(
                "Tile string representation length must be 2",
            ));
        }
        let mut tile_str_chars = tile_string.chars();
        let first_char = tile_str_chars.next().unwrap();
        let second_char = tile_str_chars.next().unwrap();
        let parse_first_char = first_char.to_string().parse::<u8>();
        if parse_first_char.is_err() {
            return Err(mahjong_error::MahjongError::new(
                "Tile string representation length must be 2",
            ));
        }
        let mut rank_num = parse_first_char.unwrap();

        match second_char {
            suit if ['m', 'p', 's'].contains(&suit) => {
                let tile_suit = match suit {
                    'm' => MahjongTileNumberedSuit::Man,
                    'p' => MahjongTileNumberedSuit::Pin,
                    's' => MahjongTileNumberedSuit::Sou,
                    _ => {
                        return Err(mahjong_error::MahjongError::new(
                            "Expected number suit, should be m, p, or s",
                        ))
                    }
                };
                if rank_num > 9 {
                    return Err(mahjong_error::MahjongError::new(
                        "Number suit rank must be 0-9",
                    ));
                }
                let mut is_red = false;
                if rank_num == 0 {
                    is_red = true;
                    rank_num = 5;
                }
                let value = MahjongTileValue::Number(rank_num, tile_suit);
                Ok(Self { value, is_red })
            }
            suit if suit == 'z' => {
                let value = match rank_num {
                    rank_num if rank_num >= 1 && rank_num <= 4 => MahjongTileValue::Wind(rank_num),
                    rank_num if rank_num >= 5 && rank_num <= 7 => {
                        MahjongTileValue::Dragon(rank_num)
                    }
                    _ => {
                        return Err(mahjong_error::MahjongError::new(
                            "Honor suit rank must be 1-7",
                        ))
                    }
                };
                Ok(Self {
                    value,
                    is_red: false,
                })
            }
            _ => Err(mahjong_error::MahjongError::new(
                "Second char must be m, p, s, or z",
            )),
        }
    }
}

pub fn get_id_from_tile_text(tile_string: &str) -> Result<MahjongTileId, mahjong_error::MahjongError> {
    match MahjongTile::from_text(tile_string) {
        Ok(tile) => tile.get_id(),
        Err(x) => Err(x),
    }
}

pub fn get_tile_text_from_id<T: Into<MahjongTileId>>(tile_id: T) -> Result<String, mahjong_error::MahjongError> {
    let tile_id: MahjongTileId = tile_id.into();
    MahjongTileValue::from_id(tile_id).map(|tile| tile.to_text())
}

// ##### CONVERSION FUNCTIONS #####
// Individual tiles can be represented as:
// - `MahjongTile` objects, which distinguish between red-fives and non-red-fives (useful for scoring, but not as useful for shanten/tenpai analysis)
// - `u8` tile_id values, which do not distinguish between red-fives and non-red-fives
// - String tile representation in MSPZ notation e.g. "1z" -> east wind, "3p" -> 3-pin (can represent red fives as "0" i.e. "0s" means red-5-sou vs "5s" means non-red-5-sou)
// Groups/sets of tiles (e.g. a meld, a hand, etc.) can be represented as:
// - a collection (e.g. `Vec`) of any of the above, but usually either `Vec<MahjongTile>` or `Vec<u8>` -- it
//   doesn't have to be a `Vec`, what we really want is a multi-set (i.e. duplicates are allowed and order doesn't
//   matter i.e. [1s, 2s, 3s] should be equal to [3s, 1s, 2s], for gameplay purposes, we can have multiple tiles of the same value/type)
// - a "count array" i.e. a `[u8; 34]` array, where the value at index i represents the number of tiles of tile_id = i. For example, [1, 2, 0, 0, ..., 0] means [1m, 2m, 2m]
// - a condensed String representation in MSPZ notation e.g. "123s444p555z" -> [1s, 2s, 3s, 4p, 4p, 4p, 5z, 5z, 5z]

// utility function to generate a list of `MahjongTile` objects from a string
pub fn get_tiles_from_string(tile_string: &str) -> Vec<MahjongTile> {
    let mut tiles = Vec::new();
    let mut tile_ranks_so_far: Vec<char> = Vec::new();
    let tile_suit_chars = vec!['m', 'p', 's', 'z'];
    for current_tile_string_char in tile_string.chars() {
        if tile_suit_chars.contains(&current_tile_string_char) {
            for tile_rank in tile_ranks_so_far {
                let mut single_tile_string = String::new();
                single_tile_string.push(tile_rank);
                single_tile_string.push(current_tile_string_char.clone());
                tiles.push(MahjongTile::from_text(single_tile_string.as_str()).unwrap());
            }
            tile_ranks_so_far = vec![];
        } else {
            // assume if it's not a tile suit char, then it's a tile rank
            tile_ranks_so_far.push(current_tile_string_char);
        }
    }
    tiles
}

// TODO make this consistent with get_tiles_from_string (i.e. it should parse both 345m and 3m4m5m correctl)
/// Note: expects input string to be concatenated mspz notation e.g. 3m4m5m (and not 345m)
pub fn tiles_to_tile_ids(tiles_string: &str) -> Vec<MahjongTileId> {
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
                let tile_id = get_id_from_tile_text(&tile_string).unwrap();
                tile_ids.push(tile_id);
            }
            rank_chars = Vec::new();
        } else {
            rank_chars.push(char);
        }
    }
    tile_ids
}

pub fn tile_ids_to_strings<T: Into<MahjongTileId> + Copy>(tile_ids: &Vec<T>) -> Vec<String> {
    tile_ids
        .iter()
        .map(|tile_id| get_tile_text_from_id(*tile_id).unwrap())
        .collect()
}

/// A compact representation of a set of tiles: stored as a fixed-length array of 34 elements, where
/// the value at index i corresponds to how many tiles of tile_id = i are in the set.
/// For example: [1, 0, 2, 0, 0, ..., 0] represents [1m, 3m, 3m]
/// Note that representation does not distinguish between red tiles and non-red-tiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MahjongTileCountArray(pub [u8; 34]);

impl Default for MahjongTileCountArray {
    fn default() -> Self {
        MahjongTileCountArray([0; 34])
    }
}

// what are some common functions e.g. add tile id X to the count array, check if N copies of tile id X are in the count array, etc.
// see shanten.rs - can likely move some of those functions over to this file

pub fn get_total_tiles_from_count_array(tile_count_array: MahjongTileCountArray) -> usize {
    let mut total_tiles: usize = 0;
    for tile_idx in 0..tile_count_array.0.len() {
        total_tiles += usize::from(tile_count_array.0[tile_idx]);
    }
    total_tiles
}

pub fn get_tile_ids_from_count_array(tile_count_array: MahjongTileCountArray) -> Vec<MahjongTileId> {
    let mut tile_ids = vec![];
    for tile_id in 0..NUM_DISTINCT_TILE_VALUES {
        let tile_idx = usize::from(tile_id);
        if tile_count_array.0[tile_idx] > 0 {
            for _i in 0..tile_count_array.0[tile_idx] {
                tile_ids.push(MahjongTileId(tile_id));
            }
        }
    }
    tile_ids
}

pub fn get_distinct_tile_ids_from_count_array(tile_count_array: MahjongTileCountArray) -> Vec<MahjongTileId> {
    let mut distinct_tile_ids = vec![];
    for tile_id in 0..NUM_DISTINCT_TILE_VALUES {
        let tile_idx = usize::from(tile_id);
        if tile_count_array.0[tile_idx] > 0 {
            distinct_tile_ids.push(MahjongTileId(tile_id));
        }
    }
    distinct_tile_ids
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_tile_ids_suit_and_rank() {
        // tile_id for 1m = 0
        assert_eq!(get_num_tile_suit(0), Some(MahjongTileNumberedSuit::Man));
        assert_eq!(get_num_tile_rank(0), Some(1));

        // tile_id for 5m = 4
        assert_eq!(get_num_tile_suit(4), Some(MahjongTileNumberedSuit::Man));
        assert_eq!(get_num_tile_rank(4), Some(5));

        // tile_id for 7p = 15
        assert_eq!(get_num_tile_suit(15), Some(MahjongTileNumberedSuit::Pin));
        assert_eq!(get_num_tile_rank(15), Some(7));

        // tile_id for 2s = 19
        assert_eq!(get_num_tile_suit(19), Some(MahjongTileNumberedSuit::Sou));
        assert_eq!(get_num_tile_rank(19), Some(2));

        // tile_id for 1z = 27
        assert_eq!(get_num_tile_suit(27), None);
        assert_eq!(get_num_tile_rank(27), None);

        // invalid tile id
        assert_eq!(get_num_tile_suit(NUM_DISTINCT_TILE_VALUES), None);
        assert_eq!(get_num_tile_rank(NUM_DISTINCT_TILE_VALUES), None);

        // invalid tile id
        assert_eq!(get_num_tile_suit(100), None);
        assert_eq!(get_num_tile_rank(100), None);
    }

    #[test]
    fn tile_value_to_id() {
        let one_man = MahjongTileValue::Number(1, MahjongTileNumberedSuit::Man);
        assert_eq!(one_man.to_id(), Ok(MahjongTileId(0)));
        let five_man = MahjongTileValue::Number(5, MahjongTileNumberedSuit::Man);
        assert_eq!(five_man.to_id(), Ok(MahjongTileId(4)));
        let nine_man = MahjongTileValue::Number(9, MahjongTileNumberedSuit::Man);
        assert_eq!(nine_man.to_id(), Ok(MahjongTileId(8)));

        let three_pin = MahjongTileValue::Number(3, MahjongTileNumberedSuit::Pin);
        assert_eq!(three_pin.to_id(), Ok(MahjongTileId(11)));
        let four_pin = MahjongTileValue::Number(4, MahjongTileNumberedSuit::Pin);
        assert_eq!(four_pin.to_id(), Ok(MahjongTileId(12)));
        let seven_pin = MahjongTileValue::Number(7, MahjongTileNumberedSuit::Pin);
        assert_eq!(seven_pin.to_id(), Ok(MahjongTileId(15)));

        let two_sou = MahjongTileValue::Number(2, MahjongTileNumberedSuit::Sou);
        assert_eq!(two_sou.to_id(), Ok(MahjongTileId(19)));
        let six_sou = MahjongTileValue::Number(6, MahjongTileNumberedSuit::Sou);
        assert_eq!(six_sou.to_id(), Ok(MahjongTileId(23)));
        let eight_sou = MahjongTileValue::Number(8, MahjongTileNumberedSuit::Sou);
        assert_eq!(eight_sou.to_id(), Ok(MahjongTileId(25)));

        let east_wind = MahjongTileValue::Wind(1);
        assert_eq!(east_wind.to_id(), Ok(MahjongTileId(27)));
        let south_wind = MahjongTileValue::Wind(2);
        assert_eq!(south_wind.to_id(), Ok(MahjongTileId(28)));
        let west_wind = MahjongTileValue::Wind(3);
        assert_eq!(west_wind.to_id(), Ok(MahjongTileId(29)));
        let north_wind = MahjongTileValue::Wind(4);
        assert_eq!(north_wind.to_id(), Ok(MahjongTileId(30)));
        let white_dragon = MahjongTileValue::Dragon(5);
        assert_eq!(white_dragon.to_id(), Ok(MahjongTileId(31)));
        let green_dragon = MahjongTileValue::Dragon(6);
        assert_eq!(green_dragon.to_id(), Ok(MahjongTileId(32)));
        let red_dragon = MahjongTileValue::Dragon(7);
        assert_eq!(red_dragon.to_id(), Ok(MahjongTileId(33)));

        // invalid to_id values:
        let invalid_man = MahjongTileValue::Number(0, MahjongTileNumberedSuit::Man);
        assert!(invalid_man.to_id().is_err());
        let invalid_sou = MahjongTileValue::Number(10, MahjongTileNumberedSuit::Sou);
        assert!(invalid_sou.to_id().is_err());
        let invalid_pin = MahjongTileValue::Number(11, MahjongTileNumberedSuit::Pin);
        assert!(invalid_pin.to_id().is_err());

        let invalid_wind = MahjongTileValue::Wind(0);
        assert!(invalid_wind.to_id().is_err());
        let invalid_wind_high = MahjongTileValue::Wind(5);
        assert!(invalid_wind_high.to_id().is_err());
        let invalid_dragon = MahjongTileValue::Dragon(1);
        assert!(invalid_dragon.to_id().is_err());
        let invalid_dragon_high = MahjongTileValue::Dragon(8);
        assert!(invalid_dragon_high.to_id().is_err());
    }

    #[test]
    fn tile_value_from_id() {
        let one_man = MahjongTileValue::Number(1, MahjongTileNumberedSuit::Man);
        assert_eq!(MahjongTileValue::from_id(0), Ok(one_man));
        let five_man = MahjongTileValue::Number(5, MahjongTileNumberedSuit::Man);
        assert_eq!(MahjongTileValue::from_id(4), Ok(five_man));
        let nine_man = MahjongTileValue::Number(9, MahjongTileNumberedSuit::Man);
        assert_eq!(MahjongTileValue::from_id(8), Ok(nine_man));

        let three_pin = MahjongTileValue::Number(3, MahjongTileNumberedSuit::Pin);
        assert_eq!(MahjongTileValue::from_id(11), Ok(three_pin));
        let four_pin = MahjongTileValue::Number(4, MahjongTileNumberedSuit::Pin);
        assert_eq!(MahjongTileValue::from_id(12), Ok(four_pin));
        let seven_pin = MahjongTileValue::Number(7, MahjongTileNumberedSuit::Pin);
        assert_eq!(MahjongTileValue::from_id(15), Ok(seven_pin));

        let two_sou = MahjongTileValue::Number(2, MahjongTileNumberedSuit::Sou);
        assert_eq!(MahjongTileValue::from_id(19), Ok(two_sou));
        let six_sou = MahjongTileValue::Number(6, MahjongTileNumberedSuit::Sou);
        assert_eq!(MahjongTileValue::from_id(23), Ok(six_sou));
        let eight_sou = MahjongTileValue::Number(8, MahjongTileNumberedSuit::Sou);
        assert_eq!(MahjongTileValue::from_id(25), Ok(eight_sou));

        let east_wind = MahjongTileValue::Wind(1);
        assert_eq!(MahjongTileValue::from_id(27), Ok(east_wind));
        let south_wind = MahjongTileValue::Wind(2);
        assert_eq!(MahjongTileValue::from_id(28), Ok(south_wind));
        let west_wind = MahjongTileValue::Wind(3);
        assert_eq!(MahjongTileValue::from_id(29), Ok(west_wind));
        let north_wind = MahjongTileValue::Wind(4);
        assert_eq!(MahjongTileValue::from_id(30), Ok(north_wind));
        let white_dragon = MahjongTileValue::Dragon(5);
        assert_eq!(MahjongTileValue::from_id(31), Ok(white_dragon));
        let green_dragon = MahjongTileValue::Dragon(6);
        assert_eq!(MahjongTileValue::from_id(32), Ok(green_dragon));
        let red_dragon = MahjongTileValue::Dragon(7);
        assert_eq!(MahjongTileValue::from_id(33), Ok(red_dragon));

        // invalid from_id values:
        assert!(MahjongTileValue::from_id(NUM_DISTINCT_TILE_VALUES).is_err());
    }

    #[test]
    fn tile_from_text() {
        let one_man = MahjongTile::from_text("1m");
        assert!(one_man.is_ok());
        let one_man_tile = one_man.unwrap();
        assert!(
            one_man_tile.value == MahjongTileValue::Number(1, MahjongTileNumberedSuit::Man)
                && !one_man_tile.is_red
        );
        let three_pin = MahjongTile::from_text("3p");
        assert!(three_pin.is_ok());
        let three_pin_tile = three_pin.unwrap();
        assert!(
            three_pin_tile.value == MahjongTileValue::Number(3, MahjongTileNumberedSuit::Pin)
                && !three_pin_tile.is_red
        );
        let five_sou = MahjongTile::from_text("5s");
        assert!(five_sou.is_ok());
        let five_sou_tile = five_sou.unwrap();
        assert!(
            five_sou_tile.value == MahjongTileValue::Number(5, MahjongTileNumberedSuit::Sou)
                && !five_sou_tile.is_red
        );
        let red_five_sou = MahjongTile::from_text("0s");
        assert!(red_five_sou.is_ok());
        let red_five_sou_tile = red_five_sou.unwrap();
        assert!(
            red_five_sou_tile.value == MahjongTileValue::Number(5, MahjongTileNumberedSuit::Sou)
                && red_five_sou_tile.is_red
        );

        let south_wind = MahjongTile::from_text("2z");
        assert!(south_wind.is_ok());
        let south_wind_tile = south_wind.unwrap();
        assert!(south_wind_tile.value == MahjongTileValue::Wind(2) && !south_wind_tile.is_red);
        let west_wind = MahjongTile::from_text("3z");
        assert!(west_wind.is_ok());
        let west_wind_tile = west_wind.unwrap();
        assert!(west_wind_tile.value == MahjongTileValue::Wind(3) && !west_wind_tile.is_red);
        let green_dragon = MahjongTile::from_text("6z");
        assert!(green_dragon.is_ok());
        let green_dragon_tile = green_dragon.unwrap();
        assert!(
            green_dragon_tile.value == MahjongTileValue::Dragon(6) && !green_dragon_tile.is_red
        );

        // invalid values
        let invalid_man = MahjongTile::from_text("am");
        assert!(invalid_man.is_err());
        let invalid_suit = MahjongTile::from_text("1w");
        assert!(invalid_suit.is_err());
        let invalid_format = MahjongTile::from_text("06");
        assert!(invalid_format.is_err());
        let too_long = MahjongTile::from_text("14m");
        assert!(too_long.is_err());
        let too_short = MahjongTile::from_text("3");
        assert!(too_short.is_err());
        let invalid_dragon = MahjongTile::from_text("8z");
        assert!(invalid_dragon.is_err());
    }

    #[test]
    fn test_get_id_from_tile_text() {
        match get_id_from_tile_text("1m") {
            Ok(tile_id) => assert_eq!(tile_id, MahjongTileId(0)),
            Err(_) => assert!(false),
        };

        match get_id_from_tile_text("3p") {
            Ok(tile_id) => assert_eq!(tile_id, MahjongTileId(11)),
            Err(_) => assert!(false),
        };

        match get_id_from_tile_text("8s") {
            Ok(tile_id) => assert_eq!(tile_id, MahjongTileId(25)),
            Err(_) => assert!(false),
        };

        match get_id_from_tile_text("1z") {
            Ok(tile_id) => assert_eq!(tile_id, MahjongTileId(27)),
            Err(_) => assert!(false),
        };

        match get_id_from_tile_text("4z") {
            Ok(tile_id) => assert_eq!(tile_id, MahjongTileId(30)),
            Err(_) => assert!(false),
        };

        // invalid tile string should return Err(...)
        assert!(get_id_from_tile_text("9z").is_err());

        // handle red fives
        match get_id_from_tile_text("0m") {
            Ok(tile_id) => assert_eq!(tile_id, MahjongTileId(4)),
            Err(_) => assert!(false),
        };
        match get_id_from_tile_text("0p") {
            Ok(tile_id) => assert_eq!(tile_id, MahjongTileId(13)),
            Err(_) => assert!(false),
        };
        match get_id_from_tile_text("0s") {
            Ok(tile_id) => assert_eq!(tile_id, MahjongTileId(22)),
            Err(_) => assert!(false),
        };

        // but there is no "0z"
        assert!(get_id_from_tile_text("0z").is_err());
    }

    #[test]
    fn test_get_tile_text_from_id() {
        match get_tile_text_from_id(0) {
            Ok(tile_string) => assert_eq!(tile_string, "1m"),
            Err(_) => assert!(false),
        };

        match get_tile_text_from_id(2) {
            Ok(tile_string) => assert_eq!(tile_string, "3m"),
            Err(_) => assert!(false),
        };

        match get_tile_text_from_id(11) {
            Ok(tile_string) => assert_eq!(tile_string, "3p"),
            Err(_) => assert!(false),
        };

        match get_tile_text_from_id(15) {
            Ok(tile_string) => assert_eq!(tile_string, "7p"),
            Err(_) => assert!(false),
        };

        match get_tile_text_from_id(22) {
            Ok(tile_string) => assert_eq!(tile_string, "5s"),
            Err(_) => assert!(false),
        };

        match get_tile_text_from_id(25) {
            Ok(tile_string) => assert_eq!(tile_string, "8s"),
            Err(_) => assert!(false),
        };

        match get_tile_text_from_id(27) {
            Ok(tile_string) => assert_eq!(tile_string, "1z"),
            Err(_) => assert!(false),
        };

        match get_tile_text_from_id(32) {
            Ok(tile_string) => assert_eq!(tile_string, "6z"),
            Err(_) => assert!(false),
        };

        match get_tile_text_from_id(33) {
            Ok(tile_string) => assert_eq!(tile_string, "7z"),
            Err(_) => assert!(false),
        };

        assert!(get_tile_text_from_id(NUM_DISTINCT_TILE_VALUES).is_err());
    }

    #[test]
    fn test_get_tiles_from_string_single_suit() {
        let tile_string = "2333344445678s".to_string();
        let mut tiles_from_string = get_tiles_from_string(&tile_string);
        tiles_from_string.sort();

        let mut expected_tiles = vec![
            MahjongTile::from_text("2s").unwrap(),
            MahjongTile::from_text("3s").unwrap(),
            MahjongTile::from_text("3s").unwrap(),
            MahjongTile::from_text("3s").unwrap(),
            MahjongTile::from_text("3s").unwrap(),
            MahjongTile::from_text("4s").unwrap(),
            MahjongTile::from_text("4s").unwrap(),
            MahjongTile::from_text("4s").unwrap(),
            MahjongTile::from_text("4s").unwrap(),
            MahjongTile::from_text("5s").unwrap(),
            MahjongTile::from_text("6s").unwrap(),
            MahjongTile::from_text("7s").unwrap(),
            MahjongTile::from_text("8s").unwrap(),
        ];
        expected_tiles.sort();
        assert_eq!(tiles_from_string, expected_tiles);
    }

    #[test]
    fn test_get_tiles_from_string_mixed_suits() {
        let tile_string = "23445588s345p11z".to_string();
        let mut tiles_from_string = get_tiles_from_string(&tile_string);
        tiles_from_string.sort();

        let mut expected_tiles = vec![
            MahjongTile::from_text("2s").unwrap(),
            MahjongTile::from_text("3s").unwrap(),
            MahjongTile::from_text("4s").unwrap(),
            MahjongTile::from_text("4s").unwrap(),
            MahjongTile::from_text("5s").unwrap(),
            MahjongTile::from_text("5s").unwrap(),
            MahjongTile::from_text("8s").unwrap(),
            MahjongTile::from_text("8s").unwrap(),
            MahjongTile::from_text("3p").unwrap(),
            MahjongTile::from_text("4p").unwrap(),
            MahjongTile::from_text("5p").unwrap(),
            MahjongTile::from_text("1z").unwrap(),
            MahjongTile::from_text("1z").unwrap(),
        ];
        expected_tiles.sort();
        assert_eq!(tiles_from_string, expected_tiles);

        let tile_string = "122234m789s345p33z".to_string();
        let mut tiles_from_string = get_tiles_from_string(&tile_string);
        tiles_from_string.sort();

        let mut expected_tiles = vec![
            MahjongTile::from_text("3z").unwrap(),
            MahjongTile::from_text("3z").unwrap(),
            MahjongTile::from_text("1m").unwrap(),
            MahjongTile::from_text("2m").unwrap(),
            MahjongTile::from_text("2m").unwrap(),
            MahjongTile::from_text("2m").unwrap(),
            MahjongTile::from_text("3m").unwrap(),
            MahjongTile::from_text("4m").unwrap(),
            MahjongTile::from_text("7s").unwrap(),
            MahjongTile::from_text("8s").unwrap(),
            MahjongTile::from_text("9s").unwrap(),
            MahjongTile::from_text("3p").unwrap(),
            MahjongTile::from_text("4p").unwrap(),
            MahjongTile::from_text("5p").unwrap(),
        ];
        expected_tiles.sort();
        assert_eq!(tiles_from_string, expected_tiles);
    }
}
