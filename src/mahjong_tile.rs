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

use crate::mahjong_error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MahjongTileSuit {
    Man,
    Pin,
    Sou,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MahjongTileValue {
    /// 1-9 m, p or s
    Number(u8, MahjongTileSuit),
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
    pub fn to_id(&self) -> Result<u8, mahjong_error::MahjongError> {
        match self {
            MahjongTileValue::Number(value, tile_suit) => {
                let val = *value;
                if val < 1 || val > 9 {
                    return Err(mahjong_error::MahjongError {
                        message: format!("Invalid number suit value {} (should be 1-9)", val),
                    });
                }
                match *tile_suit {
                    MahjongTileSuit::Man => Ok(val - 1),
                    MahjongTileSuit::Pin => Ok(val - 1 + FIRST_PINZU_ID),
                    MahjongTileSuit::Sou => Ok(val - 1 + FIRST_SOUZU_ID),
                }
            }
            MahjongTileValue::Wind(value) => {
                let val = *value;
                if val < 1 || val > 4 {
                    return Err(mahjong_error::MahjongError {
                        message: format!("Invalid wind value {} (should be 1-4)", val),
                    });
                }
                Ok(val - 1 + FIRST_WIND_ID)
            }
            MahjongTileValue::Dragon(value) => {
                let val = *value;
                if val < 5 || val > 7 {
                    return Err(mahjong_error::MahjongError {
                        message: format!("Invalid dragon value {} (should be 5-7)", val),
                    });
                }
                Ok(val - 1 + FIRST_WIND_ID)
            }
        }
    }

    /// takes an integer in range 0-33 (inclusive) that represents the distinct tile values (red tiles have the same value),
    /// and returns the corresponding value (does not include red tiles)
    /// If the input is invalid, returns a `MahjongError`
    pub fn from_id(id: u8) -> Result<Self, mahjong_error::MahjongError> {
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
            return Ok(MahjongTileValue::Number(id + 1, MahjongTileSuit::Man));
        } else if id < FIRST_SOUZU_ID {
            // 9-17 is pinzu (1p-9p)
            return Ok(MahjongTileValue::Number(
                (id - FIRST_PINZU_ID) + 1,
                MahjongTileSuit::Pin,
            ));
        } else if id < FIRST_WIND_ID {
            // 18-26 is souzu (1s-9s)
            return Ok(MahjongTileValue::Number(
                (id - FIRST_SOUZU_ID) + 1,
                MahjongTileSuit::Sou,
            ));
        } else if id < FIRST_DRAGON_ID {
            // 27-30 is winds (1z-4z)
            return Ok(MahjongTileValue::Wind((id - FIRST_WIND_ID) + 1));
        } else {
            // 31-33 is dragons (5z-7z)
            return Ok(MahjongTileValue::Dragon((id - FIRST_WIND_ID) + 1));
        }
    }
}

/// Returns Some with the rank of the tile (a number from 1-9) if the tile
/// is in a numbered suit (man, pin, or sou). Returns None otherise.
pub fn get_num_tile_rank(tile_id: u8) -> Option<u8> {
    if tile_id >= FIRST_HONOR_ID {
        Option::None
    } else {
        if tile_id < 9 {
            Option::Some(tile_id + 1)
        } else if tile_id < 18 {
            Option::Some((tile_id - 9) + 1)
        } else {
            Option::Some((tile_id - 18) + 1)
        }
    }
}

/// Returns Some with the suit of the tile (man, pin, or sou) if the tile
/// is in a numbered suit. Returns None otherise.
pub fn get_num_tile_suit(tile_id: u8) -> Option<MahjongTileSuit> {
    if tile_id >= FIRST_HONOR_ID {
        Option::None
    } else {
        if tile_id < 9 {
            Option::Some(MahjongTileSuit::Man)
        } else if tile_id < 18 {
            Option::Some(MahjongTileSuit::Pin)
        } else {
            Option::Some(MahjongTileSuit::Sou)
        }
    }
}

pub struct MahjongTile {
    value: MahjongTileValue,
    is_red: bool,
}

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

    pub fn get_id(&self) -> Result<u8, mahjong_error::MahjongError> {
        self.value.to_id()
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
                    'm' => MahjongTileSuit::Man,
                    'p' => MahjongTileSuit::Pin,
                    's' => MahjongTileSuit::Sou,
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

pub fn get_id_from_tile_text(tile_string: &str) -> Result<u8, mahjong_error::MahjongError> {
    match MahjongTile::from_text(tile_string) {
        Ok(tile) => tile.get_id(),
        Err(x) => Err(x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_tile_ids_suit_and_rank() {
        // tile_id for 1m = 0
        assert_eq!(get_num_tile_suit(0), Option::Some(MahjongTileSuit::Man));
        assert_eq!(get_num_tile_rank(0), Option::Some(1));

        // tile_id for 5m = 4
        assert_eq!(get_num_tile_suit(4), Option::Some(MahjongTileSuit::Man));
        assert_eq!(get_num_tile_rank(4), Option::Some(5));

        // tile_id for 7p = 15
        assert_eq!(get_num_tile_suit(15), Option::Some(MahjongTileSuit::Pin));
        assert_eq!(get_num_tile_rank(15), Option::Some(7));

        // tile_id for 2s = 19
        assert_eq!(get_num_tile_suit(19), Option::Some(MahjongTileSuit::Sou));
        assert_eq!(get_num_tile_rank(19), Option::Some(2));

        // tile_id for 1z = 27
        assert_eq!(get_num_tile_suit(27), Option::None);
        assert_eq!(get_num_tile_rank(27), Option::None);

        // tile_id for 7z = 34
        assert_eq!(get_num_tile_suit(34), Option::None);
        assert_eq!(get_num_tile_rank(34), Option::None);

        // invalid tile id
        assert_eq!(get_num_tile_suit(100), Option::None);
        assert_eq!(get_num_tile_rank(100), Option::None);
    }

    #[test]
    fn tile_value_to_id() {
        let one_man = MahjongTileValue::Number(1, MahjongTileSuit::Man);
        assert_eq!(one_man.to_id(), Ok(0));
        let five_man = MahjongTileValue::Number(5, MahjongTileSuit::Man);
        assert_eq!(five_man.to_id(), Ok(4));
        let nine_man = MahjongTileValue::Number(9, MahjongTileSuit::Man);
        assert_eq!(nine_man.to_id(), Ok(8));

        let three_pin = MahjongTileValue::Number(3, MahjongTileSuit::Pin);
        assert_eq!(three_pin.to_id(), Ok(11));
        let four_pin = MahjongTileValue::Number(4, MahjongTileSuit::Pin);
        assert_eq!(four_pin.to_id(), Ok(12));
        let seven_pin = MahjongTileValue::Number(7, MahjongTileSuit::Pin);
        assert_eq!(seven_pin.to_id(), Ok(15));

        let two_sou = MahjongTileValue::Number(2, MahjongTileSuit::Sou);
        assert_eq!(two_sou.to_id(), Ok(19));
        let six_sou = MahjongTileValue::Number(6, MahjongTileSuit::Sou);
        assert_eq!(six_sou.to_id(), Ok(23));
        let eight_sou = MahjongTileValue::Number(8, MahjongTileSuit::Sou);
        assert_eq!(eight_sou.to_id(), Ok(25));

        let east_wind = MahjongTileValue::Wind(1);
        assert_eq!(east_wind.to_id(), Ok(27));
        let south_wind = MahjongTileValue::Wind(2);
        assert_eq!(south_wind.to_id(), Ok(28));
        let west_wind = MahjongTileValue::Wind(3);
        assert_eq!(west_wind.to_id(), Ok(29));
        let north_wind = MahjongTileValue::Wind(4);
        assert_eq!(north_wind.to_id(), Ok(30));
        let white_dragon = MahjongTileValue::Dragon(5);
        assert_eq!(white_dragon.to_id(), Ok(31));
        let green_dragon = MahjongTileValue::Dragon(6);
        assert_eq!(green_dragon.to_id(), Ok(32));
        let red_dragon = MahjongTileValue::Dragon(7);
        assert_eq!(red_dragon.to_id(), Ok(33));

        // invalid to_id values:
        let invalid_man = MahjongTileValue::Number(0, MahjongTileSuit::Man);
        assert!(invalid_man.to_id().is_err());
        let invalid_sou = MahjongTileValue::Number(10, MahjongTileSuit::Sou);
        assert!(invalid_sou.to_id().is_err());
        let invalid_pin = MahjongTileValue::Number(11, MahjongTileSuit::Pin);
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
        let one_man = MahjongTileValue::Number(1, MahjongTileSuit::Man);
        assert_eq!(MahjongTileValue::from_id(0), Ok(one_man));
        let five_man = MahjongTileValue::Number(5, MahjongTileSuit::Man);
        assert_eq!(MahjongTileValue::from_id(4), Ok(five_man));
        let nine_man = MahjongTileValue::Number(9, MahjongTileSuit::Man);
        assert_eq!(MahjongTileValue::from_id(8), Ok(nine_man));

        let three_pin = MahjongTileValue::Number(3, MahjongTileSuit::Pin);
        assert_eq!(MahjongTileValue::from_id(11), Ok(three_pin));
        let four_pin = MahjongTileValue::Number(4, MahjongTileSuit::Pin);
        assert_eq!(MahjongTileValue::from_id(12), Ok(four_pin));
        let seven_pin = MahjongTileValue::Number(7, MahjongTileSuit::Pin);
        assert_eq!(MahjongTileValue::from_id(15), Ok(seven_pin));

        let two_sou = MahjongTileValue::Number(2, MahjongTileSuit::Sou);
        assert_eq!(MahjongTileValue::from_id(19), Ok(two_sou));
        let six_sou = MahjongTileValue::Number(6, MahjongTileSuit::Sou);
        assert_eq!(MahjongTileValue::from_id(23), Ok(six_sou));
        let eight_sou = MahjongTileValue::Number(8, MahjongTileSuit::Sou);
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
            one_man_tile.value == MahjongTileValue::Number(1, MahjongTileSuit::Man)
                && !one_man_tile.is_red
        );
        let three_pin = MahjongTile::from_text("3p");
        assert!(three_pin.is_ok());
        let three_pin_tile = three_pin.unwrap();
        assert!(
            three_pin_tile.value == MahjongTileValue::Number(3, MahjongTileSuit::Pin)
                && !three_pin_tile.is_red
        );
        let five_sou = MahjongTile::from_text("5s");
        assert!(five_sou.is_ok());
        let five_sou_tile = five_sou.unwrap();
        assert!(
            five_sou_tile.value == MahjongTileValue::Number(5, MahjongTileSuit::Sou)
                && !five_sou_tile.is_red
        );
        let red_five_sou = MahjongTile::from_text("0s");
        assert!(red_five_sou.is_ok());
        let red_five_sou_tile = red_five_sou.unwrap();
        assert!(
            red_five_sou_tile.value == MahjongTileValue::Number(5, MahjongTileSuit::Sou)
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
}
