// rethinking the Tile class, to generalize to other mahjong variants:
// red fives, red threes (rarer than red fives), flower tiles (certain variants), joker (american mahjong)
// the fundamental aspects: tile suit, tile rank, tile modifiers

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MahjongTileSuit {
    Man,
    Pin,
    Sou,
    Honor,
    // Flower, // used as a bonus tile (not in riichi)
    // Season, // used as a bonus tile (not in riichi)
    // Joker,  // used in American mahjong (not in riichi)
}

pub const ALL_SUITS: [MahjongTileSuit; 4] = [
    MahjongTileSuit::Man,
    MahjongTileSuit::Sou,
    MahjongTileSuit::Pin,
    MahjongTileSuit::Honor,
];
pub const NUMBER_SUITS: [MahjongTileSuit; 3] = [
    MahjongTileSuit::Man,
    MahjongTileSuit::Sou,
    MahjongTileSuit::Pin,
];

impl MahjongTileSuit {
    pub fn is_number_suit(&self) -> bool {
        match self {
            Self::Man => true,
            Self::Pin => true,
            Self::Sou => true,
            _ => false,
        }
    }

    pub fn is_honor_suit(&self) -> bool {
        match self {
            Self::Honor => true,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MahjongTileRank {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    East, // for winds (and flowers / seasons, but not in riichi)
    South,
    West,
    North,
    White, // for dragons
    Green,
    Red,
    // Joker, // for Joker in American mahjong
}

impl TryFrom<u8> for MahjongTileRank {
    type Error = &'static str;

    // converts number to a numeric MahjongTileRank
    fn try_from(rank_as_number: u8) -> Result<Self, Self::Error> {
        match rank_as_number {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            7 => Ok(Self::Seven),
            8 => Ok(Self::Eight),
            9 => Ok(Self::Nine),
            _ => Err("Invalid number for mahjong tile rank!"),
        }
    }
}

impl TryFrom<MahjongTileRank> for u8 {
    type Error = &'static str;

    // converts numeric MahjongTileRank to a number
    fn try_from(rank: MahjongTileRank) -> Result<Self, Self::Error> {
        match rank {
            MahjongTileRank::One => Ok(1),
            MahjongTileRank::Two => Ok(2),
            MahjongTileRank::Three => Ok(3),
            MahjongTileRank::Four => Ok(4),
            MahjongTileRank::Five => Ok(5),
            MahjongTileRank::Six => Ok(6),
            MahjongTileRank::Seven => Ok(7),
            MahjongTileRank::Eight => Ok(8),
            MahjongTileRank::Nine => Ok(9),
            _ => Err("Mahjong tile rank cannot be converted to number!"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MahjongTileModifier {
    None,
    RedTile,
}

#[derive(Clone, Copy, Debug)]
pub struct MahjongTile {
    pub suit: MahjongTileSuit,
    pub rank: MahjongTileRank,
    pub modifier: MahjongTileModifier,
}

impl PartialEq for MahjongTile {
    fn eq(&self, other: &Self) -> bool {
        // ignore modifiers, just check that the suit and rank are identical
        self.suit == other.suit && self.rank == other.rank
    }
}

impl MahjongTile {
    pub fn from_mspz(mspz_string: &str) -> Result<Self, &'static str> {
        if mspz_string.len() != 2 {
            return Err("Expect mspz string to be 2-character string");
        }
        let mut chars = mspz_string.chars();
        let tile_rank_char = match chars.next() {
            Some(x) => x,
            None => return Err("could not read first char"),
        };
        let tile_suit_char = match chars.next() {
            Some(x) => x,
            None => return Err("could not read second char"),
        };
        let tile_suit = match tile_suit_char {
            // first determine the suit
            's' => MahjongTileSuit::Sou,
            'p' => MahjongTileSuit::Pin,
            'm' => MahjongTileSuit::Man,
            'z' => MahjongTileSuit::Honor,
            _ => return Err("invalid suit"),
        };
        let tile_rank = match tile_suit {
            suit if suit.is_number_suit() => {
                match tile_rank_char {
                    '1' => MahjongTileRank::One,
                    '2' => MahjongTileRank::Two,
                    '3' => MahjongTileRank::Three,
                    '4' => MahjongTileRank::Four,
                    '5' => MahjongTileRank::Five,
                    '6' => MahjongTileRank::Six,
                    '7' => MahjongTileRank::Seven,
                    '8' => MahjongTileRank::Eight,
                    '9' => MahjongTileRank::Nine,
                    '0' => MahjongTileRank::Five, // red five
                    _ => return Err("invalid rank for number suit"),
                }
            }
            suit if suit.is_honor_suit() => match tile_rank_char {
                '1' => MahjongTileRank::East,
                '2' => MahjongTileRank::South,
                '3' => MahjongTileRank::West,
                '4' => MahjongTileRank::North,
                '5' => MahjongTileRank::White,
                '6' => MahjongTileRank::Green,
                '7' => MahjongTileRank::Red,
                _ => return Err("invalid rank for honor suit"),
            },
            _ => return Err("cannot determine tile rank"),
        };

        let modifier = if tile_suit.is_number_suit() && tile_rank_char == '0' {
            MahjongTileModifier::RedTile
        } else {
            MahjongTileModifier::None
        };

        Ok(MahjongTile {
            suit: tile_suit,
            rank: tile_rank,
            modifier: modifier,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_number_rank_parse() {
        let rank_num: u8 = 3;
        let rank = MahjongTileRank::Three;
        assert_eq!(MahjongTileRank::try_from(rank_num), Ok(rank));
        assert_eq!(u8::try_from(rank), Ok(rank_num));

        let rank_num_out_of_range: u8 = 10;
        let rank_not_numeric = MahjongTileRank::North;
        assert!(MahjongTileRank::try_from(rank_num_out_of_range).is_err());
        assert!(u8::try_from(rank_not_numeric).is_err());
    }

    #[test]
    fn tile_equality() {
        let tile_five = MahjongTile {
            suit: MahjongTileSuit::Man,
            rank: MahjongTileRank::Five,
            modifier: MahjongTileModifier::None,
        };
        let other_tile_five = MahjongTile {
            suit: MahjongTileSuit::Man,
            rank: MahjongTileRank::Five,
            modifier: MahjongTileModifier::None,
        };
        let other_suit_five = MahjongTile {
            suit: MahjongTileSuit::Sou,
            rank: MahjongTileRank::Five,
            modifier: MahjongTileModifier::None,
        };
        let tile_red_five = MahjongTile {
            suit: MahjongTileSuit::Man,
            rank: MahjongTileRank::Five,
            modifier: MahjongTileModifier::None,
        };
        assert_eq!(tile_five, other_tile_five);
        assert_ne!(tile_five, other_suit_five);
        assert_eq!(tile_five, tile_red_five);
    }

    #[test]
    fn tile_from_mspz_string() {
        let tile_nine = MahjongTile {
            suit: MahjongTileSuit::Man,
            rank: MahjongTileRank::Nine,
            modifier: MahjongTileModifier::None,
        };
        assert_eq!(MahjongTile::from_mspz("9m"), Ok(tile_nine));

        let tile_red_five: MahjongTile = MahjongTile {
            suit: MahjongTileSuit::Sou,
            rank: MahjongTileRank::Five,
            modifier: MahjongTileModifier::RedTile,
        };
        assert_eq!(MahjongTile::from_mspz("0s"), Ok(tile_red_five));

        let tile_white_dragon: MahjongTile = MahjongTile {
            suit: MahjongTileSuit::Honor,
            rank: MahjongTileRank::White,
            modifier: MahjongTileModifier::None,
        };
        assert_eq!(MahjongTile::from_mspz("5z"), Ok(tile_white_dragon));
    }
}
