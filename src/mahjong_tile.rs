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

const NUM_DISTINCT_TILE_VALUES: u8 = 34;
const FIRST_PINZU_ID: u8 = 9;
const FIRST_SOUZU_ID: u8 = 18;
const FIRST_WIND_ID: u8 = 27;
const FIRST_DRAGON_ID: u8 = 31;

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
                    id, NUM_DISTINCT_TILE_VALUES
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

// #[derive(Clone, Copy, Debug)]
// pub struct MahjongTile {
//     pub tile_value: MahjongTileValue,
//     pub is_red: bool,
//     pub modifier: MahjongTileModifier,
// }

// impl PartialEq for MahjongTile {
//     fn eq(&self, other: &Self) -> bool {
//         // ignore modifiers, just check that the suit and rank are identical
//         self.suit == other.suit && self.rank == other.rank
//     }
// }

// impl MahjongTile {
//     pub fn from_mspz(mspz_string: &str) -> Result<Self, &'static str> {
//         if mspz_string.len() != 2 {
//             return Err("Expect mspz string to be 2-character string");
//         }
//         let mut chars = mspz_string.chars();
//         let tile_rank_char = match chars.next() {
//             Some(x) => x,
//             None => return Err("could not read first char"),
//         };
//         let tile_suit_char = match chars.next() {
//             Some(x) => x,
//             None => return Err("could not read second char"),
//         };
//         let tile_suit = match tile_suit_char {
//             // first determine the suit
//             's' => MahjongTileSuit::Sou,
//             'p' => MahjongTileSuit::Pin,
//             'm' => MahjongTileSuit::Man,
//             'z' => MahjongTileSuit::Honor,
//             _ => return Err("invalid suit"),
//         };
//         let tile_rank = match tile_suit {
//             suit if suit.is_number_suit() => {
//                 match tile_rank_char {
//                     '1' => MahjongTileRank::One,
//                     '2' => MahjongTileRank::Two,
//                     '3' => MahjongTileRank::Three,
//                     '4' => MahjongTileRank::Four,
//                     '5' => MahjongTileRank::Five,
//                     '6' => MahjongTileRank::Six,
//                     '7' => MahjongTileRank::Seven,
//                     '8' => MahjongTileRank::Eight,
//                     '9' => MahjongTileRank::Nine,
//                     '0' => MahjongTileRank::Five, // red five
//                     _ => return Err("invalid rank for number suit"),
//                 }
//             }
//             suit if suit.is_honor_suit() => match tile_rank_char {
//                 '1' => MahjongTileRank::East,
//                 '2' => MahjongTileRank::South,
//                 '3' => MahjongTileRank::West,
//                 '4' => MahjongTileRank::North,
//                 '5' => MahjongTileRank::White,
//                 '6' => MahjongTileRank::Green,
//                 '7' => MahjongTileRank::Red,
//                 _ => return Err("invalid rank for honor suit"),
//             },
//             _ => return Err("cannot determine tile rank"),
//         };

//         let modifier = if tile_suit.is_number_suit() && tile_rank_char == '0' {
//             MahjongTileModifier::RedTile
//         } else {
//             MahjongTileModifier::None
//         };

//         Ok(MahjongTile {
//             suit: tile_suit,
//             rank: tile_rank,
//             modifier: modifier,
//         })
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

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

    // #[test]
    // fn tile_number_rank_parse() {
    //     let rank_num: u8 = 3;
    //     let rank = MahjongTileRank::Three;
    //     assert_eq!(MahjongTileRank::try_from(rank_num), Ok(rank));
    //     assert_eq!(u8::try_from(rank), Ok(rank_num));

    //     let rank_num_out_of_range: u8 = 10;
    //     let rank_not_numeric = MahjongTileRank::North;
    //     assert!(MahjongTileRank::try_from(rank_num_out_of_range).is_err());
    //     assert!(u8::try_from(rank_not_numeric).is_err());
    // }

    // #[test]
    // fn tile_equality() {
    //     let tile_five = MahjongTile {
    //         suit: MahjongTileSuit::Man,
    //         rank: MahjongTileRank::Five,
    //         modifier: MahjongTileModifier::None,
    //     };
    //     let other_tile_five = MahjongTile {
    //         suit: MahjongTileSuit::Man,
    //         rank: MahjongTileRank::Five,
    //         modifier: MahjongTileModifier::None,
    //     };
    //     let other_suit_five = MahjongTile {
    //         suit: MahjongTileSuit::Sou,
    //         rank: MahjongTileRank::Five,
    //         modifier: MahjongTileModifier::None,
    //     };
    //     let tile_red_five = MahjongTile {
    //         suit: MahjongTileSuit::Man,
    //         rank: MahjongTileRank::Five,
    //         modifier: MahjongTileModifier::None,
    //     };
    //     assert_eq!(tile_five, other_tile_five);
    //     assert_ne!(tile_five, other_suit_five);
    //     assert_eq!(tile_five, tile_red_five);
    // }

    // #[test]
    // fn tile_from_mspz_string() {
    //     let tile_nine = MahjongTile {
    //         suit: MahjongTileSuit::Man,
    //         rank: MahjongTileRank::Nine,
    //         modifier: MahjongTileModifier::None,
    //     };
    //     assert_eq!(MahjongTile::from_mspz("9m"), Ok(tile_nine));

    //     let tile_red_five: MahjongTile = MahjongTile {
    //         suit: MahjongTileSuit::Sou,
    //         rank: MahjongTileRank::Five,
    //         modifier: MahjongTileModifier::RedTile,
    //     };
    //     assert_eq!(MahjongTile::from_mspz("0s"), Ok(tile_red_five));

    //     let tile_white_dragon: MahjongTile = MahjongTile {
    //         suit: MahjongTileSuit::Honor,
    //         rank: MahjongTileRank::White,
    //         modifier: MahjongTileModifier::None,
    //     };
    //     assert_eq!(MahjongTile::from_mspz("5z"), Ok(tile_white_dragon));
    // }
}
