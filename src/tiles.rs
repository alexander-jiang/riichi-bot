use std::fmt;

// using the MPSZ notation, described here: https://ctan.math.utah.edu/ctan/tex-archive/graphics/mahjong/mahjong-code.pdf
pub const TILE_SUITS_CHARS: [char; 4] = ['m', 'p', 's', 'z'];
// number of tiles in a standard riichi mahjong set
pub const NUM_TILES: u32 = 3 * 4 * 9 + 4 * (4 + 3);

/// The possible suits of a tile
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum TileSuit {
    Man,
    Pin,
    Sou,
    Honor,
}

impl TileSuit {
    /// If the tile suit is a numbered suit
    fn is_number(&self) -> bool {
        self == &Self::Man || self == &Self::Pin || self == &Self::Sou
    }
}

impl TryFrom<char> for TileSuit {
    type Error = &'static str;

    /// attempts to parse TileSuit from character using MSPZ notation:
    /// tile suit is a single character `'m'` (man), `'p'` (pin), `'s'` (sou), or `'z'` (honors).
    fn try_from(suit: char) -> Result<Self, Self::Error> {
        match suit {
            'm' => Ok(Self::Man),
            'p' => Ok(Self::Pin),
            's' => Ok(Self::Sou),
            'z' => Ok(Self::Honor),
            _ => Err("Invalid tile suit char!"),
        }
    }
}

impl From<TileSuit> for char {
    /// converts TileSuit to character representation in MSPZ notation:
    /// tile suit is a single character `'m'` (man), `'p'` (pin), `'s'` (sou), or `'z'` (honors).
    fn from(tile_suit: TileSuit) -> char {
        match tile_suit {
            TileSuit::Man => 'm',
            TileSuit::Pin => 'p',
            TileSuit::Sou => 's',
            TileSuit::Honor => 'z',
        }
    }
}

/// The possible ranks (aka values) of a tile in a numbered suit (i.e. man, pin, or sou)
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum NumberTileRank {
    RedFive = 0,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl TryFrom<char> for NumberTileRank {
    type Error = &'static str;

    /// attempts to parse NumberTileRank from character using MSPZ notation:
    /// for numbered suits, tile rank is a single character `'0'` to `'9'` where 0 represents a red five.
    fn try_from(rank: char) -> Result<Self, Self::Error> {
        match rank {
            '0' => Ok(Self::RedFive),
            '1' => Ok(Self::One),
            '2' => Ok(Self::Two),
            '3' => Ok(Self::Three),
            '4' => Ok(Self::Four),
            '5' => Ok(Self::Five),
            '6' => Ok(Self::Six),
            '7' => Ok(Self::Seven),
            '8' => Ok(Self::Eight),
            '9' => Ok(Self::Nine),
            _ => Err("Invalid number tile rank char!"),
        }
    }
}

impl From<NumberTileRank> for char {
    /// converts NumberTileRank to character representation in MSPZ notation:
    /// for numbered suits, tile rank is a single character `'0'` to `'9'` where 0 represents a red five.
    fn from(tile_rank: NumberTileRank) -> char {
        match tile_rank {
            NumberTileRank::RedFive => '0',
            NumberTileRank::One => '1',
            NumberTileRank::Two => '2',
            NumberTileRank::Three => '3',
            NumberTileRank::Four => '4',
            NumberTileRank::Five => '5',
            NumberTileRank::Six => '6',
            NumberTileRank::Seven => '7',
            NumberTileRank::Eight => '8',
            NumberTileRank::Nine => '9',
        }
    }
}

/// The possible ranks (aka values) of a tile in a honor suit (i.e. winds or dragons)
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum HonorTileRank {
    East = 1,
    South,
    West,
    North,
    White,
    Green,
    Red,
}

// conversions from enum to char and vice versa
impl TryFrom<char> for HonorTileRank {
    type Error = &'static str;

    /// attempts to parse HonorTileRank from character using MSPZ notation:
    /// for honor suits, tile rank is a single character `'1'` to `'7'` where 1-4 represents a wind direction
    /// (East, South, West, North, respectively), and 5-7 represents a dragon color (White, Green, Red, respectively).
    fn try_from(rank: char) -> Result<Self, Self::Error> {
        match rank {
            '1' => Ok(Self::East),
            '2' => Ok(Self::South),
            '3' => Ok(Self::West),
            '4' => Ok(Self::North),
            '5' => Ok(Self::White),
            '6' => Ok(Self::Green),
            '7' => Ok(Self::Red),
            _ => Err("Invalid honor tile rank char!"),
        }
    }
}

impl From<HonorTileRank> for char {
    /// converts HonorTileRank to character representation in MSPZ notation:
    /// for honor suits, tile rank is a single character `'1'` to `'7'` where 1-4 represents a wind direction
    /// (East, South, West, North, respectively), and 5-7 represents a dragon color (White, Green, Red, respectively).
    fn from(tile_rank: HonorTileRank) -> char {
        match tile_rank {
            HonorTileRank::East => '1',
            HonorTileRank::South => '2',
            HonorTileRank::West => '3',
            HonorTileRank::North => '4',
            HonorTileRank::White => '5',
            HonorTileRank::Green => '6',
            HonorTileRank::Red => '7',
        }
    }
}

/// The possible ranks (aka values) of a tile
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum TileRank {
    Number(NumberTileRank),
    Honor(HonorTileRank),
}

impl From<TileRank> for char {
    /// converts TileRank (either NumberTileRank or HonorTileRank) to character representation in MSPZ notation:
    /// see documentation for `From<NumberTileRank>` and for `From<HonorTileRank>` for documentation on how tile ranks are represented
    /// for numbered tiles and honor tiles, respectively
    fn from(tile_rank_type: TileRank) -> char {
        match tile_rank_type {
            TileRank::Number(tile_rank) => char::from(tile_rank),
            TileRank::Honor(tile_rank) => char::from(tile_rank),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub serial: u32,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tile {} ({}{})",
            self.serial,
            char::from(self.rank()),
            char::from(self.suit()),
        )
    }
}

impl Tile {
    // TODO use this function when initializing tiles via serial number?
    pub fn is_valid_serial(&self) -> bool {
        self.serial < NUM_TILES
    }

    /// The suit of the tile
    pub fn suit(&self) -> TileSuit {
        if self.serial < (4 * 9) {
            TileSuit::Man
        } else if self.serial >= (4 * 9) && self.serial < (2 * 4 * 9) {
            TileSuit::Pin
        } else if self.serial >= (2 * 4 * 9) && self.serial < (3 * 4 * 9) {
            TileSuit::Sou
        } else {
            TileSuit::Honor
        }
    }

    /// The rank (aka value) of the tile
    pub fn rank(&self) -> TileRank {
        if self.is_number_suit() {
            // numbered suits
            let rank_index = self.serial % 9;

            match rank_index {
                0 => TileRank::Number(NumberTileRank::One),
                1 => TileRank::Number(NumberTileRank::Two),
                2 => TileRank::Number(NumberTileRank::Three),
                3 => TileRank::Number(NumberTileRank::Four),
                4 => {
                    let copy = (self.serial % 36) / 9;
                    // one "red five" tile from each numbered suit
                    // in serial number ordering, the red-five is in the first set of 1-9 per suit
                    if copy < 1 {
                        TileRank::Number(NumberTileRank::RedFive)
                    } else {
                        TileRank::Number(NumberTileRank::Five)
                    }
                }
                5 => TileRank::Number(NumberTileRank::Six),
                6 => TileRank::Number(NumberTileRank::Seven),
                7 => TileRank::Number(NumberTileRank::Eight),
                8 => TileRank::Number(NumberTileRank::Nine),
                _ => panic!("Invalid rank index for number tile"),
            }
        } else {
            // must be an honor tile
            let rank_index = (self.serial - (3 * 36)) % 7;

            match rank_index {
                0 => TileRank::Honor(HonorTileRank::East),
                1 => TileRank::Honor(HonorTileRank::South),
                2 => TileRank::Honor(HonorTileRank::West),
                3 => TileRank::Honor(HonorTileRank::North),
                4 => TileRank::Honor(HonorTileRank::White),
                5 => TileRank::Honor(HonorTileRank::Green),
                6 => TileRank::Honor(HonorTileRank::Red),
                _ => panic!("Invalid rank index for honor tile"),
            }
        }
    }

    /// The numerical value of a tile for the purposes of computing a tile sequence meld/group.
    /// For a numbered tile, returns a Some<u32> corresponding to the tile's rank e.g. for 3-man, `Some(3u32)`.
    /// Otherwise, returns None.
    pub fn sequence_rank_num(&self) -> Option<u32> {
        if !self.is_number_suit() {
            None
        } else {
            let char_digit = char::from(self.rank())
                .to_digit(10)
                .expect("Invalid numbered tile rank char!");
            if char_digit == 0 {
                // red five is still a five tile, just represented with a 0
                Some(5)
            } else {
                Some(char_digit)
            }
        }
    }

    // TODO helper function for char::from(self.rank()) and char::from(self.suit())?

    /// A human-readable suit (not MSPZ notation), a single character.
    /// Same as MSPZ for numbered suits. Uses `'w'` for winds, and `'d'` for dragons.
    pub fn human_suit(&self) -> char {
        if self.is_number_suit() {
            char::from(self.suit())
        } else if self.is_honor() {
            // must be honor tile
            assert!(matches!(self.rank(), TileRank::Honor(_)));
            match self.rank() {
                TileRank::Honor(tile_rank) => match tile_rank {
                    HonorTileRank::East => 'w',
                    HonorTileRank::South => 'w',
                    HonorTileRank::West => 'w',
                    HonorTileRank::North => 'w',
                    HonorTileRank::White => 'd',
                    HonorTileRank::Green => 'd',
                    HonorTileRank::Red => 'd',
                },
                _ => panic!(
                    "rank for honor tile must be TileRank::Honor! serial={}",
                    self.serial
                ),
            }
        } else {
            panic!("Invalid tile serial number! {}", self.serial);
        }
    }

    /// A human-readable rank (not MSPZ notation), a single character.
    /// For numbered suits: `'1'` to `'9'`, and `'0'` for red five. Honor tiles are their first letter in english (winds: E, S, W, N; dragons: W, G, R)
    pub fn human_rank(&self) -> char {
        if self.is_number_suit() {
            char::from(self.rank())
        } else if self.is_honor() {
            // must be honor tile
            assert!(matches!(self.rank(), TileRank::Honor(_)));
            match self.rank() {
                TileRank::Honor(tile_rank) => match tile_rank {
                    HonorTileRank::East => 'E',
                    HonorTileRank::South => 'S',
                    HonorTileRank::West => 'W',
                    HonorTileRank::North => 'N',
                    HonorTileRank::White => 'W',
                    HonorTileRank::Green => 'G',
                    HonorTileRank::Red => 'R',
                },
                _ => panic!(
                    "rank for honor tile must be TileRank::Honor! serial={}",
                    self.serial
                ),
            }
        } else {
            panic!("Invalid tile serial number! {}", self.serial);
        }
    }

    /// Represents the tile using MSPZ notation as a two character string: the rank followed by the suit
    /// e.g. for 8-man: "8m" , for red-5-sou, "0s", for north wind: "4z", for red dragon: "7z"
    pub fn to_string(&self) -> String {
        let mut tile_string = String::new();
        tile_string.push(char::from(self.rank()));
        tile_string.push(char::from(self.suit()));
        tile_string
    }

    /// Represents the tile using human-readable notation as a two character string: the rank followed by the suit
    /// e.g. for 8-man: "8m" , for red-5-sou, "0s", for north wind: "Nw", for red dragon: "Rd"
    pub fn to_human_string(&self) -> String {
        let mut tile_string = String::new();
        tile_string.push(self.human_rank());
        tile_string.push(self.human_suit());
        tile_string
    }

    // constructors

    /// Constructs a Tile from its suit and rank (in MSPZ notation)
    /// e.g. 'm', '7' -> 7-man; 's', '0' -> red-5-sou; 'z', '1' -> East wind
    pub fn from_suit_and_rank(suit: TileSuit, rank: TileRank, copy: u32) -> Self {
        // compute serial number based on suit and rank
        // assert suit is valid ([mpsz])
        // assert rank is valid ([0-9])
        // assert copy is valid (0-3)
        if suit == TileSuit::Man || suit == TileSuit::Pin || suit == TileSuit::Sou {
            assert!(matches!(rank, TileRank::Number(_)));
            let suit_serial_start = match suit {
                TileSuit::Man => 0 * (4 * 9),
                TileSuit::Pin => 1 * (4 * 9),
                TileSuit::Sou => 2 * (4 * 9),
                _ => panic!("Invalid suit for numbered tile"),
            };
            let rank_digit = char::from(rank)
                .to_digit(10)
                .expect("Invalid numbered tile rank char (valid ranks are 0-9)");
            if rank_digit == 0 {
                // red fives take the place of the first copy of a five in the serial number ordering
                assert!(copy == 0, "One copy of red fives only");
                Self {
                    serial: suit_serial_start + 4,
                }
            } else if rank_digit == 5 {
                // the first copy of non-red-fives are alongside the second copy of non-five numbered tiles (by serial number)
                assert!(copy < 3, "Only three copies of non-red fives");
                Self {
                    serial: suit_serial_start + 4 + (copy + 1) * 9,
                }
            } else {
                assert!(
                    copy < 4,
                    "Only four copies of numbered tiles (except fives)"
                );
                Self {
                    serial: suit_serial_start + (rank_digit - 1) + copy * 9,
                }
            }
        } else if suit == TileSuit::Honor {
            assert!(matches!(rank, TileRank::Honor(_)));
            assert!(copy < 4, "Only four copies of honor tiles");
            let rank_digit = char::from(rank)
                .to_digit(8)
                .expect("Invalid honor tile rank char (valid ranks are 1-7)");
            assert!(
                rank_digit >= 1 && rank_digit <= 7,
                "Invalid honor tile rank char (valid ranks are 1-7)"
            );
            Self {
                serial: 3 * (4 * 9) + (rank_digit - 1) + copy * 7,
            }
        } else {
            panic!("Invalid tile suit {:?}", suit);
        }
    }

    /// Constructs a Tile from the same 2-character representation used by `to_string()`
    /// e.g. "7m" -> 7-man
    pub fn from_string(tile_string: &str) -> Self {
        // parse into suit and rank
        // assert tile_string length is 2
        let mut tile_chars = tile_string.chars();
        let rank_char = tile_chars
            .next()
            .expect("Must have tile rank char in string");
        let suit_char = tile_chars
            .next()
            .expect("Must have tile suit char in string");

        let tile_suit = TileSuit::try_from(suit_char).expect("Failed conversion to TileSuit");

        let rank = if tile_suit.is_number() {
            let tile_rank = NumberTileRank::try_from(rank_char);
            let number_tile_rank = tile_rank.expect("Invalid number tile rank");
            TileRank::Number(number_tile_rank)
        } else {
            let tile_rank = HonorTileRank::try_from(rank_char);
            let honor_tile_rank = tile_rank.expect("Invalid honor tile rank");
            TileRank::Honor(honor_tile_rank)
        };

        // TODO why should the copy always be 0?
        Self::from_suit_and_rank(tile_suit, rank, 0)
    }

    // helper functions

    /// If the tile is in a numbered suit (man, pin, or sou)
    pub fn is_number_suit(&self) -> bool {
        // TODO how to enforce that if Tile.suit is a number suit, the tile_rank is TileRank::Number(_)? and vice versa for honor tiles
        self.suit() == TileSuit::Man || self.suit() == TileSuit::Pin || self.suit() == TileSuit::Sou
    }

    /// If the tile is rank 1 or 9 in a numbered suit
    pub fn is_terminal(&self) -> bool {
        // example yaku:
        // - chanta (at least 1 terminal or honor tile in each meld and in the pair)
        // - junchan (at least 1 terminal tile in each meld and in the pair)
        // - chinroutou (hand is entirely terminal tiles)
        (char::from(self.rank()) == '1' || char::from(self.rank()) == '9') && self.is_number_suit()
    }

    /// If the tile is a wind tile or a dragon tile (honor tiles are also known as word tiles)
    pub fn is_honor(&self) -> bool {
        // example yaku:
        // - honroutou (hand is entirely terminal or honor tiles)
        // - tsuuiisou (hand is entirely honor tiles)
        self.suit() == TileSuit::Honor
    }

    /// If the tile is a dragon tile (all dragons count for yakuhai)
    pub fn is_dragon(&self) -> bool {
        self.suit() == TileSuit::Honor
            && (self.rank() == TileRank::Honor(HonorTileRank::Green)
                || self.rank() == TileRank::Honor(HonorTileRank::Red)
                || self.rank() == TileRank::Honor(HonorTileRank::White))
    }

    /// Is this tile a dora tile based on the indicator tile?
    pub fn is_dora_from_indicator(&self, dora_indicator: &Tile) -> bool {
        if self.suit() != dora_indicator.suit() {
            return false;
        }
        match self.suit() {
            TileSuit::Honor => {
                let indicated_dora_rank = match dora_indicator.rank() {
                    TileRank::Honor(indicator_rank) => match indicator_rank {
                        HonorTileRank::East => HonorTileRank::South,
                        HonorTileRank::South => HonorTileRank::West,
                        HonorTileRank::West => HonorTileRank::North,
                        HonorTileRank::North => HonorTileRank::East,
                        HonorTileRank::White => HonorTileRank::Green,
                        HonorTileRank::Green => HonorTileRank::Red,
                        HonorTileRank::Red => HonorTileRank::White,
                    },
                    _ => panic!("Expected dora indicator to be an honor tile!"),
                };
                match self.rank() {
                    TileRank::Honor(self_rank) => self_rank == indicated_dora_rank,
                    _ => panic!("Expected this tile to also be an honor tile!"),
                }
            }
            _ => {
                let indicated_dora_rank = match dora_indicator.rank() {
                    TileRank::Number(indicator_rank) => match indicator_rank {
                        NumberTileRank::One => NumberTileRank::Two,
                        NumberTileRank::Two => NumberTileRank::Three,
                        NumberTileRank::Three => NumberTileRank::Four,
                        NumberTileRank::Four => NumberTileRank::Five,
                        NumberTileRank::RedFive => NumberTileRank::Six,
                        NumberTileRank::Five => NumberTileRank::Six,
                        NumberTileRank::Six => NumberTileRank::Seven,
                        NumberTileRank::Seven => NumberTileRank::Eight,
                        NumberTileRank::Eight => NumberTileRank::Nine,
                        NumberTileRank::Nine => NumberTileRank::One,
                    },
                    _ => panic!("Expected dora indicator to be a number tile!"),
                };
                match self.rank() {
                    TileRank::Number(self_rank) => {
                        // if the dora indicator is a 4, then both red-five and normal-five count as dora tiles
                        if self_rank == NumberTileRank::RedFive
                            && indicated_dora_rank == NumberTileRank::Five
                        {
                            return true;
                        }
                        self_rank == indicated_dora_rank
                    }
                    _ => panic!("Expected this tile to also be a number tile!"),
                }
            }
        }
    }

    /// If the tile is rank 2-8 in a numbered suit, i.e. is not an honor tile or a terminal tile
    pub fn is_simple(&self) -> bool {
        // example yaku:
        // - tanyou (hand is entirely simple tiles)
        !self.is_honor() && !self.is_terminal()
    }

    /// If the tile is painted with only green - i.e. 2,3,4,6,8-sou and green dragon
    pub fn is_all_green(&self) -> bool {
        // used in the ryuuiisou yaku (hand is entirely made of tiles that are all green)
        (self.is_honor() && self.rank() == TileRank::Honor(HonorTileRank::Green))
            || (self.suit() == TileSuit::Sou
                && (self.rank() == TileRank::Number(NumberTileRank::Two)
                    || self.rank() == TileRank::Number(NumberTileRank::Three)
                    || self.rank() == TileRank::Number(NumberTileRank::Four)
                    || self.rank() == TileRank::Number(NumberTileRank::Six)
                    || self.rank() == TileRank::Number(NumberTileRank::Eight)))
    }

    /// If the tile is a red five tile
    pub fn is_red_five(&self) -> bool {
        // used for counting dora
        self.is_number_suit() && self.rank() == TileRank::Number(NumberTileRank::RedFive)
    }
}

/// A group of tiles - used for identifying winning hand shapes (generally, 4 complete groups and a pair),
/// and for classifying their value (based on yaku list)
#[derive(Debug, Clone)]
pub enum TileGroup {
    /// three tiles with the same suit and same rank
    Triplet {
        open: bool,
        // TODO how to ensure the tiles in this triplet are all the same rank?
        tiles: [Tile; 3],
    },
    /// four tiles with the same suit and same rank
    Quad {
        open: bool,
        added: bool,
        // TODO how to ensure the tiles in this quad are all the same rank?
        tiles: [Tile; 4],
    },
    /// three tiles in a numbered suit with sequentially increasing rank e.g. 1-2-3 or 6-7-8
    Sequence {
        open: bool,
        // TODO how to ensure the tiles in this sequence form a valid sequence?
        tiles: [Tile; 3],
    },
    /// two tiles with the same suit and same rank
    Pair { tiles: [Tile; 2] },
    /// two tiles in a numbered suit that are adjacent ranks and do not include a terminal: e.g. 2-3, 4-5, 7-8
    OpenWait { tiles: [Tile; 2] },
    /// two tiles in a numbered suit that are separated by one rank: e.g. 1-3, 4-6, 5-7
    ClosedWait { tiles: [Tile; 2] },
    /// two tiles at the edge of a numbered suit: 1-2 or 8-9
    EdgeWait { tiles: [Tile; 2] },
    /// a single tile that isn't connected to another
    SingleTile { tile: Tile },
}

impl TileGroup {
    /// Is the tile group complete? (i.e. a triplet, a quad, or a sequence)
    /// Generally, a winning hand requires 4 complete groups (aka melds) and a pair
    pub fn is_complete(&self) -> bool {
        match self {
            Self::Triplet { .. } => true,
            Self::Quad { .. } => true,
            Self::Sequence { .. } => true,
            // all other groups are incomplete (even the Pair, despite all winning hands requiring a Pair)
            _ => false,
        }
    }

    /// Is the tile group open?
    /// A tile group being open (instead of closed) may disqualify a hand from winning, or may reduce a winning hand's value.
    pub fn is_open(&self) -> bool {
        match self {
            Self::Triplet { open, .. } => *open,
            Self::Quad { open, .. } => *open,
            Self::Sequence { open, .. } => *open,
            // all other groups cannot be open by the rules
            _ => false,
        }
    }

    // TODO check if group is valid - shouldn't we enforce this on construction?
    // TODO can refactor some of the common/duplicated code in this function
    // TODO write tests for this function
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Triplet { tiles, .. } => {
                // check that all tiles have the same rank & suit
                let rank = tiles[0].rank();
                let suit = tiles[0].suit();
                for index in 1..3 {
                    if tiles[index].rank() != rank || tiles[index].suit() != suit {
                        return false;
                    }
                }
                true
            }
            Self::Quad { open, added, tiles } => {
                // an added-quad cannot be closed
                if *added && !(*open) {
                    return false;
                }

                // check that all tiles have the same rank & suit
                let rank = tiles[0].rank();
                let suit = tiles[0].suit();
                for index in 1..4 {
                    if tiles[index].rank() != rank || tiles[index].suit() != suit {
                        return false;
                    }
                }
                true
            }
            Self::Sequence { tiles, .. } => {
                let suit = tiles[0].suit();
                // all tiles must be in the same numbered suit (no sequences possible in honors suits)
                if !suit.is_number() {
                    return false;
                }
                for index in 1..3 {
                    if tiles[index].suit() != suit {
                        return false;
                    }
                }

                // check that the tile ranks increase sequentially
                let rank0 = tiles[0]
                    .sequence_rank_num()
                    .expect("Tile should be in a numbered suit");
                let rank1 = tiles[1]
                    .sequence_rank_num()
                    .expect("Tile should be in a numbered suit");
                let rank2 = tiles[2]
                    .sequence_rank_num()
                    .expect("Tile should be in a numbered suit");
                let mut tile_seq_nums = [rank0, rank1, rank2];
                tile_seq_nums.sort();
                rank0 + 1 == rank1 && rank1 + 1 == rank2
            }
            Self::Pair { tiles } => {
                // check that both tiles have the same rank & suit
                let rank = tiles[0].rank();
                let suit = tiles[0].suit();
                tiles[1].rank() == rank && tiles[1].suit() == suit
            }
            Self::OpenWait { tiles, .. } => {
                // both tiles must be in the same numbered suit (no sequences possible in honors suits)
                let suit = tiles[0].suit();
                if !suit.is_number() || tiles[1].suit() != suit {
                    return false;
                }

                // neither tile can be a terminal tile
                if tiles[0].is_terminal() || tiles[1].is_terminal() {
                    return false;
                }

                // check that both tiles have adjacent ranks
                let rank0 = tiles[0]
                    .sequence_rank_num()
                    .expect("Tile should be in a numbered suit");
                let rank1 = tiles[1]
                    .sequence_rank_num()
                    .expect("Tile should be in a numbered suit");
                let mut tile_seq_nums = [rank0, rank1];
                tile_seq_nums.sort();
                rank0 + 1 == rank1
            }
            Self::ClosedWait { tiles, .. } => {
                // both tiles must be in the same numbered suit (no sequences possible in honors suits)
                let suit = tiles[0].suit();
                if !suit.is_number() || tiles[1].suit() != suit {
                    return false;
                }

                // check that both tiles are separated by 1 rank
                let rank0 = tiles[0]
                    .sequence_rank_num()
                    .expect("Tile should be in a numbered suit");
                let rank1 = tiles[1]
                    .sequence_rank_num()
                    .expect("Tile should be in a numbered suit");
                let mut tile_seq_nums = [rank0, rank1];
                tile_seq_nums.sort();
                rank0 + 2 == rank1
            }
            Self::EdgeWait { tiles, .. } => {
                // both tiles must be in the same numbered suit (no sequences possible in honors suits)
                let suit = tiles[0].suit();
                if !suit.is_number() || tiles[1].suit() != suit {
                    return false;
                }

                // of the two tiles, exactly one tile must be a terminal tile
                if (tiles[0].is_terminal() && tiles[1].is_terminal())
                    || (!tiles[0].is_terminal() && !tiles[1].is_terminal())
                {
                    return false;
                }

                // check that both tiles have adjacent ranks
                let rank0 = tiles[0]
                    .sequence_rank_num()
                    .expect("Tile should be in a numbered suit");
                let rank1 = tiles[1]
                    .sequence_rank_num()
                    .expect("Tile should be in a numbered suit");
                let mut tile_seq_nums = [rank0, rank1];
                tile_seq_nums.sort();
                rank0 + 1 == rank1
            }
            Self::SingleTile { .. } => true,
        }
    }
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;

    use std::collections::HashMap;

    #[test]
    fn test_convert_between_tile_suit_enum_and_char() {
        // char to TileSuit via TileSuit::try_from()
        assert_eq!(TileSuit::try_from('m'), Ok(TileSuit::Man));
        assert_eq!(TileSuit::try_from('p'), Ok(TileSuit::Pin));
        assert_eq!(TileSuit::try_from('s'), Ok(TileSuit::Sou));
        assert_eq!(TileSuit::try_from('z'), Ok(TileSuit::Honor));
        assert!(TileSuit::try_from('d').is_err());

        // TileSuit to char via char::from()
        assert_eq!(char::from(TileSuit::Man), 'm');
        assert_eq!(char::from(TileSuit::Pin), 'p');
        assert_eq!(char::from(TileSuit::Sou), 's');
        assert_eq!(char::from(TileSuit::Honor), 'z');
    }

    #[test]
    fn test_convert_between_number_tile_rank_enum_and_char() {
        // char to NumberTileRank via NumberTileRank::try_from()
        assert_eq!(NumberTileRank::try_from('0'), Ok(NumberTileRank::RedFive));
        assert_eq!(NumberTileRank::try_from('1'), Ok(NumberTileRank::One));
        assert_eq!(NumberTileRank::try_from('2'), Ok(NumberTileRank::Two));
        assert_eq!(NumberTileRank::try_from('3'), Ok(NumberTileRank::Three));
        assert_eq!(NumberTileRank::try_from('4'), Ok(NumberTileRank::Four));
        assert_eq!(NumberTileRank::try_from('5'), Ok(NumberTileRank::Five));
        assert_eq!(NumberTileRank::try_from('6'), Ok(NumberTileRank::Six));
        assert_eq!(NumberTileRank::try_from('7'), Ok(NumberTileRank::Seven));
        assert_eq!(NumberTileRank::try_from('8'), Ok(NumberTileRank::Eight));
        assert_eq!(NumberTileRank::try_from('9'), Ok(NumberTileRank::Nine));
        assert!(NumberTileRank::try_from('a').is_err());

        // NumberTileRank to char via char::from()
        assert_eq!(char::from(NumberTileRank::RedFive), '0');
        assert_eq!(char::from(NumberTileRank::One), '1');
        assert_eq!(char::from(NumberTileRank::Two), '2');
        assert_eq!(char::from(NumberTileRank::Three), '3');
        assert_eq!(char::from(NumberTileRank::Four), '4');
        assert_eq!(char::from(NumberTileRank::Five), '5');
        assert_eq!(char::from(NumberTileRank::Six), '6');
        assert_eq!(char::from(NumberTileRank::Seven), '7');
        assert_eq!(char::from(NumberTileRank::Eight), '8');
        assert_eq!(char::from(NumberTileRank::Nine), '9');
    }

    #[test]
    fn test_convert_between_honor_tile_rank_enum_and_char() {
        // char to HonorTileRank via HonorTileRank::try_from()
        assert_eq!(HonorTileRank::try_from('1'), Ok(HonorTileRank::East));
        assert_eq!(HonorTileRank::try_from('2'), Ok(HonorTileRank::South));
        assert_eq!(HonorTileRank::try_from('3'), Ok(HonorTileRank::West));
        assert_eq!(HonorTileRank::try_from('4'), Ok(HonorTileRank::North));
        assert_eq!(HonorTileRank::try_from('5'), Ok(HonorTileRank::White));
        assert_eq!(HonorTileRank::try_from('6'), Ok(HonorTileRank::Green));
        assert_eq!(HonorTileRank::try_from('7'), Ok(HonorTileRank::Red));
        assert!(HonorTileRank::try_from('0').is_err());
        assert!(HonorTileRank::try_from('8').is_err());

        // HonorTileRank to char via char::from()
        assert_eq!(char::from(HonorTileRank::East), '1');
        assert_eq!(char::from(HonorTileRank::South), '2');
        assert_eq!(char::from(HonorTileRank::West), '3');
        assert_eq!(char::from(HonorTileRank::North), '4');
        assert_eq!(char::from(HonorTileRank::White), '5');
        assert_eq!(char::from(HonorTileRank::Green), '6');
        assert_eq!(char::from(HonorTileRank::Red), '7');
    }

    #[test]
    fn test_tile_from_and_to_suit_rank() {
        // 1-man, first copy, serial number 0
        let man_tile =
            Tile::from_suit_and_rank(TileSuit::Man, TileRank::Number(NumberTileRank::One), 0);
        assert_eq!(char::from(man_tile.suit()), 'm');
        assert_eq!(char::from(man_tile.rank()), '1');
        assert_eq!(man_tile.serial, 0 + 0 + 0 * 9);

        // red-5-man, serial number 4
        let man_red_five =
            Tile::from_suit_and_rank(TileSuit::Man, TileRank::Number(NumberTileRank::RedFive), 0);
        assert_eq!(char::from(man_red_five.suit()), 'm');
        assert_eq!(char::from(man_red_five.rank()), '0');
        assert_eq!(man_red_five.serial, 0 + 4 + 0 * 9);

        // 5-man, first copy, serial number 13 (since the red-5-man is serial number 4)
        let man_red_five =
            Tile::from_suit_and_rank(TileSuit::Man, TileRank::Number(NumberTileRank::Five), 0);
        assert_eq!(char::from(man_red_five.suit()), 'm');
        assert_eq!(char::from(man_red_five.rank()), '5');
        assert_eq!(man_red_five.serial, 0 + 4 + 1 * 9);

        // 5-man, third copy, serial number 31 (since the red-5-man is serial number 4)
        let man_red_five =
            Tile::from_suit_and_rank(TileSuit::Man, TileRank::Number(NumberTileRank::Five), 2);
        assert_eq!(char::from(man_red_five.suit()), 'm');
        assert_eq!(char::from(man_red_five.rank()), '5');
        assert_eq!(man_red_five.serial, 0 + 4 + 3 * 9);

        // 4-pin, third copy, serial number 57
        let pin_tile =
            Tile::from_suit_and_rank(TileSuit::Pin, TileRank::Number(NumberTileRank::Four), 2);
        assert_eq!(char::from(pin_tile.suit()), 'p');
        assert_eq!(char::from(pin_tile.rank()), '4');
        assert_eq!(pin_tile.serial, (4 * 9) + 3 + 2 * 9);

        // 9-sou, fourth copy, serial number 107
        let sou_tile =
            Tile::from_suit_and_rank(TileSuit::Sou, TileRank::Number(NumberTileRank::Nine), 3);
        assert_eq!(char::from(sou_tile.suit()), 's');
        assert_eq!(char::from(sou_tile.rank()), '9');
        assert_eq!(sou_tile.serial, 2 * (4 * 9) + 8 + 3 * 9);

        // west wind, third copy, serial number 124
        let wind_tile =
            Tile::from_suit_and_rank(TileSuit::Honor, TileRank::Honor(HonorTileRank::West), 2);
        assert_eq!(char::from(wind_tile.suit()), 'z');
        assert_eq!(char::from(wind_tile.rank()), '3');
        assert_eq!(wind_tile.serial, 3 * (4 * 9) + 2 + 2 * 7);

        // red dragon, first copy, serial number 114
        let dragon_tile =
            Tile::from_suit_and_rank(TileSuit::Honor, TileRank::Honor(HonorTileRank::Red), 0);
        assert_eq!(char::from(dragon_tile.suit()), 'z');
        assert_eq!(char::from(dragon_tile.rank()), '7');
        assert_eq!(dragon_tile.serial, 3 * (4 * 9) + 6 + 0 * 7);
    }

    #[test]
    fn test_tile_from_and_to_string() {
        let man_tile = Tile::from_string("1m");
        assert_eq!(char::from(man_tile.suit()), 'm');
        assert_eq!(char::from(man_tile.rank()), '1');
        assert_eq!(man_tile.to_string(), "1m".to_string());

        let pin_tile = Tile::from_string("4p");
        assert_eq!(char::from(pin_tile.suit()), 'p');
        assert_eq!(char::from(pin_tile.rank()), '4');
        assert_eq!(pin_tile.to_string(), "4p".to_string());

        let sou_tile = Tile::from_string("9s");
        assert_eq!(char::from(sou_tile.suit()), 's');
        assert_eq!(char::from(sou_tile.rank()), '9');
        assert_eq!(sou_tile.to_string(), "9s".to_string());

        let east_wind_tile = Tile::from_string("1z");
        assert_eq!(char::from(east_wind_tile.suit()), 'z');
        assert_eq!(char::from(east_wind_tile.rank()), '1');
        assert_eq!(east_wind_tile.to_string(), "1z".to_string());

        let south_wind_tile = Tile::from_string("2z");
        assert_eq!(char::from(south_wind_tile.suit()), 'z');
        assert_eq!(char::from(south_wind_tile.rank()), '2');
        assert_eq!(south_wind_tile.to_string(), "2z".to_string());

        let west_wind_tile = Tile::from_string("3z");
        assert_eq!(char::from(west_wind_tile.suit()), 'z');
        assert_eq!(char::from(west_wind_tile.rank()), '3');
        assert_eq!(west_wind_tile.to_string(), "3z".to_string());

        let north_wind_tile = Tile::from_string("4z");
        assert_eq!(char::from(north_wind_tile.suit()), 'z');
        assert_eq!(char::from(north_wind_tile.rank()), '4');
        assert_eq!(north_wind_tile.to_string(), "4z".to_string());

        let white_dragon_tile = Tile::from_string("5z");
        assert_eq!(char::from(white_dragon_tile.suit()), 'z');
        assert_eq!(char::from(white_dragon_tile.rank()), '5');
        assert_eq!(white_dragon_tile.to_string(), "5z".to_string());

        let green_dragon_tile = Tile::from_string("6z");
        assert_eq!(char::from(green_dragon_tile.suit()), 'z');
        assert_eq!(char::from(green_dragon_tile.rank()), '6');
        assert_eq!(green_dragon_tile.to_string(), "6z".to_string());

        let red_dragon_tile = Tile::from_string("7z");
        assert_eq!(char::from(red_dragon_tile.suit()), 'z');
        assert_eq!(char::from(red_dragon_tile.rank()), '7');
        assert_eq!(red_dragon_tile.to_string(), "7z".to_string());
    }

    #[test]
    fn test_tile_sequence_rank_num() {
        let man_tile = Tile::from_string("1m");
        assert_eq!(man_tile.sequence_rank_num(), Some(1));

        let pin_tile = Tile::from_string("4p");
        assert_eq!(pin_tile.sequence_rank_num(), Some(4));

        let pin_normal_five_tile = Tile::from_string("5p");
        assert_eq!(pin_normal_five_tile.sequence_rank_num(), Some(5));

        // red-five tiles should return Some(5) (red-five tiles are interchangeable with normal-five tiles in sequences)
        let pin_red_five_tile = Tile::from_string("0p");
        assert_eq!(pin_red_five_tile.sequence_rank_num(), Some(5));

        let sou_tile = Tile::from_string("9s");
        assert_eq!(sou_tile.sequence_rank_num(), Some(9));

        // honor tiles should return None (honor tiles cannot form a sequence)
        let east_wind_tile = Tile::from_string("1z");
        assert_eq!(east_wind_tile.sequence_rank_num(), None);

        let south_wind_tile = Tile::from_string("2z");
        assert_eq!(south_wind_tile.sequence_rank_num(), None);

        let west_wind_tile = Tile::from_string("3z");
        assert_eq!(west_wind_tile.sequence_rank_num(), None);

        let north_wind_tile = Tile::from_string("4z");
        assert_eq!(north_wind_tile.sequence_rank_num(), None);

        let white_dragon_tile = Tile::from_string("5z");
        assert_eq!(white_dragon_tile.sequence_rank_num(), None);

        let green_dragon_tile = Tile::from_string("6z");
        assert_eq!(green_dragon_tile.sequence_rank_num(), None);

        let red_dragon_tile = Tile::from_string("7z");
        assert_eq!(red_dragon_tile.sequence_rank_num(), None);
    }

    #[test]
    fn test_tile_human_and_mspz_notation() {
        let man_tile = Tile::from_string("1m");
        assert!(!man_tile.is_honor());
        assert!(!man_tile.is_dragon());
        assert_eq!(man_tile.human_suit(), 'm');
        assert_eq!(man_tile.human_rank(), '1');
        assert_eq!(man_tile.to_human_string(), "1m".to_string());

        let pin_tile = Tile::from_string("4p");
        assert!(!pin_tile.is_honor());
        assert!(!pin_tile.is_dragon());
        assert_eq!(pin_tile.human_suit(), 'p');
        assert_eq!(pin_tile.human_rank(), '4');
        assert_eq!(pin_tile.to_human_string(), "4p".to_string());

        let sou_tile = Tile::from_string("9s");
        assert!(!sou_tile.is_honor());
        assert!(!sou_tile.is_dragon());
        assert_eq!(sou_tile.human_suit(), 's');
        assert_eq!(sou_tile.human_rank(), '9');
        assert_eq!(sou_tile.to_human_string(), "9s".to_string());

        let east_wind_tile = Tile::from_string("1z");
        assert!(east_wind_tile.is_honor());
        assert!(!east_wind_tile.is_dragon());
        assert_eq!(east_wind_tile.human_suit(), 'w');
        assert_eq!(east_wind_tile.human_rank(), 'E');
        assert_eq!(east_wind_tile.to_human_string(), "Ew".to_string());

        let south_wind_tile = Tile::from_string("2z");
        assert!(south_wind_tile.is_honor());
        assert!(!south_wind_tile.is_dragon());
        assert_eq!(south_wind_tile.human_suit(), 'w');
        assert_eq!(south_wind_tile.human_rank(), 'S');
        assert_eq!(south_wind_tile.to_human_string(), "Sw".to_string());

        let west_wind_tile = Tile::from_string("3z");
        assert!(west_wind_tile.is_honor());
        assert!(!west_wind_tile.is_dragon());
        assert_eq!(west_wind_tile.human_suit(), 'w');
        assert_eq!(west_wind_tile.human_rank(), 'W');
        assert_eq!(west_wind_tile.to_human_string(), "Ww".to_string());

        let north_wind_tile = Tile::from_string("4z");
        assert!(north_wind_tile.is_honor());
        assert!(!north_wind_tile.is_dragon());
        assert_eq!(north_wind_tile.human_suit(), 'w');
        assert_eq!(north_wind_tile.human_rank(), 'N');
        assert_eq!(north_wind_tile.to_human_string(), "Nw".to_string());

        let white_dragon_tile = Tile::from_string("5z");
        assert!(white_dragon_tile.is_honor());
        assert!(white_dragon_tile.is_dragon());
        assert_eq!(white_dragon_tile.human_suit(), 'd');
        assert_eq!(white_dragon_tile.human_rank(), 'W');
        assert_eq!(white_dragon_tile.to_human_string(), "Wd".to_string());

        let green_dragon_tile = Tile::from_string("6z");
        assert!(green_dragon_tile.is_honor());
        assert!(green_dragon_tile.is_dragon());
        assert_eq!(green_dragon_tile.human_suit(), 'd');
        assert_eq!(green_dragon_tile.human_rank(), 'G');
        assert_eq!(green_dragon_tile.to_human_string(), "Gd".to_string());

        let red_dragon_tile = Tile::from_string("7z");
        assert!(red_dragon_tile.is_honor());
        assert!(red_dragon_tile.is_dragon());
        assert_eq!(red_dragon_tile.human_suit(), 'd');
        assert_eq!(red_dragon_tile.human_rank(), 'R');
        assert_eq!(red_dragon_tile.to_human_string(), "Rd".to_string());
    }

    // verify the count of tiles of each suit if you iterate through all serial numbers
    #[test]
    fn test_tile_suit_counts() {
        let mut suit_counts = HashMap::new();
        for serial in 0..NUM_TILES {
            let tile = Tile { serial };
            let count = suit_counts.entry(char::from(tile.suit())).or_insert(0);
            *count += 1;
        }

        assert_eq!(suit_counts.get(&'m'), Some(&(4 * 9)));
        assert_eq!(suit_counts.get(&'p'), Some(&(4 * 9)));
        assert_eq!(suit_counts.get(&'s'), Some(&(4 * 9)));
        assert_eq!(suit_counts.get(&'z'), Some(&(4 * (4 + 3))));

        for (key, _val) in suit_counts.iter() {
            if !TILE_SUITS_CHARS.contains(key) {
                panic!("invalid suit found in hash map! {}", key);
            }
        }
    }

    // verify the count of terminal tiles, honor tiles, simple tiles, all green tiles (i.e. test each helper function)
    #[test]
    fn test_terminal_tile_counts() {
        // 3 suits * 2 ranks per suit (1 and 9) * 4 copies
        let expected_num_terminal_tiles: u32 = 3 * 4 * 2;

        let mut num_terminal_tiles: u32 = 0;
        for serial in 0..NUM_TILES {
            let tile = Tile { serial };
            if tile.is_terminal() {
                num_terminal_tiles += 1;
            }
        }
        let num_terminal_tiles = num_terminal_tiles;

        assert_eq!(num_terminal_tiles, expected_num_terminal_tiles);
    }

    #[test]
    fn test_honor_tile_counts() {
        // (4 winds + 3 dragons) * 4 copies
        let expected_num_honor_tiles: u32 = (4 + 3) * 4;

        let mut num_honor_tiles: u32 = 0;
        for serial in 0..NUM_TILES {
            let tile = Tile { serial };
            if tile.is_honor() {
                num_honor_tiles += 1;
            }
        }
        let num_honor_tiles = num_honor_tiles;

        assert_eq!(num_honor_tiles, expected_num_honor_tiles);
    }

    #[test]
    fn test_dragon_tile_counts() {
        // 3 dragons * 4 copies
        let expected_num_dragon_tiles: u32 = 3 * 4;

        let mut num_dragon_tiles: u32 = 0;
        for serial in 0..NUM_TILES {
            let tile = Tile { serial };
            if tile.is_dragon() {
                num_dragon_tiles += 1;
            }
        }
        let num_dragon_tiles = num_dragon_tiles;

        assert_eq!(num_dragon_tiles, expected_num_dragon_tiles);
    }

    #[test]
    fn test_simple_tile_counts() {
        // 3 suits * 7 ranks per suit (2 through 8, inclusive) * 4 copies
        let expected_num_simple_tiles: u32 = 3 * 7 * 4;

        let mut num_simple_tiles: u32 = 0;
        for serial in 0..NUM_TILES {
            let tile = Tile { serial };
            if tile.is_simple() {
                num_simple_tiles += 1;
            }
        }
        let num_simple_tiles = num_simple_tiles;

        assert_eq!(num_simple_tiles, expected_num_simple_tiles);
    }

    #[test]
    fn test_all_green_tile_counts() {
        // 6 tiles (green dragon + 2, 3, 4, 6, 8 sou) * 4 copies
        let expected_num_all_green_tiles: u32 = 6 * 4;

        let mut num_all_green_tiles: u32 = 0;
        for serial in 0..NUM_TILES {
            let tile = Tile { serial };
            if tile.is_all_green() {
                num_all_green_tiles += 1;
            }
        }
        let num_all_green_tiles = num_all_green_tiles;

        assert_eq!(num_all_green_tiles, expected_num_all_green_tiles);
    }

    #[test]
    fn test_red_fives_counts() {
        // 3 tiles are red-fives (one per suit)
        let expected_num_red_fives: u32 = 3;

        let mut num_red_fives: u32 = 0;
        for serial in 0..NUM_TILES {
            let tile = Tile { serial };
            if tile.is_red_five() {
                num_red_fives += 1;
            }
        }
        let num_red_fives = num_red_fives;

        assert_eq!(num_red_fives, expected_num_red_fives);
    }

    #[test]
    fn test_tile_group_is_valid() {
        let valid_triplet_group = TileGroup::Triplet {
            open: true,
            tiles: [
                Tile::from_suit_and_rank(TileSuit::Honor, TileRank::Honor(HonorTileRank::Red), 0),
                Tile::from_suit_and_rank(TileSuit::Honor, TileRank::Honor(HonorTileRank::Red), 1),
                Tile::from_suit_and_rank(TileSuit::Honor, TileRank::Honor(HonorTileRank::Red), 2),
            ],
        };
        assert!(valid_triplet_group.is_valid());
        assert!(valid_triplet_group.is_complete());
        assert!(valid_triplet_group.is_open());

        let invalid_triplet_group = TileGroup::Triplet {
            open: true,
            tiles: [
                Tile::from_suit_and_rank(TileSuit::Honor, TileRank::Honor(HonorTileRank::East), 0),
                Tile::from_suit_and_rank(TileSuit::Honor, TileRank::Honor(HonorTileRank::South), 1),
                Tile::from_suit_and_rank(TileSuit::Honor, TileRank::Honor(HonorTileRank::West), 2),
            ],
        };
        assert!(!invalid_triplet_group.is_valid());

        // TODO add more test cases for different group types (quad, sequences, open wait, etc.)
    }

    #[test]
    fn test_is_dora_from_indicator_four() {
        let indicator = Tile::from_string("4m");
        // indicated dora is 5-man
        assert!(Tile::from_string("5m").is_dora_from_indicator(&indicator));
        // red-5-man also is dora (worth 2 han: 1 from red-five, and 1 from dora)
        assert!(Tile::from_string("0m").is_dora_from_indicator(&indicator));
        // other man tiles are not dora
        assert!(!Tile::from_string("4m").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("3m").is_dora_from_indicator(&indicator));
        // 5-pin, 5-sou, white dragon (5z), and red-fives in pin and sou are not dora
        assert!(!Tile::from_string("5p").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("5s").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("5z").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("0p").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("0s").is_dora_from_indicator(&indicator));
    }

    #[test]
    fn test_is_dora_from_indicator_red_five() {
        let indicator = Tile::from_string("0p");
        // indicated dora is 6-pin
        assert!(Tile::from_string("6p").is_dora_from_indicator(&indicator));
        // other pin tiles are not dora
        assert!(!Tile::from_string("5p").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("4p").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("1p").is_dora_from_indicator(&indicator));
        // 6-man, 6-sou, green dragon (6z) are not dora
        assert!(!Tile::from_string("6m").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("6s").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("6z").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("1m").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("1s").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("1z").is_dora_from_indicator(&indicator));
    }

    #[test]
    fn test_is_dora_from_indicator_normal_five() {
        let indicator = Tile::from_string("5s");
        // indicated dora is 6-sou
        assert!(Tile::from_string("6s").is_dora_from_indicator(&indicator));
        // other sou tiles are not dora
        assert!(!Tile::from_string("0s").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("5s").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("4s").is_dora_from_indicator(&indicator));
        // 6-man, 6-pin, green dragon (6z) are not dora
        assert!(!Tile::from_string("6m").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("6p").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("6z").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("1m").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("1p").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("1z").is_dora_from_indicator(&indicator));
    }

    #[test]
    fn test_is_dora_from_indicator_nine() {
        let indicator = Tile::from_string("9m");
        // indicated dora is 1-man, 9 wraps to 1
        assert!(Tile::from_string("1m").is_dora_from_indicator(&indicator));
        // other man tiles are not dora
        assert!(!Tile::from_string("0m").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("9m").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("8m").is_dora_from_indicator(&indicator));
        // 1-pin, 1-sou, east wind (1z) are not dora
        assert!(!Tile::from_string("1p").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("1s").is_dora_from_indicator(&indicator));
        assert!(!Tile::from_string("1z").is_dora_from_indicator(&indicator));
    }

    #[test]
    fn test_is_dora_from_indicator_winds() {
        let indicator = Tile::from_string("1z");
        // indicated dora is south wind (2z)
        assert!(Tile::from_string("2z").is_dora_from_indicator(&indicator));

        let indicator = Tile::from_string("2z");
        // indicated dora is west wind (3z)
        assert!(Tile::from_string("3z").is_dora_from_indicator(&indicator));

        let indicator = Tile::from_string("3z");
        // indicated dora is north wind (4z)
        assert!(Tile::from_string("4z").is_dora_from_indicator(&indicator));

        let indicator = Tile::from_string("4z");
        // indicated dora is east wind (1z), north wraps to east
        assert!(Tile::from_string("1z").is_dora_from_indicator(&indicator));
        // white dragon (5z) is not dora
        assert!(!Tile::from_string("5z").is_dora_from_indicator(&indicator));
    }

    #[test]
    fn test_is_dora_from_indicator_dragons() {
        let indicator = Tile::from_string("5z");
        // indicated dora is green dragon (6z)
        assert!(Tile::from_string("6z").is_dora_from_indicator(&indicator));

        let indicator = Tile::from_string("6z");
        // indicated dora is red dragon (7z)
        assert!(Tile::from_string("7z").is_dora_from_indicator(&indicator));

        let indicator = Tile::from_string("7z");
        // indicated dora is white dragon (5z), red wraps to white
        assert!(Tile::from_string("5z").is_dora_from_indicator(&indicator));
        // east wind (1z) is not dora
        assert!(!Tile::from_string("1z").is_dora_from_indicator(&indicator));
    }
}
