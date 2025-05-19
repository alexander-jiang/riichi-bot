use crate::mahjong_error;
pub use crate::mahjong_tile;

// pub enum TileSource {
//     Draw,
//     DeadWallDraw,
//     FromOpponentDiscard,
// }

// pub struct AdditionalTileInfo {
//     tile: mahjong_tile::MahjongTile,
//     tile_source: TileSource,
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeldType {
    ClosedTriplet,  // ankou
    OpenTriplet,    // minkou
    ClosedQuad,     // ankan (all four tiles are self-drawn)
    OpenQuad, // daiminkan (three of four tiles are already self-drawn, the fourth is called from opponent discard)
    AddedQuad, // shouminkan (formed by first having a open triplet/minkou, then self-drawing the fourth copy of tile)
    ClosedSequence, // minjun
    OpenSequence, // anjun
    Pair,      // jantou, also called the "eye"
    Isolated,
}

impl MeldType {
    pub fn is_open(&self) -> bool {
        match self {
            Self::OpenTriplet | Self::OpenQuad | Self::AddedQuad | Self::OpenSequence => true,
            Self::ClosedTriplet
            | Self::ClosedQuad
            | Self::ClosedSequence
            | Self::Pair
            | Self::Isolated => false,
        }
    }
}

#[derive(Clone)]
pub struct MahjongMeld {
    pub meld_type: MeldType,
    pub tile_ids: [Option<u8>; 4], // None value represents the tiles that the meld doesn't use
    pub added_tile_id: Option<u8>, // set to None for closed melds
    pub added_tile_player: Option<u8>, // set to None for closed melds
}

pub fn is_valid_meld(tile_ids: &[u8], meld_type: MeldType) -> bool {
    let num_tile_ids = tile_ids.len();

    // validate the tile ids for the meld type
    match meld_type {
        MeldType::ClosedTriplet | MeldType::OpenTriplet => {
            if num_tile_ids != 3 {
                return false;
            }
            // all tiles must be the same
            if tile_ids[0] != tile_ids[1] || tile_ids[1] != tile_ids[2] {
                return false;
            }
            return true;
        }
        MeldType::ClosedQuad | MeldType::OpenQuad | MeldType::AddedQuad => {
            if num_tile_ids != 4 {
                return false;
            }
            // all tiles must be the same
            if tile_ids[0] != tile_ids[1]
                || tile_ids[1] != tile_ids[2]
                || tile_ids[2] != tile_ids[3]
            {
                return false;
            }
            return true;
        }
        MeldType::Pair => {
            if num_tile_ids != 2 {
                return false;
            }
            // all tiles must be the same
            if tile_ids[0] != tile_ids[1] {
                return false;
            }
            return true;
        }
        MeldType::ClosedSequence | MeldType::OpenSequence => {
            if num_tile_ids != 3 {
                return false;
            }

            let first_tile_suit = mahjong_tile::get_num_tile_suit(tile_ids[0]);
            let second_tile_suit = mahjong_tile::get_num_tile_suit(tile_ids[1]);
            let third_tile_suit = mahjong_tile::get_num_tile_suit(tile_ids[2]);
            if first_tile_suit.is_none() || second_tile_suit.is_none() || third_tile_suit.is_none()
            {
                // only numbered tiles can form sequences
                return false;
            }
            if first_tile_suit.unwrap() != second_tile_suit.unwrap()
                || second_tile_suit.unwrap() != third_tile_suit.unwrap()
            {
                // tiles must be in same suit
                return false;
            }

            let first_tile_rank = mahjong_tile::get_num_tile_rank(tile_ids[0]);
            let second_tile_rank = mahjong_tile::get_num_tile_rank(tile_ids[1]);
            let third_tile_rank = mahjong_tile::get_num_tile_rank(tile_ids[2]);
            if first_tile_rank.is_none() || second_tile_rank.is_none() || third_tile_rank.is_none()
            {
                // only numbered tiles can form sequences
                return false;
            }
            let first_tile_rank = first_tile_rank.unwrap();
            let second_tile_rank = second_tile_rank.unwrap();
            let third_tile_rank = third_tile_rank.unwrap();
            if first_tile_rank == second_tile_rank
                || second_tile_rank == third_tile_rank
                || first_tile_rank == third_tile_rank
            {
                // sequence ranks must be distinct
                return false;
            }
            let max_rank = std::cmp::max(
                std::cmp::max(first_tile_rank, second_tile_rank),
                third_tile_rank,
            );
            let min_rank = std::cmp::min(
                std::cmp::min(first_tile_rank, second_tile_rank),
                third_tile_rank,
            );
            if max_rank - min_rank != 2 {
                // max and min rank must be exactly 2 apart
                return false;
            }
            // if there are 3 tiles, all 3 tile ranks are distinct, and
            // max rank and min rank are exactly 2 apart, that means it must be a sequence
            true
        }
        MeldType::Isolated => {
            if num_tile_ids != 1 {
                return false;
            }
            true
        }
    }
}

impl MahjongMeld {
    pub fn from_tile_ids(
        tile_ids: &[u8],
        meld_type: MeldType,
    ) -> Result<Self, mahjong_error::MahjongError> {
        let num_tile_ids = tile_ids.len();
        if !is_valid_meld(tile_ids, meld_type) {
            return Err(mahjong_error::MahjongError::new(
                "Wrong number of tile_ids for MahjongMeld type",
            ));
        }

        let mut meld_tile_ids: [Option<u8>; 4] = [None; 4];
        let mut meld_id = 0;
        while meld_id < num_tile_ids {
            meld_tile_ids[meld_id] = Some(tile_ids[meld_id]);
            meld_id += 1;
        }

        // TODO set the added_tile_info when ready (based on if meld is open)
        Ok(Self {
            meld_type: meld_type,
            tile_ids: meld_tile_ids,
            added_tile_id: None,
            added_tile_player: None,
        })
    }

    pub fn num_tiles(&self) -> u8 {
        let mut count = 0;
        for tile_id_option in self.tile_ids {
            if tile_id_option.is_some() {
                count += 1;
            }
        }
        // TODO should we validate that this number of tiles matches the expected number of tiles based on meld type?
        count
    }

    pub fn is_open(&self) -> bool {
        self.meld_type.is_open()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_is_valid_meld_sequence() {
        // honor tiles cannot form sequences
        let wind_tile_ids = [
            mahjong_tile::get_id_from_tile_text("1z").unwrap(),
            mahjong_tile::get_id_from_tile_text("2z").unwrap(),
            mahjong_tile::get_id_from_tile_text("3z").unwrap(),
        ];
        assert!(!is_valid_meld(&wind_tile_ids, MeldType::ClosedSequence));

        // honor tiles cannot form sequences
        let dragon_tile_ids = [
            mahjong_tile::get_id_from_tile_text("5z").unwrap(),
            mahjong_tile::get_id_from_tile_text("6z").unwrap(),
            mahjong_tile::get_id_from_tile_text("7z").unwrap(),
        ];
        assert!(!is_valid_meld(&dragon_tile_ids, MeldType::ClosedSequence));

        // must be in same suit
        let different_suit = [
            mahjong_tile::get_id_from_tile_text("3m").unwrap(),
            mahjong_tile::get_id_from_tile_text("4m").unwrap(),
            mahjong_tile::get_id_from_tile_text("5s").unwrap(),
        ];
        assert!(!is_valid_meld(&different_suit, MeldType::ClosedSequence));

        // must be consecutive
        let nonconsecutive = [
            mahjong_tile::get_id_from_tile_text("1m").unwrap(),
            mahjong_tile::get_id_from_tile_text("2m").unwrap(),
            mahjong_tile::get_id_from_tile_text("4m").unwrap(),
        ];
        assert!(!is_valid_meld(&nonconsecutive, MeldType::ClosedSequence));

        let duplicate_rank = [
            mahjong_tile::get_id_from_tile_text("4p").unwrap(),
            mahjong_tile::get_id_from_tile_text("3p").unwrap(),
            mahjong_tile::get_id_from_tile_text("4p").unwrap(),
        ];
        assert!(!is_valid_meld(&duplicate_rank, MeldType::ClosedSequence));

        // sequences cannot wrap around
        let wrap_around = [
            mahjong_tile::get_id_from_tile_text("1m").unwrap(),
            mahjong_tile::get_id_from_tile_text("8m").unwrap(),
            mahjong_tile::get_id_from_tile_text("9m").unwrap(),
        ];
        assert!(!is_valid_meld(&wrap_around, MeldType::ClosedSequence));

        // too many tiles
        let too_many_tiles = [
            mahjong_tile::get_id_from_tile_text("1p").unwrap(),
            mahjong_tile::get_id_from_tile_text("1p").unwrap(),
            mahjong_tile::get_id_from_tile_text("2p").unwrap(),
            mahjong_tile::get_id_from_tile_text("3p").unwrap(),
        ];
        assert!(!is_valid_meld(&too_many_tiles, MeldType::ClosedSequence));

        // not enough tiles
        let not_enough_tiles = [
            mahjong_tile::get_id_from_tile_text("7s").unwrap(),
            mahjong_tile::get_id_from_tile_text("8s").unwrap(),
        ];
        assert!(!is_valid_meld(&not_enough_tiles, MeldType::ClosedSequence));

        // tile ids can be given in any order
        let tile_ids = [
            mahjong_tile::get_id_from_tile_text("2p").unwrap(),
            mahjong_tile::get_id_from_tile_text("4p").unwrap(),
            mahjong_tile::get_id_from_tile_text("3p").unwrap(),
        ];
        assert!(is_valid_meld(&tile_ids, MeldType::ClosedSequence));

        // open sequence is okay too
        let tile_ids = [
            mahjong_tile::get_id_from_tile_text("9s").unwrap(),
            mahjong_tile::get_id_from_tile_text("8s").unwrap(),
            mahjong_tile::get_id_from_tile_text("7s").unwrap(),
        ];
        assert!(is_valid_meld(&tile_ids, MeldType::OpenSequence));
    }

    #[test]
    fn check_is_valid_meld_triplet() {
        // honor tiles can form triplets
        let wind_triplet = [
            mahjong_tile::get_id_from_tile_text("2z").unwrap(),
            mahjong_tile::get_id_from_tile_text("2z").unwrap(),
            mahjong_tile::get_id_from_tile_text("2z").unwrap(),
        ];
        assert!(is_valid_meld(&wind_triplet, MeldType::ClosedTriplet));

        // honor tiles can form triplets
        let dragon_triplet = [
            mahjong_tile::get_id_from_tile_text("5z").unwrap(),
            mahjong_tile::get_id_from_tile_text("5z").unwrap(),
            mahjong_tile::get_id_from_tile_text("5z").unwrap(),
        ];
        assert!(is_valid_meld(&dragon_triplet, MeldType::ClosedTriplet));

        // must be in same suit
        let different_suit = [
            mahjong_tile::get_id_from_tile_text("3m").unwrap(),
            mahjong_tile::get_id_from_tile_text("3m").unwrap(),
            mahjong_tile::get_id_from_tile_text("3s").unwrap(),
        ];
        assert!(!is_valid_meld(&different_suit, MeldType::ClosedTriplet));

        // must be all identical
        let not_identical = [
            mahjong_tile::get_id_from_tile_text("1m").unwrap(),
            mahjong_tile::get_id_from_tile_text("1m").unwrap(),
            mahjong_tile::get_id_from_tile_text("2m").unwrap(),
        ];
        assert!(!is_valid_meld(&not_identical, MeldType::ClosedTriplet));

        // too many tiles
        let too_many_tiles = [
            mahjong_tile::get_id_from_tile_text("1p").unwrap(),
            mahjong_tile::get_id_from_tile_text("1p").unwrap(),
            mahjong_tile::get_id_from_tile_text("1p").unwrap(),
            mahjong_tile::get_id_from_tile_text("1p").unwrap(),
        ];
        assert!(!is_valid_meld(&too_many_tiles, MeldType::ClosedTriplet));

        // not enough tiles
        let not_enough_tiles = [
            mahjong_tile::get_id_from_tile_text("7s").unwrap(),
            mahjong_tile::get_id_from_tile_text("7s").unwrap(),
        ];
        assert!(!is_valid_meld(&not_enough_tiles, MeldType::ClosedTriplet));

        let tile_ids = [
            mahjong_tile::get_id_from_tile_text("4p").unwrap(),
            mahjong_tile::get_id_from_tile_text("4p").unwrap(),
            mahjong_tile::get_id_from_tile_text("4p").unwrap(),
        ];
        assert!(is_valid_meld(&tile_ids, MeldType::ClosedTriplet));

        // open triplet is okay too
        let tile_ids = [
            mahjong_tile::get_id_from_tile_text("3s").unwrap(),
            mahjong_tile::get_id_from_tile_text("3s").unwrap(),
            mahjong_tile::get_id_from_tile_text("3s").unwrap(),
        ];
        assert!(is_valid_meld(&tile_ids, MeldType::OpenTriplet));
    }

    #[test]
    fn check_is_valid_meld_quad() {
        // honor tiles can form quads
        let wind_quad = [
            mahjong_tile::get_id_from_tile_text("4z").unwrap(),
            mahjong_tile::get_id_from_tile_text("4z").unwrap(),
            mahjong_tile::get_id_from_tile_text("4z").unwrap(),
            mahjong_tile::get_id_from_tile_text("4z").unwrap(),
        ];
        assert!(is_valid_meld(&wind_quad, MeldType::ClosedQuad));

        // honor tiles can form quads
        let dragon_quad = [
            mahjong_tile::get_id_from_tile_text("7z").unwrap(),
            mahjong_tile::get_id_from_tile_text("7z").unwrap(),
            mahjong_tile::get_id_from_tile_text("7z").unwrap(),
            mahjong_tile::get_id_from_tile_text("7z").unwrap(),
        ];
        assert!(is_valid_meld(&dragon_quad, MeldType::ClosedQuad));

        // must be in same suit
        let different_suit = [
            mahjong_tile::get_id_from_tile_text("3m").unwrap(),
            mahjong_tile::get_id_from_tile_text("3m").unwrap(),
            mahjong_tile::get_id_from_tile_text("3m").unwrap(),
            mahjong_tile::get_id_from_tile_text("3s").unwrap(),
        ];
        assert!(!is_valid_meld(&different_suit, MeldType::ClosedQuad));

        // must be all identical
        let not_identical = [
            mahjong_tile::get_id_from_tile_text("6s").unwrap(),
            mahjong_tile::get_id_from_tile_text("6s").unwrap(),
            mahjong_tile::get_id_from_tile_text("6s").unwrap(),
            mahjong_tile::get_id_from_tile_text("2s").unwrap(),
        ];
        assert!(!is_valid_meld(&not_identical, MeldType::ClosedQuad));

        // too many tiles
        let too_many_tiles = [
            mahjong_tile::get_id_from_tile_text("9m").unwrap(),
            mahjong_tile::get_id_from_tile_text("9m").unwrap(),
            mahjong_tile::get_id_from_tile_text("9m").unwrap(),
            mahjong_tile::get_id_from_tile_text("9m").unwrap(),
            mahjong_tile::get_id_from_tile_text("9m").unwrap(),
        ];
        assert!(!is_valid_meld(&too_many_tiles, MeldType::ClosedQuad));

        // not enough tiles
        let not_enough_tiles = [
            mahjong_tile::get_id_from_tile_text("4s").unwrap(),
            mahjong_tile::get_id_from_tile_text("4s").unwrap(),
            mahjong_tile::get_id_from_tile_text("4s").unwrap(),
        ];
        assert!(!is_valid_meld(&not_enough_tiles, MeldType::ClosedQuad));

        let tile_ids = [
            mahjong_tile::get_id_from_tile_text("2p").unwrap(),
            mahjong_tile::get_id_from_tile_text("2p").unwrap(),
            mahjong_tile::get_id_from_tile_text("2p").unwrap(),
            mahjong_tile::get_id_from_tile_text("2p").unwrap(),
        ];
        assert!(is_valid_meld(&tile_ids, MeldType::ClosedQuad));

        // open quad is okay too
        let tile_ids = [
            mahjong_tile::get_id_from_tile_text("3s").unwrap(),
            mahjong_tile::get_id_from_tile_text("3s").unwrap(),
            mahjong_tile::get_id_from_tile_text("3s").unwrap(),
            mahjong_tile::get_id_from_tile_text("3s").unwrap(),
        ];
        assert!(is_valid_meld(&tile_ids, MeldType::OpenQuad));

        // added quad is okay too
        let tile_ids = [
            mahjong_tile::get_id_from_tile_text("8m").unwrap(),
            mahjong_tile::get_id_from_tile_text("8m").unwrap(),
            mahjong_tile::get_id_from_tile_text("8m").unwrap(),
            mahjong_tile::get_id_from_tile_text("8m").unwrap(),
        ];
        assert!(is_valid_meld(&tile_ids, MeldType::AddedQuad));
    }

    #[test]
    fn check_is_valid_meld_pair() {
        // honor tiles can form pairs
        let wind_pair = [
            mahjong_tile::get_id_from_tile_text("3z").unwrap(),
            mahjong_tile::get_id_from_tile_text("3z").unwrap(),
        ];
        assert!(is_valid_meld(&wind_pair, MeldType::Pair));

        // honor tiles can form pairs
        let dragon_pair = [
            mahjong_tile::get_id_from_tile_text("6z").unwrap(),
            mahjong_tile::get_id_from_tile_text("6z").unwrap(),
        ];
        assert!(is_valid_meld(&dragon_pair, MeldType::Pair));

        // must be in same suit
        let different_suit = [
            mahjong_tile::get_id_from_tile_text("3m").unwrap(),
            mahjong_tile::get_id_from_tile_text("3s").unwrap(),
        ];
        assert!(!is_valid_meld(&different_suit, MeldType::Pair));

        // must be identical
        let not_identical = [
            mahjong_tile::get_id_from_tile_text("6s").unwrap(),
            mahjong_tile::get_id_from_tile_text("2s").unwrap(),
        ];
        assert!(!is_valid_meld(&not_identical, MeldType::Pair));

        // too many tiles
        let too_many_tiles = [
            mahjong_tile::get_id_from_tile_text("9m").unwrap(),
            mahjong_tile::get_id_from_tile_text("9m").unwrap(),
            mahjong_tile::get_id_from_tile_text("9m").unwrap(),
        ];
        assert!(!is_valid_meld(&too_many_tiles, MeldType::Pair));

        // not enough tiles
        let not_enough_tiles = [mahjong_tile::get_id_from_tile_text("4s").unwrap()];
        assert!(!is_valid_meld(&not_enough_tiles, MeldType::Pair));

        let tile_ids = [
            mahjong_tile::get_id_from_tile_text("2p").unwrap(),
            mahjong_tile::get_id_from_tile_text("2p").unwrap(),
        ];
        assert!(is_valid_meld(&tile_ids, MeldType::Pair));
    }

    #[test]
    fn meld_from_tile_ids() {
        let sequence_tile_ids = [
            mahjong_tile::get_id_from_tile_text("1m").unwrap(),
            mahjong_tile::get_id_from_tile_text("2m").unwrap(),
            mahjong_tile::get_id_from_tile_text("3m").unwrap(),
        ];
        match MahjongMeld::from_tile_ids(&sequence_tile_ids, MeldType::ClosedSequence) {
            Ok(_meld) => assert!(true),
            Err(_) => assert!(false),
        };

        let red_five_sequence_tile_ids = [
            mahjong_tile::get_id_from_tile_text("3p").unwrap(),
            mahjong_tile::get_id_from_tile_text("4p").unwrap(),
            mahjong_tile::get_id_from_tile_text("0p").unwrap(),
        ];
        match MahjongMeld::from_tile_ids(&red_five_sequence_tile_ids, MeldType::ClosedSequence) {
            Ok(_meld) => assert!(true),
            Err(_) => assert!(false),
        };

        let triplet_tile_ids = [
            mahjong_tile::get_id_from_tile_text("6s").unwrap(),
            mahjong_tile::get_id_from_tile_text("6s").unwrap(),
            mahjong_tile::get_id_from_tile_text("6s").unwrap(),
        ];
        match MahjongMeld::from_tile_ids(&triplet_tile_ids, MeldType::ClosedTriplet) {
            Ok(_meld) => assert!(true),
            Err(_) => assert!(false),
        };

        let quad_tile_ids = [
            mahjong_tile::get_id_from_tile_text("1z").unwrap(),
            mahjong_tile::get_id_from_tile_text("1z").unwrap(),
            mahjong_tile::get_id_from_tile_text("1z").unwrap(),
            mahjong_tile::get_id_from_tile_text("1z").unwrap(),
        ];
        match MahjongMeld::from_tile_ids(&quad_tile_ids, MeldType::ClosedQuad) {
            Ok(_meld) => assert!(true),
            Err(_) => assert!(false),
        };

        let pair_tile_ids = [
            mahjong_tile::get_id_from_tile_text("5z").unwrap(),
            mahjong_tile::get_id_from_tile_text("5z").unwrap(),
        ];
        match MahjongMeld::from_tile_ids(&pair_tile_ids, MeldType::Pair) {
            Ok(_meld) => assert!(true),
            Err(_) => assert!(false),
        };
    }
}
