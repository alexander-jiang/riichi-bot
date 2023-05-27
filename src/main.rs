// using the MPSZ notation, described here: https://ctan.math.utah.edu/ctan/tex-archive/graphics/mahjong/mahjong-code.pdf
pub const TILE_SUITS_CHARS: [char; 4] = ['m', 'p', 's', 'z'];
// number of tiles in a standard riichi mahjong set
pub const NUM_TILES: u32 = 3 * 4 * 9 + 4 * (4 + 3);

#[derive(Debug)]
pub struct Tile {
    serial: u32,
}

impl Tile {
    /// The suit of the tile in MSPZ notation, a single character `'m'` (man), `'p'` (pin), `'s'` (sou), or `'z'` (honors).
    /// e.g. for 3-sou: 's', for green dragon: 'z'
    pub fn suit(&self) -> char {
        if self.serial < (4 * 9) {
            'm'
        } else if self.serial >= (4 * 9) && self.serial < (2 * 4 * 9) {
            'p'
        } else if self.serial >= (2 * 4 * 9) && self.serial < (3 * 4 * 9) {
            's'
        } else if self.serial >= (3 * 4 * 9) && self.serial < (3 * 4 * 9 + 4 * 4 + 3 * 4) {
            'z'
        } else {
            panic!("Invalid tile serial number! {}", self.serial);
        }
    }

    /// The rank (aka value) of the tile in MSPZ notation, a single character `'0'` to `'9'`
    /// Red fives are 0, honor tiles are numbered: East is 1, then South, West, North, White, Green, and Red is 7
    /// e.g. for 3-sou: '3', for green dragon: '6', for south wind: '2', for red five (in any suit): '0'
    pub fn rank(&self) -> char {
        if self.is_number_suit() {
            // numbered suits
            let rank = self.serial % 9;
            let rank_char = char::from_digit(rank + 1, 10).expect("Invalid rank char for numbered tile");

            if rank_char == '5' {
                let copy = (self.serial % 36) / 9;
                // one "red five" tile from each numbered suit
                // in serial number ordering, the red-five is in the first set of 1-9 per suit
                if copy < 1 { '0' } else { '5' }
            } else {
                rank_char
            }
        } else if self.is_honor() {
            let rank = (self.serial - (3 * 36)) % 7;
            char::from_digit(rank + 1, 10).expect("Invalid rank char for honor tile")
        } else {
            panic!("Invalid tile serial number! {}", self.serial);
        }
    }

    /// A human-readable suit (not MSPZ notation), a single character.
    /// Same as MSPZ for numbered suits. Uses `'w'` for winds, and `'d'` for dragons.
    pub fn human_suit(&self) -> char {
        if self.is_number_suit() {
            self.suit()
        } else if self.is_honor() {
            match self.rank() {
                '1' => 'w',
                '2' => 'w',
                '3' => 'w',
                '4' => 'w',
                '5' => 'd',
                '6' => 'd',
                '7' => 'd',
                _   => panic!("invalid rank char {} for honor tile, serial={}", self.rank(), self.serial),
            }
        } else {
            panic!("Invalid tile serial number! {}", self.serial);
        }
    }

    /// A human-readable rank (not MSPZ notation), a single character.
    /// For numbered suits: `'1'` to `'9'`, and `'0'` for red five. Honor tiles are their first letter in english (winds: E, S, W, N; dragons: W, G, R)
    pub fn human_rank(&self) -> char {
        if self.is_number_suit() {
            self.rank()
        } else if self.is_honor() {
            match self.rank() {
                '1' => 'E',
                '2' => 'S',
                '3' => 'W',
                '4' => 'N',
                '5' => 'W',
                '6' => 'G',
                '7' => 'R',
                _   => panic!("invalid rank char {} for honor tile, serial={}", self.rank(), self.serial),
            }
        } else {
            panic!("Invalid tile serial number! {}", self.serial);
        }
    }

    // TODO implement this as Display trait instead?
    /// Represents the tile using MSPZ notation as a two character string: the rank followed by the suit
    /// e.g. for 8-man: "8m" , for red-5-sou, "0s", for north wind: "4z", for red dragon: "7z"
    pub fn to_string(&self) -> String {
        let mut tile_string = String::new();
        tile_string.push(self.rank());
        tile_string.push(self.suit());
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
    pub fn from_suit_and_rank(suit: char, rank: char, copy: u32) -> Self {
        // compute serial number based on suit and rank
        // assert suit is valid ([mpsz])
        // assert rank is valid ([0-9])
        // assert copy is valid (0-3)
        if suit == 'm' || suit == 'p' || suit == 's' {
            let suit_serial_start = match suit {
                'm' => 0 * (4 * 9),
                'p' => 1 * (4 * 9),
                's' => 2 * (4 * 9),
                _   => panic!("Invalid numbered suit {}", suit),
            };
            let rank_digit = rank.to_digit(10).expect("Invalid numbered tile rank char (valid ranks are 0-9)");
            if rank_digit == 0 {
                // red fives take the place of the first copy of a five in the serial number ordering
                assert!(copy == 0, "One copy of red fives only");
                Self { serial: suit_serial_start + 4 }
            } else if rank_digit == 5 {
                // the first copy of non-red-fives are alongside the second copy of non-five numbered tiles (by serial number)
                assert!(copy < 3, "Only three copies of non-red fives");
                Self { serial: suit_serial_start + 4 + (copy + 1) * 9 }
            } else {
                assert!(copy < 4, "Only four copies of numbered tiles (except fives)");
                Self { serial: suit_serial_start + (rank_digit - 1) + copy * 9}
            }
        } else if suit == 'z' {
            assert!(copy < 4, "Only four copies of honor tiles");
            let rank_digit = rank.to_digit(8).expect("Invalid honor tile rank char (valid ranks are 1-7)");
            assert!(rank_digit >= 1 && rank_digit <= 7, "Invalid honor tile rank char (valid ranks are 1-7)");
            Self { serial: 3 * (4 * 9) + (rank_digit - 1) + copy * 7 }
        } else {
            panic!("Invalid tile suit char {}", suit);
        }
    }

    /// Constructs a Tile from the same 2-character representation used by `to_string()`
    /// e.g. "7m" -> 7-man
    pub fn from_string(tile_string: &str) -> Self {
        // parse into suit and rank
        // assert tile_string length is 2
        let mut tile_chars = tile_string.chars();
        let rank = tile_chars.next();
        let suit = tile_chars.next();

        if rank.is_some() && suit.is_some() {
            // TODO why should the copy always be 0?
            Self::from_suit_and_rank(suit.expect("Must have tile suit"), rank.expect("Must have tile rank"), 0)
        } else {
            panic!("invalid tile string {}", tile_string);
        }
    }

    // helper functions

    /// If the tile is in a numbered suit (man, pin, or sou)
    pub fn is_number_suit(&self) -> bool {
        self.suit() == 'm' || self.suit() == 'p' || self.suit() == 's'
    }

    /// If the tile is rank 1 or 9 in a numbered suit
    pub fn is_terminal(&self) -> bool {
        // example yaku:
        // - chanta (at least 1 terminal or honor tile in each meld and in the pair)
        // - junchan (at least 1 terminal tile in each meld and in the pair)
        // - chinroutou (hand is entirely terminal tiles)
        (self.rank() == '1' || self.rank() == '9') && self.is_number_suit()
    }

    /// If the tile is a wind tile or a dragon tile (honor tiles are also known as word tiles)
    pub fn is_honor(&self) -> bool {
        // example yaku:
        // - honroutou (hand is entirely terminal or honor tiles)
        // - tsuuiisou (hand is entirely honor tiles)
        self.suit() == 'z'
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
        (self.suit() == 'z' && self.rank() == '6') || (self.suit() == 's' && (self.rank() == '2' || self.rank() == '3' || self.rank() == '4' || self.rank() == '6' || self.rank() == '8'))
    }

    /// If the tile is a red five tile
    pub fn is_red_five(&self) -> bool {
        // used for counting dora
        self.is_number_suit() && self.rank() == '0'
    }
}

fn main() {
    // let tile = Tile { suit: TileSuit::Man, rank: NumberTileType::Five};
    // println!("It's a tile: {}", tile.repr());

    let mut hand: Vec<String> = Vec::new();
    hand.push(String::from("1s"));
    hand.push(String::from("2s"));
    hand.push(String::from("3s"));
    hand.push(String::from("4s"));
    hand.push(String::from("5s"));
    hand.push(String::from("6s"));
    hand.push(String::from("7s"));
    hand.push(String::from("8s"));
    hand.push(String::from("9s"));
    hand.push(String::from("Wd"));
    hand.push(String::from("Wd"));
    hand.push(String::from("Wd"));
    hand.push(String::from("Nw"));
    hand.push(String::from("Nw"));
    println!("hand:");
    for tile in &hand {
        println!("{tile}");
    }
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;

    use std::collections::HashMap;

    #[test]
    fn test_tile_from_and_to_suit_rank() {
        // 1-man, first copy, serial number 0
        let man_tile = Tile::from_suit_and_rank('m', '1', 0);
        assert_eq!(man_tile.suit(), 'm');
        assert_eq!(man_tile.rank(), '1');
        assert_eq!(man_tile.serial, 0 + 0 + 0 * 9);

        // red-5-man, serial number 4
        let man_red_five = Tile::from_suit_and_rank('m', '0', 0);
        assert_eq!(man_red_five.suit(), 'm');
        assert_eq!(man_red_five.rank(), '0');
        assert_eq!(man_red_five.serial, 0 + 4 + 0 * 9);

        // 5-man, first copy, serial number 13 (since the red-5-man is serial number 4)
        let man_red_five = Tile::from_suit_and_rank('m', '5', 0);
        assert_eq!(man_red_five.suit(), 'm');
        assert_eq!(man_red_five.rank(), '5');
        assert_eq!(man_red_five.serial, 0 + 4 + 1 * 9);

        // 5-man, third copy, serial number 31 (since the red-5-man is serial number 4)
        let man_red_five = Tile::from_suit_and_rank('m', '5', 2);
        assert_eq!(man_red_five.suit(), 'm');
        assert_eq!(man_red_five.rank(), '5');
        assert_eq!(man_red_five.serial, 0 + 4 + 3 * 9);

        // 4-pin, third copy, serial number 57
        let pin_tile = Tile::from_suit_and_rank('p', '4', 2);
        assert_eq!(pin_tile.suit(), 'p');
        assert_eq!(pin_tile.rank(), '4');
        assert_eq!(pin_tile.serial, (4 * 9) + 3 + 2 * 9);

        // 9-sou, fourth copy, serial number 107
        let sou_tile = Tile::from_suit_and_rank('s', '9', 3);
        assert_eq!(sou_tile.suit(), 's');
        assert_eq!(sou_tile.rank(), '9');
        assert_eq!(sou_tile.serial, 2 * (4 * 9) + 8 + 3 * 9);

        // west wind, third copy, serial number 124
        let wind_tile = Tile::from_suit_and_rank('z', '3', 2);
        assert_eq!(wind_tile.suit(), 'z');
        assert_eq!(wind_tile.rank(), '3');
        assert_eq!(wind_tile.serial, 3 * (4 * 9) + 2 + 2 * 7);

        // red dragon, first copy, serial number 114
        let dragon_tile = Tile::from_suit_and_rank('z', '7', 0);
        assert_eq!(dragon_tile.suit(), 'z');
        assert_eq!(dragon_tile.rank(), '7');
        assert_eq!(dragon_tile.serial, 3 * (4 * 9) + 6 + 0 * 7);
    }

    #[test]
    fn test_tile_from_and_to_string() {
        let man_tile = Tile::from_string("1m");
        assert_eq!(man_tile.suit(), 'm');
        assert_eq!(man_tile.rank(), '1');
        assert_eq!(man_tile.to_string(), "1m".to_string());

        let pin_tile = Tile::from_string("4p");
        assert_eq!(pin_tile.suit(), 'p');
        assert_eq!(pin_tile.rank(), '4');
        assert_eq!(pin_tile.to_string(), "4p".to_string());

        let sou_tile = Tile::from_string("9s");
        assert_eq!(sou_tile.suit(), 's');
        assert_eq!(sou_tile.rank(), '9');
        assert_eq!(sou_tile.to_string(), "9s".to_string());

        let east_wind_tile = Tile::from_string("1z");
        assert_eq!(east_wind_tile.suit(), 'z');
        assert_eq!(east_wind_tile.rank(), '1');
        assert_eq!(east_wind_tile.to_string(), "1z".to_string());

        let south_wind_tile = Tile::from_string("2z");
        assert_eq!(south_wind_tile.suit(), 'z');
        assert_eq!(south_wind_tile.rank(), '2');
        assert_eq!(south_wind_tile.to_string(), "2z".to_string());

        let west_wind_tile = Tile::from_string("3z");
        assert_eq!(west_wind_tile.suit(), 'z');
        assert_eq!(west_wind_tile.rank(), '3');
        assert_eq!(west_wind_tile.to_string(), "3z".to_string());

        let north_wind_tile = Tile::from_string("4z");
        assert_eq!(north_wind_tile.suit(), 'z');
        assert_eq!(north_wind_tile.rank(), '4');
        assert_eq!(north_wind_tile.to_string(), "4z".to_string());

        let white_dragon_tile = Tile::from_string("5z");
        assert_eq!(white_dragon_tile.suit(), 'z');
        assert_eq!(white_dragon_tile.rank(), '5');
        assert_eq!(white_dragon_tile.to_string(), "5z".to_string());

        let green_dragon_tile = Tile::from_string("6z");
        assert_eq!(green_dragon_tile.suit(), 'z');
        assert_eq!(green_dragon_tile.rank(), '6');
        assert_eq!(green_dragon_tile.to_string(), "6z".to_string());

        let red_dragon_tile = Tile::from_string("7z");
        assert_eq!(red_dragon_tile.suit(), 'z');
        assert_eq!(red_dragon_tile.rank(), '7');
        assert_eq!(red_dragon_tile.to_string(), "7z".to_string());
    }


    #[test]
    fn test_tile_human_and_mspz_notation() {
        let man_tile = Tile::from_string("1m");
        assert_eq!(man_tile.human_suit(), 'm');
        assert_eq!(man_tile.human_rank(), '1');
        assert_eq!(man_tile.to_human_string(), "1m".to_string());

        let pin_tile = Tile::from_string("4p");
        assert_eq!(pin_tile.human_suit(), 'p');
        assert_eq!(pin_tile.human_rank(), '4');
        assert_eq!(pin_tile.to_human_string(), "4p".to_string());

        let sou_tile = Tile::from_string("9s");
        assert_eq!(sou_tile.human_suit(), 's');
        assert_eq!(sou_tile.human_rank(), '9');
        assert_eq!(sou_tile.to_human_string(), "9s".to_string());

        let east_wind_tile = Tile::from_string("1z");
        assert_eq!(east_wind_tile.human_suit(), 'w');
        assert_eq!(east_wind_tile.human_rank(), 'E');
        assert_eq!(east_wind_tile.to_human_string(), "Ew".to_string());

        let south_wind_tile = Tile::from_string("2z");
        assert_eq!(south_wind_tile.human_suit(), 'w');
        assert_eq!(south_wind_tile.human_rank(), 'S');
        assert_eq!(south_wind_tile.to_human_string(), "Sw".to_string());

        let west_wind_tile = Tile::from_string("3z");
        assert_eq!(west_wind_tile.human_suit(), 'w');
        assert_eq!(west_wind_tile.human_rank(), 'W');
        assert_eq!(west_wind_tile.to_human_string(), "Ww".to_string());

        let north_wind_tile = Tile::from_string("4z");
        assert_eq!(north_wind_tile.human_suit(), 'w');
        assert_eq!(north_wind_tile.human_rank(), 'N');
        assert_eq!(north_wind_tile.to_human_string(), "Nw".to_string());

        let white_dragon_tile = Tile::from_string("5z");
        assert_eq!(white_dragon_tile.human_suit(), 'd');
        assert_eq!(white_dragon_tile.human_rank(), 'W');
        assert_eq!(white_dragon_tile.to_human_string(), "Wd".to_string());

        let green_dragon_tile = Tile::from_string("6z");
        assert_eq!(green_dragon_tile.human_suit(), 'd');
        assert_eq!(green_dragon_tile.human_rank(), 'G');
        assert_eq!(green_dragon_tile.to_human_string(), "Gd".to_string());

        let red_dragon_tile = Tile::from_string("7z");
        assert_eq!(red_dragon_tile.human_suit(), 'd');
        assert_eq!(red_dragon_tile.human_rank(), 'R');
        assert_eq!(red_dragon_tile.to_human_string(), "Rd".to_string());
    }

    // verify the count of tiles of each suit if you iterate through all serial numbers
    #[test]
    fn test_tile_suit_counts() {
        let mut suit_counts = HashMap::new();
        for serial in 0..NUM_TILES {
            let tile = Tile {serial};
            let count = suit_counts.entry(tile.suit()).or_insert(0);
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
            let tile = Tile {serial};
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
            let tile = Tile {serial};
            if tile.is_honor() {
                num_honor_tiles += 1;
            }
        }
        let num_honor_tiles = num_honor_tiles;

        assert_eq!(num_honor_tiles, expected_num_honor_tiles);
    }

    #[test]
    fn test_simple_tile_counts() {
        // 3 suits * 7 ranks per suit (2 through 8, inclusive) * 4 copies
        let expected_num_simple_tiles: u32 = 3 * 7 * 4;

        let mut num_simple_tiles: u32 = 0;
        for serial in 0..NUM_TILES {
            let tile = Tile {serial};
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
            let tile = Tile {serial};
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
            let tile = Tile {serial};
            if tile.is_red_five() {
                num_red_fives += 1;
            }
        }
        let num_red_fives = num_red_fives;

        assert_eq!(num_red_fives, expected_num_red_fives);
    }
}
