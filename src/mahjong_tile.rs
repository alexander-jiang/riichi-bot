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

pub const NUM_DISTINCT_TILE_VALUES: u8 = 34;
pub const FIRST_PINZU_ID: u8 = 9;
pub const FIRST_SOUZU_ID: u8 = 18;
pub const FIRST_WIND_ID: u8 = 27;
pub const FIRST_HONOR_ID: u8 = FIRST_WIND_ID;
pub const FIRST_DRAGON_ID: u8 = 31;

/// An integer representation of one of the distinct Mahjong tile value (maps 1-to-1 with a `u8` for more compact storage)
/// These are the distinct tile values used for winning hand shape / tenpai / shanten calculation i.e. does not
/// distinguish between red and non-red tiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MahjongTileId(pub u8);

// implement `From<T> for MahjongTileId` trait -> the corresponding `Into<MahjongTileId> for T` trait will be defined automatically
impl From<u8> for MahjongTileId {
    fn from(id: u8) -> Self {
        let tile_id = MahjongTileId(id);
        if !tile_id.is_valid_id() {
            panic!("invalid tile id {}", tile_id);
        }
        tile_id
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

// implement Display trait using MSPZ notation i.e. green dragon -> "6z", east wind = "1z" (use Debug for the raw tile_id value)
impl fmt::Display for MahjongTileId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_text())
    }
}

impl MahjongTileId {
    fn new_number_tile(rank: u8, suit: MahjongTileNumberedSuit) -> Self {
        if rank == 0 || rank > 9 {
            panic!("invalid number tile rank {}", rank);
        }
        match suit {
            MahjongTileNumberedSuit::Man => MahjongTileId(rank - 1),
            MahjongTileNumberedSuit::Pin => MahjongTileId(rank - 1 + FIRST_PINZU_ID),
            MahjongTileNumberedSuit::Sou => MahjongTileId(rank - 1 + FIRST_SOUZU_ID),
        }
    }

    fn new_honor_tile(rank: u8) -> Self {
        if rank == 0 || rank > 7 {
            panic!("invalid honor tile rank {}", rank);
        }
        MahjongTileId(FIRST_HONOR_ID + rank - 1)
    }

    fn is_valid_id(&self) -> bool {
        self.0 < NUM_DISTINCT_TILE_VALUES
    }

    fn is_numbered_suit(&self) -> bool {
        self.0 < FIRST_HONOR_ID
    }

    // fn is_wind_tile(&self) -> bool {
    //     self.0 >= FIRST_WIND_ID && self.0 < FIRST_DRAGON_ID
    // }

    // fn is_dragon_tile(&self) -> bool {
    //     self.0 >= FIRST_DRAGON_ID && self.0 < NUM_DISTINCT_TILE_VALUES
    // }

    /// Returns Some with the rank of the tile (a number from 1-9) if the tile
    /// is in a numbered suit (man, pin, or sou). Returns None otherise.
    fn get_num_tile_rank(&self) -> Option<u8> {
        if !self.is_numbered_suit() {
            None
        } else {
            if self.0 < FIRST_PINZU_ID {
                Some(self.0 + 1)
            } else if self.0 < FIRST_SOUZU_ID {
                Some(self.0 - FIRST_PINZU_ID + 1)
            } else {
                Some(self.0 - FIRST_SOUZU_ID + 1)
            }
        }
    }

    /// Returns Some with the suit of the tile (man, pin, or sou) if the tile
    /// is in a numbered suit. Returns None otherise.
    fn get_num_tile_suit(&self) -> Option<MahjongTileNumberedSuit> {
        if !self.is_numbered_suit() {
            None
        } else if self.0 < FIRST_PINZU_ID {
            // 0-8 is manzu (1m-9m)
            Some(MahjongTileNumberedSuit::Man)
        } else if self.0 < FIRST_SOUZU_ID {
            // 9-17 is pinzu (1p-9p)
            Some(MahjongTileNumberedSuit::Pin)
        } else {
            // 18-26 is souzu (1s-9s)
            Some(MahjongTileNumberedSuit::Sou)
        }
    }

    /// Convert the tile id to MSPZ notation i.e. 1 of circles -> "1p", green dragon -> "6z", east wind = "1z".
    /// Panics if the tile id is not valid.
    fn to_text(&self) -> String {
        if self.is_numbered_suit() {
            let rank = self.get_num_tile_rank().unwrap();
            let suit = self.get_num_tile_suit().unwrap();
            let mut tile_string = String::new();
            tile_string.push_str(&(rank.to_string()));
            tile_string.push_str(match suit {
                MahjongTileNumberedSuit::Man => "m",
                MahjongTileNumberedSuit::Pin => "p",
                MahjongTileNumberedSuit::Sou => "s",
            });
            tile_string
        } else if self.is_valid_id() {
            let rank = self.0 - FIRST_HONOR_ID + 1;
            let mut tile_string = rank.to_string().to_owned();
            tile_string.push_str("z");
            tile_string
        } else {
            panic!("invalid tile id: {}", self.0)
            // format!("invalid tile id: {}", self.0)
        }
    }
}

/// Returns Some with the rank of the tile (a number from 1-9) if the tile
/// is in a numbered suit (man, pin, or sou). Returns None otherise.
pub fn get_num_tile_rank<T: Into<MahjongTileId>>(tile_id: T) -> Option<u8> {
    let tile_id: MahjongTileId = tile_id.into();
    tile_id.get_num_tile_rank()
}

/// Returns Some with the suit of the tile (man, pin, or sou) if the tile
/// is in a numbered suit. Returns None otherise.
pub fn get_num_tile_suit<T: Into<MahjongTileId>>(tile_id: T) -> Option<MahjongTileNumberedSuit> {
    let tile_id: MahjongTileId = tile_id.into();
    tile_id.get_num_tile_suit()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MahjongTile {
    pub value: MahjongTileId,
    pub is_red: bool,
}
// TODO is it valid to compare based on MahjongTile.value (i.e. should a red-five be considered equal to a normal five?)

impl Default for MahjongTile {
    // default to red dragon, the prototypical mahjong tile
    fn default() -> Self {
        Self {
            value: MahjongTileId(NUM_DISTINCT_TILE_VALUES - 1),
            is_red: false,
        }
    }
}

impl<T: Into<MahjongTileId>> From<T> for MahjongTile {
    fn from(value: T) -> Self {
        let tile_id: MahjongTileId = value.into();
        MahjongTile {
            value: tile_id,
            ..Default::default()
        }
    }
}

impl MahjongTile {
    /// Parses the MSPZ notation i.e. 1 of circles -> "1p", green dragon -> "6z", east wind = "1z".
    /// Accepts red-fives as "0" i.e. red-five-man is "0m", vs. five-man is "5m"
    /// Returns Error if the string isn't valid.
    pub fn from_text<S: AsRef<str>>(tile_string: S) -> Result<Self, mahjong_error::MahjongError> {
        let tile_string = tile_string.as_ref();
        if tile_string.len() != 2 {
            return Err(mahjong_error::MahjongError::new(
                "Tile string representation length must be 2",
            ));
        }
        let mut tile_str_chars = tile_string.chars();
        let first_char = tile_str_chars.next().unwrap();
        let second_char = tile_str_chars.next().unwrap();

        // Parse first character -> rank
        let parse_first_char = first_char.to_string().parse::<u8>();
        if parse_first_char.is_err() {
            return Err(mahjong_error::MahjongError::new(
                "First character must be a number",
            ));
        }
        let mut rank_num = parse_first_char.unwrap();

        // Parse first character -> suit
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
                        "Number suit rank must be 0-9, where 0 represents a red five",
                    ));
                }
                let mut is_red = false;
                if rank_num == 0 {
                    is_red = true;
                    rank_num = 5;
                }
                let value = MahjongTileId::new_number_tile(rank_num, tile_suit);
                Ok(Self { value, is_red })
            }
            suit if suit == 'z' => {
                let value = match rank_num {
                    rank_num if rank_num >= 1 && rank_num <= 7 => {
                        MahjongTileId::new_honor_tile(rank_num)
                    }
                    _ => {
                        return Err(mahjong_error::MahjongError::new(
                            "Honor suit rank must be 1-7 (1-4 for winds, 5-7 for dragons)",
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

pub fn get_id_from_tile_text(
    tile_string: &str,
) -> Result<MahjongTileId, mahjong_error::MahjongError> {
    MahjongTile::from_text(tile_string).map(|tile| tile.value)
}

pub fn get_tile_text_from_id<T: Into<MahjongTileId>>(tile_id: T) -> String {
    let tile_id: MahjongTileId = tile_id.into();
    tile_id.to_text()
}

// ##### CONVERSION FUNCTIONS #####
// Individual tiles can be represented as:
// - `MahjongTile` objects, which distinguish between red-fives and non-red-fives (useful for scoring, but not as useful for shanten/tenpai analysis)
// - `MahjongTileValue` objects, which ... i'm not sure why we have...
// - tile_id values (`u8` or `MahjongTileId`, which are interchangeable), which do not distinguish between red-fives and non-red-fives
// - String tile representation in MSPZ notation e.g. "1z" -> east wind, "3p" -> 3-pin (can represent red fives as "0" i.e. "0s" means red-5-sou vs "5s" means non-red-5-sou)
// Groups/sets of tiles (e.g. a meld, a hand, etc.) can be represented as:
// - a collection (e.g. `Vec`) of any of the above, but usually either `Vec<MahjongTile>` or `Vec<u8>` -- it
//   doesn't have to be a `Vec`, what we really want is a multi-set (i.e. duplicates are allowed and order doesn't
//   matter i.e. [1s, 2s, 3s] should be equal to [3s, 1s, 2s], for gameplay purposes, we can have multiple tiles of the same value/type)
// - a "count array" i.e. a `[u8; 34]` array, where the value at index i represents the number of tiles of tile_id = i. For example, [1, 2, 0, 0, ..., 0] means [1m, 2m, 2m]
// - a condensed String representation in MSPZ notation e.g. "123s444p555z" -> [1s, 2s, 3s, 4p, 4p, 4p, 5z, 5z, 5z]

/// utility function to generate a list of `MahjongTile` objects from a string
pub fn get_tiles_from_string(tile_string: &str) -> Vec<MahjongTile> {
    let mut tiles = Vec::new();
    let mut tile_ranks_so_far: Vec<char> = Vec::new();
    let tile_suit_chars = vec!['m', 'p', 's', 'z'];
    for current_tile_string_char in tile_string.chars() {
        if tile_suit_chars.contains(&current_tile_string_char) {
            // potential optimization: is it faster to tile_suit_chars.contains(...) vs. doing == for each possible char
            for tile_rank in tile_ranks_so_far {
                let mut single_tile_string = String::new();
                single_tile_string.push(tile_rank);
                single_tile_string.push(current_tile_string_char);
                let tile = MahjongTile::from_text(single_tile_string.as_str()).unwrap();
                tiles.push(tile);
            }
            tile_ranks_so_far = vec![];
        } else {
            // assume if it's not a tile suit char, then it's a tile rank
            tile_ranks_so_far.push(current_tile_string_char);
        }
    }
    tiles
}

/// utility function to generate a list of `MahjongTileId` objects from a string
pub fn get_tile_ids_from_string(tiles_string: &str) -> Vec<MahjongTileId> {
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
            // assume if it's not a tile suit char, then it's a tile rank
            rank_chars.push(char);
        }
    }
    tile_ids
}

pub fn tile_ids_to_strings<T: Into<MahjongTileId> + Copy>(tile_ids: &Vec<T>) -> Vec<String> {
    tile_ids
        .iter()
        .map(|tile_id| get_tile_text_from_id(*tile_id))
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

pub const FOUR_OF_EACH_TILE_COUNT_ARRAY: MahjongTileCountArray = MahjongTileCountArray([4u8; 34]);

// what are some common functions e.g. add tile id X to the count array, check if N copies of tile id X are in the count array, etc.
// see shanten.rs - can likely move some of those functions over to this file

pub fn get_total_tiles_from_count_array(tile_count_array: MahjongTileCountArray) -> usize {
    let mut total_tiles: usize = 0;
    for tile_idx in 0..tile_count_array.0.len() {
        total_tiles += usize::from(tile_count_array.0[tile_idx]);
    }
    total_tiles
}

pub fn get_tile_ids_from_count_array(
    tile_count_array: MahjongTileCountArray,
) -> Vec<MahjongTileId> {
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

pub fn get_distinct_tile_ids_from_count_array(
    tile_count_array: MahjongTileCountArray,
) -> Vec<MahjongTileId> {
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
    fn test_get_num_tile_suit_and_rank() {
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
    }

    #[test]
    fn test_mahjong_tile_id_new() {
        let one_man = MahjongTileId::new_number_tile(1, MahjongTileNumberedSuit::Man);
        assert_eq!(one_man, MahjongTileId(0));
        let five_man = MahjongTileId::new_number_tile(5, MahjongTileNumberedSuit::Man);
        assert_eq!(five_man, MahjongTileId(4));
        let nine_man = MahjongTileId::new_number_tile(9, MahjongTileNumberedSuit::Man);
        assert_eq!(nine_man, MahjongTileId(8));

        let three_pin = MahjongTileId::new_number_tile(3, MahjongTileNumberedSuit::Pin);
        assert_eq!(three_pin, MahjongTileId(11));
        let four_pin = MahjongTileId::new_number_tile(4, MahjongTileNumberedSuit::Pin);
        assert_eq!(four_pin, MahjongTileId(12));
        let seven_pin = MahjongTileId::new_number_tile(7, MahjongTileNumberedSuit::Pin);
        assert_eq!(seven_pin, MahjongTileId(15));

        let two_sou = MahjongTileId::new_number_tile(2, MahjongTileNumberedSuit::Sou);
        assert_eq!(two_sou, MahjongTileId(19));
        let six_sou = MahjongTileId::new_number_tile(6, MahjongTileNumberedSuit::Sou);
        assert_eq!(six_sou, MahjongTileId(23));
        let eight_sou = MahjongTileId::new_number_tile(8, MahjongTileNumberedSuit::Sou);
        assert_eq!(eight_sou, MahjongTileId(25));

        let east_wind = MahjongTileId::new_honor_tile(1);
        assert_eq!(east_wind, MahjongTileId(27));
        let south_wind = MahjongTileId::new_honor_tile(2);
        assert_eq!(south_wind, MahjongTileId(28));
        let west_wind = MahjongTileId::new_honor_tile(3);
        assert_eq!(west_wind, MahjongTileId(29));
        let north_wind = MahjongTileId::new_honor_tile(4);
        assert_eq!(north_wind, MahjongTileId(30));
        let white_dragon = MahjongTileId::new_honor_tile(5);
        assert_eq!(white_dragon, MahjongTileId(31));
        let green_dragon = MahjongTileId::new_honor_tile(6);
        assert_eq!(green_dragon, MahjongTileId(32));
        let red_dragon = MahjongTileId::new_honor_tile(7);
        assert_eq!(red_dragon, MahjongTileId(33));
    }

    macro_rules! test_statements_that_should_panic {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            #[should_panic]
            fn $name() {
                #[allow(unused_must_use)]
                $value;
            }
        )*
        }
    }
    test_statements_that_should_panic! {
        test_get_num_tile_suit_invalid: get_num_tile_suit(NUM_DISTINCT_TILE_VALUES),
        test_get_num_tile_rank_invalid: get_num_tile_rank(NUM_DISTINCT_TILE_VALUES),
        test_get_num_tile_suit_invalid_100: get_num_tile_suit(100),
        test_get_num_tile_rank_invalid_100: get_num_tile_rank(100),
        test_mahjong_tile_id_new_invalid_zero_man: MahjongTileId::new_number_tile(0, MahjongTileNumberedSuit::Man),
        test_mahjong_tile_id_new_invalid_ten_sou: MahjongTileId::new_number_tile(10, MahjongTileNumberedSuit::Sou),
        test_mahjong_tile_id_new_invalid_eleven_pin: MahjongTileId::new_number_tile(11, MahjongTileNumberedSuit::Pin),
        test_mahjong_tile_id_new_invalid_zero_honor: MahjongTileId::new_honor_tile(0),
        test_mahjong_tile_id_new_invalid_eight_honor: MahjongTileId::new_honor_tile(8),
        test_mahjong_tile_id_invalid: MahjongTileId::from(NUM_DISTINCT_TILE_VALUES),
        test_get_tile_text_from_id_invalid_id: get_tile_text_from_id(NUM_DISTINCT_TILE_VALUES),
    }

    #[test]
    fn tile_from_text() {
        let one_man = MahjongTile::from_text("1m");
        assert!(one_man.is_ok());
        let one_man_tile = one_man.unwrap();
        assert!(
            one_man_tile.value == MahjongTileId::new_number_tile(1, MahjongTileNumberedSuit::Man)
                && !one_man_tile.is_red
        );
        let three_pin = MahjongTile::from_text("3p");
        assert!(three_pin.is_ok());
        let three_pin_tile = three_pin.unwrap();
        assert!(
            three_pin_tile.value == MahjongTileId::new_number_tile(3, MahjongTileNumberedSuit::Pin)
                && !three_pin_tile.is_red
        );
        let five_sou = MahjongTile::from_text("5s");
        assert!(five_sou.is_ok());
        let five_sou_tile = five_sou.unwrap();
        assert!(
            five_sou_tile.value == MahjongTileId::new_number_tile(5, MahjongTileNumberedSuit::Sou)
                && !five_sou_tile.is_red
        );
        let red_five_sou = MahjongTile::from_text("0s");
        assert!(red_five_sou.is_ok());
        let red_five_sou_tile = red_five_sou.unwrap();
        assert!(
            red_five_sou_tile.value
                == MahjongTileId::new_number_tile(5, MahjongTileNumberedSuit::Sou)
                && red_five_sou_tile.is_red
        );

        let south_wind = MahjongTile::from_text("2z");
        assert!(south_wind.is_ok());
        let south_wind_tile = south_wind.unwrap();
        assert!(
            south_wind_tile.value == MahjongTileId::new_honor_tile(2) && !south_wind_tile.is_red
        );
        let west_wind = MahjongTile::from_text("3z");
        assert!(west_wind.is_ok());
        let west_wind_tile = west_wind.unwrap();
        assert!(west_wind_tile.value == MahjongTileId::new_honor_tile(3) && !west_wind_tile.is_red);
        let green_dragon = MahjongTile::from_text("6z");
        assert!(green_dragon.is_ok());
        let green_dragon_tile = green_dragon.unwrap();
        assert!(
            green_dragon_tile.value == MahjongTileId::new_honor_tile(6)
                && !green_dragon_tile.is_red
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
        assert_eq!(get_tile_text_from_id(0), "1m");
        assert_eq!(get_tile_text_from_id(2), "3m");
        assert_eq!(get_tile_text_from_id(11), "3p");
        assert_eq!(get_tile_text_from_id(15), "7p");
        assert_eq!(get_tile_text_from_id(22), "5s");
        assert_eq!(get_tile_text_from_id(25), "8s");
        assert_eq!(get_tile_text_from_id(27), "1z");
        assert_eq!(get_tile_text_from_id(32), "6z");
        assert_eq!(get_tile_text_from_id(33), "7z");
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

    #[test]
    fn test_get_tile_ids_from_string() {
        let mut tiles = get_tile_ids_from_string("3m4m5m");
        tiles.sort();
        let mut expected_tiles = vec![
            MahjongTile::from_text("3m").unwrap().value,
            MahjongTile::from_text("4m").unwrap().value,
            MahjongTile::from_text("5m").unwrap().value,
        ];
        expected_tiles.sort();
        assert_eq!(tiles, expected_tiles);

        let mut tiles = get_tile_ids_from_string("345m");
        tiles.sort();
        let mut expected_tiles = vec![
            MahjongTile::from_text("3m").unwrap().value,
            MahjongTile::from_text("4m").unwrap().value,
            MahjongTile::from_text("5m").unwrap().value,
        ];
        expected_tiles.sort();
        assert_eq!(tiles, expected_tiles);
    }

    #[test]
    fn test_get_total_tiles_from_count_array() {
        let empty_tile_count_array: MahjongTileCountArray = Default::default();
        assert_eq!(get_total_tiles_from_count_array(empty_tile_count_array), 0);

        assert_eq!(
            get_total_tiles_from_count_array(FOUR_OF_EACH_TILE_COUNT_ARRAY),
            4 * 34
        );

        let mut tile_count_array: MahjongTileCountArray = Default::default();
        tile_count_array.0[usize::from(get_id_from_tile_text("3m").unwrap())] = 2;
        tile_count_array.0[usize::from(get_id_from_tile_text("2p").unwrap())] = 1;
        tile_count_array.0[usize::from(get_id_from_tile_text("4s").unwrap())] = 1;
        tile_count_array.0[usize::from(get_id_from_tile_text("6z").unwrap())] = 2;
        assert_eq!(get_total_tiles_from_count_array(tile_count_array), 6);
    }

    #[test]
    fn test_get_tile_ids_from_count_array() {
        let empty_tile_count_array: MahjongTileCountArray = Default::default();
        let empty_tile_ids = get_tile_ids_from_count_array(empty_tile_count_array);
        assert!(empty_tile_ids.is_empty());

        let mut tile_count_array: MahjongTileCountArray = Default::default();
        tile_count_array.0[usize::from(get_id_from_tile_text("3m").unwrap())] = 2;
        tile_count_array.0[usize::from(get_id_from_tile_text("2p").unwrap())] = 1;
        let tile_ids = get_tile_ids_from_count_array(tile_count_array);
        assert_eq!(tile_ids.len(), 3);
        let matching_3m_tiles: Vec<MahjongTileId> = tile_ids
            .iter()
            .cloned()
            .filter(|&tile_id| tile_id == get_id_from_tile_text("3m").unwrap())
            .collect();
        assert_eq!(matching_3m_tiles.len(), 2);
        let matching_2p_tiles: Vec<MahjongTileId> = tile_ids
            .iter()
            .cloned()
            .filter(|&tile_id| tile_id == get_id_from_tile_text("2p").unwrap())
            .collect();
        assert_eq!(matching_2p_tiles.len(), 1);
    }

    #[test]
    fn test_get_distinct_tile_ids_from_count_array() {
        let empty_tile_count_array: MahjongTileCountArray = Default::default();
        let empty_tile_ids = get_distinct_tile_ids_from_count_array(empty_tile_count_array);
        assert!(empty_tile_ids.is_empty());

        let mut tile_count_array: MahjongTileCountArray = Default::default();
        tile_count_array.0[usize::from(get_id_from_tile_text("3m").unwrap())] = 2;
        tile_count_array.0[usize::from(get_id_from_tile_text("2p").unwrap())] = 1;
        let distinct_tile_ids = get_distinct_tile_ids_from_count_array(tile_count_array);
        assert_eq!(distinct_tile_ids.len(), 2);
        assert!(distinct_tile_ids.contains(&get_id_from_tile_text("3m").unwrap()));
        assert!(distinct_tile_ids.contains(&get_id_from_tile_text("2p").unwrap()));

        assert!(!distinct_tile_ids.contains(&get_id_from_tile_text("3p").unwrap()));
        assert!(!distinct_tile_ids.contains(&get_id_from_tile_text("2m").unwrap()));
    }
}
