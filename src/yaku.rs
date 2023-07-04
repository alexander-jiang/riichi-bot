use crate::{state, tile_grouping, tiles};

pub enum Yaku {
    // 1 han
    MenzenTsumo, // i.e. fully concealed hand, winning with a closed hand by self-draw
    Riichi,
    Ippatsu,
    Pinfu,
    Iipeikou, // i.e. pure double sequence, hand contains two identical sequences
    Haitei,   // i.e. winning on the last tile drawn from the wall
    Houtei,   // i.e. winning on the last discarded tile before exhaustive draw
    Rinshan,  // i.e. winning on a tile drawn from the dead wall (i.e. after calling a quad)
    Chankan,  // i.e. robbing a kan, winning on the tile added to another player's open triplet
    Tanyao,   // i.e. all simples, hand has no terminal or honor tiles
    Yakuhai,

    // 2 han
    DoubleRiichi,
    Chanta, // i.e. half outside hand, terminal or honor tile in each hand group (including the pair)
    SanshokuDoujun, // i.e. mixed triple sequence, hand includes three sequences with the same ranks across the three numbered suits
    Ittsu, // i.e. pure straight, hand includes the sequences 123, 456, and 789 in a single suit
    Toitoi, // i.e. all triplets, hand uses 4 triplets and a pair
    Sanankou, // i.e. three concealed/closed triplets
    SanshokuDoukou, // i.e. mixed triple triplet, hand includes three triplets with the same ranks across the three numbered suits
    Sankantsu,      // i.e. three quads
    Chiitoitsu,     // i.e. seven pairs
    Honroutou,      // i.e. all terminals and honors
    Shousangen, // i.e. small three dragons, two triplets of dragons, and a pair of the third dragon

    // 3 han
    Honitsu,    // i.e. half flush, hand only uses a single numbered suit and honor tiles
    Junchan,    // i.e. fully outside hand, terminal in each hand group (including the pair)
    Ryanpeikou, // i.e. two sets of two identical sequences

    // 6 han
    Chinitsu, // i.e. full flush, hand only uses a single numbered suit

    // yakuman (i.e. equivalent to 13+ han)
    KazoeYakuman,  // i.e. if a hand earns 13+ han based on other regular yaku and dora
    KokushiMusou,  // i.e. thirteen orphans
    Suuankou,      // i.e. four closed triplets
    Daisangen,     // i.e. big three dragons
    Shousuushii,   // i.e. small winds
    Daisuushii,    // i.e. big winds
    Tsuuiisou,     // i.e. all honors
    Chinroutou,    // i.e. all terminals
    Ryuuiisou,     // i.e. all green
    ChuurenPoutou, // i.e. nine gates
    Suukantsu,     // i.e. four quads
    Tenhou,        // i.e. heavenly hand, dealer wins with their initial draw
    Chiihou, // i.e. earthly hand, non-dealer wins with their initial draw (and no tile calls were made)

    // special case
    NagashiMangan, // at exhaustive draw, if a player only discarded terminals and honors, and none of their discards were called
}

impl Yaku {
    pub fn han_value(yaku: &Self) -> u32 {
        match yaku {
            Self::MenzenTsumo => 1,
            Self::Riichi => 1,
            Self::Ippatsu => 1,
            Self::Pinfu => 1,
            Self::Iipeikou => 1,
            Self::Haitei => 1,
            Self::Houtei => 1,
            Self::Rinshan => 1,
            Self::Chankan => 1,
            Self::Tanyao => 1,
            Self::Yakuhai => 1,
            Self::DoubleRiichi => 2,
            Self::Chanta => 2,
            Self::SanshokuDoujun => 2,
            Self::Ittsu => 2,
            Self::Toitoi => 2,
            Self::Sanankou => 2,
            Self::SanshokuDoukou => 2,
            Self::Sankantsu => 2,
            Self::Chiitoitsu => 2,
            Self::Honroutou => 2,
            Self::Shousangen => 2,
            Self::Honitsu => 3,
            Self::Junchan => 3,
            Self::Ryanpeikou => 3,
            Self::Chinitsu => 6,
            Self::KazoeYakuman => 13,
            Self::KokushiMusou => 13,
            Self::Suuankou => 13,
            Self::Daisangen => 13,
            Self::Shousuushii => 13,
            Self::Daisuushii => 13,
            Self::Tsuuiisou => 13,
            Self::Chinroutou => 13,
            Self::Ryuuiisou => 13,
            Self::ChuurenPoutou => 13,
            Self::Suukantsu => 13,
            Self::Tenhou => 13,
            Self::Chiihou => 13,
            Self::NagashiMangan => 5, // this yaku is not compatible with other yaku but is worth mangan tsumo, which can be reached at 5 han
        }
    }
}

pub fn is_yakuhai_tile(
    tile: &tiles::Tile,
    round_wind_rank: tiles::TileRank,
    seat_wind_rank: tiles::TileRank,
) -> bool {
    tile.is_dragon()
        || (tile.is_honor() && (tile.rank() == round_wind_rank || tile.rank() == seat_wind_rank))
}

/// There can be multiple groups that contribute to the yakuhai yaku
pub fn han_from_yakuhai_yaku(
    tile_grouping: &Vec<tiles::TileGroup>,
    hand_state: &state::HandState,
    player_state: &state::PlayerState,
) -> Option<u32> {
    // yakuhai can be scored with open hand
    let round_wind_rank = hand_state.round_wind.to_rank();
    let seat_wind_rank = player_state.seat_wind.to_rank();
    // you can get multiple han from yakuhai if the round wind + seat wind is the same, if there are multiple groups
    let mut yakuhai_han: u32 = 0;
    for tile_group in tile_grouping {
        match tile_group {
            tiles::TileGroup::Triplet { tiles, .. } => {
                assert!(tile_group.is_valid());
                if is_yakuhai_tile(&tiles[0], round_wind_rank, seat_wind_rank) {
                    yakuhai_han += 1;
                }
                if tiles[0].rank() == round_wind_rank && tiles[0].rank() == seat_wind_rank {
                    yakuhai_han += 1;
                }
            }
            tiles::TileGroup::Quad { tiles, .. } => {
                assert!(tile_group.is_valid());
                if is_yakuhai_tile(&tiles[0], round_wind_rank, seat_wind_rank) {
                    yakuhai_han += 1;
                }
                if tiles[0].rank() == round_wind_rank && tiles[0].rank() == seat_wind_rank {
                    yakuhai_han += 1;
                }
            }
            _ => continue,
        }
    }
    if yakuhai_han == 0 {
        None
    } else {
        Some(yakuhai_han)
    }
}

pub fn has_riichi_yaku(
    _tile_grouping: &Vec<tiles::TileGroup>,
    _hand_state: &state::HandState,
    player_state: &state::PlayerState,
) -> bool {
    // TODO in_riichi assumes player's hand is closed
    player_state.in_riichi
}

pub fn has_tanyao(
    tile_grouping: &Vec<tiles::TileGroup>,
    _hand_state: &state::HandState,
    _player_state: &state::PlayerState,
) -> bool {
    // tanyao can be scored with open hand
    for tile_group in tile_grouping {
        match tile_group {
            tiles::TileGroup::Quad { tiles, .. } => {
                assert!(tile_group.is_valid());
                if !tiles[0].is_simple() {
                    return false;
                }
            }
            tiles::TileGroup::Triplet { tiles, .. } => {
                assert!(tile_group.is_valid());
                if !tiles[0].is_simple() {
                    return false;
                }
            }
            tiles::TileGroup::Sequence { tiles, .. } => {
                assert!(tile_group.is_valid());
                if !tiles[0].is_simple() || !tiles[1].is_simple() || !tiles[2].is_simple() {
                    return false;
                }
            }
            tiles::TileGroup::Pair { tiles, .. } => {
                assert!(tile_group.is_valid());
                if !tiles[0].is_simple() {
                    return false;
                }
            }
            // all other tile groups are invalid (should not be found in a complete hand)
            _ => {
                panic!("Invalid tile group for a complete hand");
            }
        }
    }
    true
}

pub fn has_pinfu(
    tile_grouping: &Vec<tiles::TileGroup>,
    hand_state: &state::HandState,
    player_state: &state::PlayerState,
) -> bool {
    // pinfu is closed only, no quads or triplets allowed (only sequences + non-yakuhai pair)
    let round_wind_rank = hand_state.round_wind.to_rank();
    let seat_wind_rank = player_state.seat_wind.to_rank();
    let mut num_closed_sequences = 0;
    for tile_group in tile_grouping {
        match tile_group {
            tiles::TileGroup::Quad { .. } => {
                return false;
            }
            tiles::TileGroup::Triplet { .. } => {
                return false;
            }
            tiles::TileGroup::Sequence { open, .. } => {
                assert!(tile_group.is_valid());
                if !open {
                    return false;
                }
                num_closed_sequences += 1;
            }
            tiles::TileGroup::Pair { tiles, .. } => {
                assert!(tile_group.is_valid());
                if is_yakuhai_tile(&tiles[0], round_wind_rank, seat_wind_rank) {
                    return false;
                }
            }
            // all other tile groups are invalid (should not be found in a complete hand)
            _ => {
                panic!("Invalid tile group for a complete hand");
            }
        }
    }
    // pinfu must not be thirteen orphans or seven pairs
    if num_closed_sequences != 4 {
        return false;
    }
    // TODO additionally must check the wait pattern (must be a open-wait i.e. two-sided wait, not a closed-wait, edge-wait, or pair-wait)
    true
}

// TODO do we assume that the winning grouping is already checked as a winning hand?
// TODO we need to make sure that the fu is scored using the grouping that produces the maximum han
pub fn scoring_fu(
    player_tiles: &Vec<tiles::Tile>,
    added_tile: &tiles::Tile,
    tile_grouping: &Vec<tiles::TileGroup>,
    hand_state: &state::HandState,
    player_state: &state::PlayerState,
) -> u32 {
    let mut new_tiles = player_tiles.clone();
    new_tiles.push(added_tile.clone());
    let _existing_tile_groups: Vec<tiles::TileGroup> = Vec::new();

    // special case: chiitoitsu (seven pairs) is always scored as 25 fu
    if let Some(_seven_pairs_groups) =
        tile_grouping::seven_pairs_tile_grouping(&new_tiles, &_existing_tile_groups)
    {
        return 25;
    }

    // assert this is a winning hand, and get hand grouping(s)
    let _tile_groups = tile_grouping::tile_grouping(&new_tiles, &_existing_tile_groups)
        .expect("Should be a winning hand");

    // TODO we need to make sure that the fu is consistent with the grouping that scores the maximum han

    // fu from tile groups (triplets and quads earn fu based on open/closed and if the tile is simple or not)
    let mut fu_from_groups = 0;
    for tile_group in tile_grouping {
        fu_from_groups += match tile_group {
            tiles::TileGroup::Triplet { open, tiles } => {
                assert!(tile_group.is_valid());
                let mut triplet_fu = 2;
                if !tiles[0].is_simple() {
                    triplet_fu *= 2;
                }
                if !open {
                    triplet_fu *= 2;
                }
                triplet_fu
            }
            tiles::TileGroup::Quad { open, tiles, .. } => {
                assert!(tile_group.is_valid());
                let mut quad_fu = 8;
                if !tiles[0].is_simple() {
                    quad_fu *= 2;
                }
                if !open {
                    quad_fu *= 2;
                }
                quad_fu
            }
            _ => 0,
        };
    }
    let fu_from_groups = fu_from_groups;

    // TODO fu from waits
    let fu_from_wait = 0;

    // fu from pair (earns 2 fu if the tile would be yakuhai, 4 fu if the wind is both seat and dealer wind)
    let round_wind_rank = hand_state.round_wind.to_rank();
    let seat_wind_rank = player_state.seat_wind.to_rank();
    let pair_tile =
        tiles::get_pair_group(&tile_grouping).expect("Should be a pair in winning hand");
    let fu_from_pair = if is_yakuhai_tile(&pair_tile, round_wind_rank, seat_wind_rank) {
        if pair_tile.rank() == round_wind_rank && pair_tile.rank() == seat_wind_rank {
            4
        } else {
            2
        }
    } else {
        0
    };

    // fu from winning condition
    let is_hand_closed: bool = true;
    let winning_condition = player_state.winning_tile_source;

    let fu_from_winning_condition = match winning_condition.expect("Must be a winning tile source")
    {
        state::WinningTileSource::Discard => {
            if is_hand_closed {
                10
            } else {
                // TODO if no fu from tile groups or waiting pattern, then the 20 fu hand is forced to 30 fu
                0
            }
        }
        state::WinningTileSource::SelfDraw => {
            // TODO if the closed hand with tsumo satisfies all other criteria for pinfu, these 2 fu are not awarded (the 1 han for pinfu is awarded instead)
            2
        }
        state::WinningTileSource::DeadWall => {
            // TODO some scoring rule variations (rishan fu) don't award 2 fu for tsumo win off of kan replacement tile, as winning off of this tile awards the rinshan yaku (1 han)
            2
        }
        state::WinningTileSource::RobbingKan => {
            // TODO is there any fu for robbing a kan?
            0
        }
    };

    fu_from_groups + fu_from_pair + fu_from_wait + fu_from_winning_condition
}

#[cfg(test)]
mod tests {
    // importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_yakuhai_closed_white_dragon_triplet() {
        // winning hands taken from my Mahjong Soul logs
        // game: 4-player East round, Silver room, 2023-06-03 09:26
        // round: East 4 (0 repeat), winning hand by West (open hand, ron)
        // scoring: 4 han, 30 fu = 7700 pts (white dragon, dora x3 (7m, 8p))
        let tile_groups: Vec<tiles::TileGroup> = vec![
            tiles::TileGroup::Sequence {
                open: true,
                tiles: [
                    tiles::Tile::from_string("4s"),
                    tiles::Tile::from_string("5s"),
                    tiles::Tile::from_string("3s"),
                ],
            },
            tiles::TileGroup::Triplet {
                open: true,
                tiles: [
                    tiles::Tile::from_string("7m"),
                    tiles::Tile::from_string("7m"),
                    tiles::Tile::from_string("7m"),
                ],
            },
            tiles::TileGroup::Sequence {
                open: false,
                tiles: [
                    tiles::Tile::from_string("3m"),
                    tiles::Tile::from_string("4m"),
                    tiles::Tile::from_string("2m"),
                ],
            },
            tiles::TileGroup::Triplet {
                open: false,
                tiles: [
                    tiles::Tile::from_string("5z"), // white dragon
                    tiles::Tile::from_string("5z"),
                    tiles::Tile::from_string("5z"),
                ],
            },
            tiles::TileGroup::Pair {
                tiles: [
                    tiles::Tile::from_string("8m"),
                    tiles::Tile::from_string("8m"),
                ],
            },
        ];

        // check yaku
        let hand_state = state::HandState {
            round_wind: state::WindDirection::East,
            any_calls_made: true,
            tiles_remaining: 12,
            dora_indicators: vec![
                tiles::Tile::from_string("6m"),
                tiles::Tile::from_string("7p"),
            ],
            riichi_sticks: 1,
            honba_sticks: 0,
        };
        let player_state = state::PlayerState {
            discards: vec![
                tiles::Tile::from_string("4z"),
                tiles::Tile::from_string("3z"),
                tiles::Tile::from_string("8p"),
                tiles::Tile::from_string("1s"),
                tiles::Tile::from_string("1z"),
                tiles::Tile::from_string("1p"),
                tiles::Tile::from_string("2p"),
                tiles::Tile::from_string("9s"),
                tiles::Tile::from_string("9s"),
                tiles::Tile::from_string("0p"),
                tiles::Tile::from_string("2p"),
                tiles::Tile::from_string("6m"),
                tiles::Tile::from_string("4p"),
                tiles::Tile::from_string("8m"),
                tiles::Tile::from_string("1s"),
            ],
            seat_wind: state::WindDirection::West,
            in_riichi: false,
            in_double_riichi: false,
            in_ippatsu_turn: false,
            any_discards_called_by_others: false,
            winning_tile_source: Some(state::WinningTileSource::Discard), // from East (opposite player / toimen)
        };
        assert_eq!(
            han_from_yakuhai_yaku(&tile_groups, &hand_state, &player_state),
            Some(1)
        );
    }

    #[test]
    fn test_yakuhai_same_round_and_seat_wind() {
        // test multiple han from yakuhai
        let tile_groups: Vec<tiles::TileGroup> = vec![
            tiles::TileGroup::Triplet {
                open: true,
                tiles: [
                    tiles::Tile::from_string("2z"), // south wind
                    tiles::Tile::from_string("2z"),
                    tiles::Tile::from_string("2z"),
                ],
            },
            tiles::TileGroup::Triplet {
                open: true,
                tiles: [
                    tiles::Tile::from_string("1z"), // east wind
                    tiles::Tile::from_string("1z"),
                    tiles::Tile::from_string("1z"),
                ],
            },
            tiles::TileGroup::Sequence {
                open: false,
                tiles: [
                    tiles::Tile::from_string("1s"),
                    tiles::Tile::from_string("2s"),
                    tiles::Tile::from_string("3s"),
                ],
            },
            tiles::TileGroup::Triplet {
                open: false,
                tiles: [
                    tiles::Tile::from_string("7z"), // red dragon
                    tiles::Tile::from_string("7z"),
                    tiles::Tile::from_string("7z"),
                ],
            },
            tiles::TileGroup::Pair {
                tiles: [
                    tiles::Tile::from_string("8m"),
                    tiles::Tile::from_string("8m"),
                ],
            },
        ];

        // check yaku
        let hand_state = state::HandState {
            round_wind: state::WindDirection::South,
            any_calls_made: true,
            tiles_remaining: 10,
            dora_indicators: vec![tiles::Tile::from_string("1m")],
            riichi_sticks: 0,
            honba_sticks: 0,
        };
        let player_state = state::PlayerState {
            discards: vec![
                tiles::Tile::from_string("8p"),
                tiles::Tile::from_string("1s"),
                tiles::Tile::from_string("0p"),
                tiles::Tile::from_string("2p"),
                tiles::Tile::from_string("6m"),
                tiles::Tile::from_string("4p"),
                tiles::Tile::from_string("8m"),
                tiles::Tile::from_string("1s"),
            ],
            seat_wind: state::WindDirection::South,
            in_riichi: false,
            in_double_riichi: false,
            in_ippatsu_turn: false,
            any_discards_called_by_others: false,
            winning_tile_source: Some(state::WinningTileSource::Discard),
        };
        // south wind = 2 han (seat wind + round wind)
        // east wind = 0 han
        // red dragon = 1 han
        assert_eq!(
            han_from_yakuhai_yaku(&tile_groups, &hand_state, &player_state),
            Some(3)
        );
    }

    #[test]
    fn test_yakuhai_quads() {
        // test yakuhai from quads
        let tile_groups: Vec<tiles::TileGroup> = vec![
            tiles::TileGroup::Sequence {
                open: true,
                tiles: [
                    tiles::Tile::from_string("2p"),
                    tiles::Tile::from_string("3p"),
                    tiles::Tile::from_string("1p"),
                ],
            },
            tiles::TileGroup::Quad {
                open: true,
                added: false,
                tiles: [
                    tiles::Tile::from_string("1z"), // east wind
                    tiles::Tile::from_string("1z"),
                    tiles::Tile::from_string("1z"),
                    tiles::Tile::from_string("1z"),
                ],
            },
            tiles::TileGroup::Sequence {
                open: false,
                tiles: [
                    tiles::Tile::from_string("1s"),
                    tiles::Tile::from_string("2s"),
                    tiles::Tile::from_string("3s"),
                ],
            },
            tiles::TileGroup::Triplet {
                open: false,
                tiles: [
                    tiles::Tile::from_string("7s"),
                    tiles::Tile::from_string("7s"),
                    tiles::Tile::from_string("7s"),
                ],
            },
            tiles::TileGroup::Pair {
                tiles: [
                    tiles::Tile::from_string("8m"),
                    tiles::Tile::from_string("8m"),
                ],
            },
        ];

        // check yaku
        let hand_state = state::HandState {
            round_wind: state::WindDirection::East,
            any_calls_made: true,
            tiles_remaining: 10,
            dora_indicators: vec![tiles::Tile::from_string("1m")],
            riichi_sticks: 0,
            honba_sticks: 0,
        };
        let player_state = state::PlayerState {
            discards: vec![
                tiles::Tile::from_string("8p"),
                tiles::Tile::from_string("1s"),
                tiles::Tile::from_string("0p"),
                tiles::Tile::from_string("2p"),
                tiles::Tile::from_string("6m"),
                tiles::Tile::from_string("4p"),
                tiles::Tile::from_string("8m"),
                tiles::Tile::from_string("1s"),
            ],
            seat_wind: state::WindDirection::West,
            in_riichi: false,
            in_double_riichi: false,
            in_ippatsu_turn: false,
            any_discards_called_by_others: false,
            winning_tile_source: Some(state::WinningTileSource::Discard),
        };
        // east wind = 1 han (round wind)
        assert_eq!(
            han_from_yakuhai_yaku(&tile_groups, &hand_state, &player_state),
            Some(1)
        );
    }

    #[test]
    fn test_yakuhai_not_from_guest_winds() {
        // test no yakuhai from guest winds (neither seat wind nor round wind)
        let tile_groups: Vec<tiles::TileGroup> = vec![
            tiles::TileGroup::Sequence {
                open: true,
                tiles: [
                    tiles::Tile::from_string("2p"),
                    tiles::Tile::from_string("3p"),
                    tiles::Tile::from_string("1p"),
                ],
            },
            tiles::TileGroup::Triplet {
                open: true,
                tiles: [
                    tiles::Tile::from_string("2z"), // south wind
                    tiles::Tile::from_string("2z"),
                    tiles::Tile::from_string("2z"),
                ],
            },
            tiles::TileGroup::Sequence {
                open: false,
                tiles: [
                    tiles::Tile::from_string("1s"),
                    tiles::Tile::from_string("2s"),
                    tiles::Tile::from_string("3s"),
                ],
            },
            tiles::TileGroup::Triplet {
                open: false,
                tiles: [
                    tiles::Tile::from_string("7s"),
                    tiles::Tile::from_string("7s"),
                    tiles::Tile::from_string("7s"),
                ],
            },
            tiles::TileGroup::Pair {
                tiles: [
                    tiles::Tile::from_string("8m"),
                    tiles::Tile::from_string("8m"),
                ],
            },
        ];

        // check yaku
        let hand_state = state::HandState {
            round_wind: state::WindDirection::East,
            any_calls_made: true,
            tiles_remaining: 10,
            dora_indicators: vec![tiles::Tile::from_string("1m")],
            riichi_sticks: 0,
            honba_sticks: 0,
        };
        let player_state = state::PlayerState {
            discards: vec![
                tiles::Tile::from_string("8p"),
                tiles::Tile::from_string("1s"),
                tiles::Tile::from_string("0p"),
                tiles::Tile::from_string("2p"),
                tiles::Tile::from_string("6m"),
                tiles::Tile::from_string("4p"),
                tiles::Tile::from_string("8m"),
                tiles::Tile::from_string("1s"),
            ],
            seat_wind: state::WindDirection::West,
            in_riichi: false,
            in_double_riichi: false,
            in_ippatsu_turn: false,
            any_discards_called_by_others: false,
            winning_tile_source: Some(state::WinningTileSource::Discard),
        };
        // south wind = 0 han (round wind)
        assert_eq!(
            han_from_yakuhai_yaku(&tile_groups, &hand_state, &player_state),
            None
        );
    }

    #[test]
    fn test_tanyao_closed() {
        // test tanyao (closed hand)
        // example hand from https://riichi.wiki/Tanyao
        let tile_groups: Vec<tiles::TileGroup> = vec![
            tiles::TileGroup::Triplet {
                open: false,
                tiles: [
                    tiles::Tile::from_string("2s"),
                    tiles::Tile::from_string("2s"),
                    tiles::Tile::from_string("2s"),
                ],
            },
            tiles::TileGroup::Sequence {
                open: false,
                tiles: [
                    tiles::Tile::from_string("6s"),
                    tiles::Tile::from_string("7s"),
                    tiles::Tile::from_string("8s"),
                ],
            },
            tiles::TileGroup::Sequence {
                open: false,
                tiles: [
                    tiles::Tile::from_string("4m"),
                    tiles::Tile::from_string("5m"),
                    tiles::Tile::from_string("6m"),
                ],
            },
            tiles::TileGroup::Triplet {
                open: false,
                tiles: [
                    tiles::Tile::from_string("3p"),
                    tiles::Tile::from_string("3p"),
                    tiles::Tile::from_string("3p"),
                ],
            },
            tiles::TileGroup::Pair {
                tiles: [
                    tiles::Tile::from_string("5p"),
                    tiles::Tile::from_string("5p"),
                ],
            },
        ];

        // check yaku
        let hand_state = state::HandState {
            round_wind: state::WindDirection::East,
            any_calls_made: true,
            tiles_remaining: 40,
            dora_indicators: vec![tiles::Tile::from_string("2m")],
            riichi_sticks: 0,
            honba_sticks: 0,
        };
        let player_state = state::PlayerState {
            discards: vec![
                tiles::Tile::from_string("8p"),
                tiles::Tile::from_string("1s"),
            ],
            seat_wind: state::WindDirection::West,
            in_riichi: false,
            in_double_riichi: false,
            in_ippatsu_turn: false,
            any_discards_called_by_others: false,
            winning_tile_source: Some(state::WinningTileSource::Discard),
        };
        assert!(has_tanyao(&tile_groups, &hand_state, &player_state));
    }

    #[test]
    fn test_tanyao_open() {
        // test tanyao (open hand)
        // example hand from https://riichi.wiki/Tanyao
        let tile_groups: Vec<tiles::TileGroup> = vec![
            tiles::TileGroup::Sequence {
                open: false,
                tiles: [
                    tiles::Tile::from_string("6m"),
                    tiles::Tile::from_string("7m"),
                    tiles::Tile::from_string("8m"),
                ],
            },
            tiles::TileGroup::Sequence {
                open: false,
                tiles: [
                    tiles::Tile::from_string("3s"),
                    tiles::Tile::from_string("4s"),
                    tiles::Tile::from_string("5s"),
                ],
            },
            tiles::TileGroup::Triplet {
                open: false,
                tiles: [
                    tiles::Tile::from_string("3p"),
                    tiles::Tile::from_string("3p"),
                    tiles::Tile::from_string("3p"),
                ],
            },
            tiles::TileGroup::Triplet {
                open: true,
                tiles: [
                    tiles::Tile::from_string("2m"),
                    tiles::Tile::from_string("2m"),
                    tiles::Tile::from_string("2m"),
                ],
            },
            tiles::TileGroup::Pair {
                tiles: [
                    tiles::Tile::from_string("6p"),
                    tiles::Tile::from_string("6p"),
                ],
            },
        ];

        // check yaku
        let hand_state = state::HandState {
            round_wind: state::WindDirection::East,
            any_calls_made: true,
            tiles_remaining: 40,
            dora_indicators: vec![tiles::Tile::from_string("2m")],
            riichi_sticks: 0,
            honba_sticks: 0,
        };
        let player_state = state::PlayerState {
            discards: vec![
                tiles::Tile::from_string("8p"),
                tiles::Tile::from_string("1s"),
            ],
            seat_wind: state::WindDirection::West,
            in_riichi: false,
            in_double_riichi: false,
            in_ippatsu_turn: false,
            any_discards_called_by_others: false,
            winning_tile_source: Some(state::WinningTileSource::Discard),
        };
        assert!(has_tanyao(&tile_groups, &hand_state, &player_state));
    }

    #[test]
    fn test_pinfu() {
        // test pinfu
        // https://riichi.wiki/Pinfu
        let tile_groups: Vec<tiles::TileGroup> = vec![
            tiles::TileGroup::Sequence {
                open: false,
                tiles: [
                    tiles::Tile::from_string("1m"),
                    tiles::Tile::from_string("2m"),
                    tiles::Tile::from_string("3m"),
                ],
            },
            tiles::TileGroup::Sequence {
                open: false,
                tiles: [
                    tiles::Tile::from_string("2s"),
                    tiles::Tile::from_string("3s"),
                    tiles::Tile::from_string("4s"),
                ],
            },
            tiles::TileGroup::Sequence {
                open: false,
                tiles: [
                    tiles::Tile::from_string("7s"),
                    tiles::Tile::from_string("8s"),
                    tiles::Tile::from_string("9s"), // winning tile
                ],
            },
            tiles::TileGroup::Sequence {
                open: false,
                tiles: [
                    tiles::Tile::from_string("5p"),
                    tiles::Tile::from_string("6p"),
                    tiles::Tile::from_string("7p"),
                ],
            },
            tiles::TileGroup::Pair {
                tiles: [
                    tiles::Tile::from_string("9p"),
                    tiles::Tile::from_string("9p"),
                ],
            },
        ];

        // check yaku
        let hand_state = state::HandState {
            round_wind: state::WindDirection::East,
            any_calls_made: true,
            tiles_remaining: 40,
            dora_indicators: vec![tiles::Tile::from_string("2m")],
            riichi_sticks: 0,
            honba_sticks: 0,
        };
        let player_state = state::PlayerState {
            discards: vec![
                tiles::Tile::from_string("8p"),
                tiles::Tile::from_string("1s"),
            ],
            seat_wind: state::WindDirection::West,
            in_riichi: false,
            in_double_riichi: false,
            in_ippatsu_turn: false,
            any_discards_called_by_others: false,
            winning_tile_source: Some(state::WinningTileSource::Discard),
        };
        assert!(has_pinfu(&tile_groups, &hand_state, &player_state));
    }
}
