use std::collections::HashSet;

use crate::mahjong_tile::{MahjongTileCountArray, MahjongTileId, MahjongWindOrder};
use crate::shanten::{get_hand_interpretations, HandInterpretation, MeldType, TileMeld};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RiichiInfo {
    Riichi {
        is_ippatsu: bool,
        is_double_riichi: bool,
    },
    NoRiichi,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HandState {
    Open,
    Closed { riichi_info: RiichiInfo },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WinningTileSource {
    Discard {
        is_last_discard: bool, // winning on the last discard is the yaku "houtei raoyui"
    }, // TODO add player id that discarded?
    SelfDraw {
        is_first_draw: bool, // note: winning on first draw as dealer is the yakuman "tenhou" / "blessing of heaven". Winning on the first draw as non-dealer (with no prior tile calls by any player) is the yakuman "chiihou" / "blessing of earth"
        is_last_draw: bool,  // winning on the last draw is the yaku "haitei raoyue"
    },
    AfterKan, // winning on the replacement tile drawn after declaring a kan is the yaku "rinshan kaihou"
    RobbingKan, // winning by robbing a kan is the yaku "chankan"
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WinningTileInfo {
    source: WinningTileSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HandInfo {
    hand_state: HandState,
    round_wind: MahjongWindOrder,
    seat_wind: MahjongWindOrder,
    round_number: u8, // 1-4 inclusive (represents how many different players have been dealer, including this hand)
    honba_counter: u16, // this number +1 is the "repeat"/renchan number e.g. east round, 2nd dealer, 1 honba -> East-2, 1st bonus round
    dora_tiles: Vec<MahjongTileId>,
}

// Box<Yaku> as Yaku is a trait (which doesn't have a defined size), so Box<Yaku> is like a pointer
const YAKU_LIST: Vec<Box<dyn Yaku>> = vec![];

pub trait Yaku {
    /// Returns the number of han earned from this yaku (0 if the hand is not eligible for yaku).
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8;
}

pub struct RiichiYaku;
impl Yaku for RiichiYaku {
    fn han_value(
        &self,
        _hand_interpretation: &HandInterpretation,
        _winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        match &hand_info.hand_state {
            HandState::Closed { riichi_info } => match riichi_info {
                RiichiInfo::Riichi { .. } => 1,
                RiichiInfo::NoRiichi => 0,
            },
            HandState::Open => 0,
        }
    }
}

// TODO ippatsu, double riichi

pub struct YakuhaiYaku;
impl Yaku for YakuhaiYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        _winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        let mut yakuhai_han = 0;
        // check for a triplet or quadruplet of a yakuhai tile (dragon, seat wind, or round wind)
        for tile_group in hand_interpretation.groups.iter() {
            if tile_group.meld_type == MeldType::Triplet
                || tile_group.meld_type == MeldType::Quadruplet
            {
                let tile_id = tile_group.tile_ids.get(0).unwrap();
                if tile_id.is_dragon_tile() {
                    yakuhai_han += 1;
                    continue;
                }
                if tile_id.is_wind_tile() {
                    let tile_wind = tile_id.get_wind().unwrap();
                    // Note that if the wind is both the round and seat wind, this counts for 2 han
                    if tile_wind == hand_info.round_wind {
                        yakuhai_han += 1;
                    }
                    if tile_wind == hand_info.seat_wind {
                        yakuhai_han += 1;
                    }
                }
            }
        }
        yakuhai_han
    }
}

impl YakuhaiYaku {
    /// helper function for checking if a tile id is a yakuhai tile
    fn is_yakuhai_tile(tile_id: MahjongTileId, hand_info: &HandInfo) -> bool {
        if tile_id.is_dragon_tile() {
            true
        } else if tile_id.is_wind_tile() {
            let tile_wind = tile_id.get_wind().unwrap();
            tile_wind == hand_info.round_wind || tile_wind == hand_info.seat_wind
        } else {
            false
        }
    }

    /// helper function for checking if a tile id is a double-yakuhai tile (i.e. if the tile is both the round wind and seat wind)
    fn is_double_yakuhai_tile(tile_id: MahjongTileId, hand_info: &HandInfo) -> bool {
        if tile_id.is_wind_tile() {
            let tile_wind = tile_id.get_wind().unwrap();
            tile_wind == hand_info.round_wind && tile_wind == hand_info.seat_wind
        } else {
            false
        }
    }
}

pub struct PinfuYaku;
impl Yaku for PinfuYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // pinfu must be closed
        if hand_info.hand_state == HandState::Open {
            return 0;
        }

        let mut ryanmen_wait = None;
        let mut pairs: Vec<TileMeld> = Vec::new();
        let mut num_completed_sequences = 0;
        // pinfu must not include triplets or quadruplets (there must be 3 complete sequences, and a ryanmen wait)
        for tile_group in hand_interpretation.groups.iter() {
            if tile_group.meld_type == MeldType::Triplet
                || tile_group.meld_type == MeldType::Quadruplet
            {
                return 0;
            } else if tile_group.meld_type == MeldType::Ryanmen {
                if ryanmen_wait.is_some() {
                    // how could there be two ryanmen waits? this is not pinfu but also this might be a bug??
                    return 0;
                }
                ryanmen_wait = Some(tile_group.clone());
            } else if tile_group.meld_type == MeldType::Pair {
                pairs.push(tile_group.clone());
            } else if tile_group.meld_type == MeldType::Sequence {
                num_completed_sequences += 1;
            }
        }

        // winning tile must complete a ryanmen wait
        if let Some(ryanmen_group) = ryanmen_wait {
            if !ryanmen_group
                .tile_ids_to_complete_group()
                .contains(&winning_tile)
            {
                // we shouldn't hit this case, might be a bug??
                return 0;
            }
        } else {
            // if no ryanmen wait, then it's not pinfu
            return 0;
        }

        // the checks below exclude non-standard winning shapes like 13 orphans and chiitoi
        if num_completed_sequences != 3 {
            return 0;
        }

        // the pair must not be yakuhai
        if pairs.len() != 1 {
            return 0;
        }
        let pair_tile = pairs.get(0).unwrap().tile_ids.get(0).unwrap();
        if YakuhaiYaku::is_yakuhai_tile(*pair_tile, hand_info) {
            return 0;
        }

        // if we've passed all conditions above, the hand is eligible for pinfu (1 han)
        1
    }
}

/// note: maximum possible fu is 110 fu, so using a u8 to represent fu is okay
pub fn compute_han_and_fu(
    hand_tiles: MahjongTileCountArray,
    winning_tile: MahjongTileId,
    hand_info: HandInfo,
    winning_tile_info: WinningTileInfo,
) -> (u8, u8) {
    // First, get the possible hand shape interpretations
    // TODO eliminate some interpretations based on winning tile (some interpretations aren't possible)
    let interpretations = get_hand_interpretations(hand_tiles);
    let mut max_scoring_han_fu = (0u8, 0u8);
    for interpretation in interpretations {
        // for each hand shape interpretation, compute han and fu
        let interpretation_scoring = compute_han_and_fu_hand_interpretation(
            &interpretation,
            winning_tile,
            &hand_info,
            &winning_tile_info,
        );
        // then pick the hand shape interpretation with the highest score (sort by han, then fu)
        if interpretation_scoring.0 > max_scoring_han_fu.0
            && interpretation_scoring.1 > max_scoring_han_fu.1
        {
            max_scoring_han_fu = interpretation_scoring;
        }
    }
    max_scoring_han_fu
}

pub fn compute_han_and_fu_hand_interpretation(
    hand_interpretation: &HandInterpretation,
    winning_tile: MahjongTileId,
    hand_info: &HandInfo,
    winning_tile_info: &WinningTileInfo,
) -> (u8, u8) {
    // for each yaku, does this hand meet the conditions for yaku? if so, add the correct number of han
    let mut total_han = 0;
    for yaku in YAKU_LIST {
        total_han += yaku.han_value(
            hand_interpretation,
            winning_tile,
            hand_info,
            winning_tile_info,
        );
    }

    let mut fu = compute_raw_fu(
        hand_interpretation,
        winning_tile,
        hand_info,
        winning_tile_info,
    );
    // round up the fu to nearest 10
    let fu_remainder = fu % 10;
    if fu_remainder != 0 {
        fu += 10 - fu_remainder;
    }

    (total_han, fu)
}

fn compute_fu_for_triplet_or_quad(tile_meld: &TileMeld) -> u8 {
    let melded_tile_id = tile_meld.tile_ids.get(0).unwrap();
    let is_quad = tile_meld.meld_type == MeldType::Quadruplet;
    let is_closed = tile_meld.is_closed;
    let is_terminal_or_honor = melded_tile_id.is_terminal_tile() || melded_tile_id.is_honor_tile();

    let mut meld_fu = 2; // base fu for an open triplet of simple tiles
    if is_closed {
        // 2x fu for closed triplet/quad (instead of open)
        meld_fu = meld_fu << 1;
    }
    if is_terminal_or_honor {
        // 2x fu for a triplet/quad of a terminal or honor tile (instead of a triplet/quad of a simple tile)
        meld_fu = meld_fu << 1;
    }
    if is_quad {
        // 4x fu for a quad (instead of a triplet)
        meld_fu = meld_fu << 2;
    }
    meld_fu
}

/// computes fu without rounding up, even if the hand already has enough han to make fu irrelevant for normal scoring
pub fn compute_raw_fu(
    hand_interpretation: &HandInterpretation,
    winning_tile: MahjongTileId,
    hand_info: &HandInfo,
    winning_tile_info: &WinningTileInfo,
) -> u8 {
    // special cases: chiitoi is always 25 fu. and thirteen orphans doesn't have a defined fu, so I will assign it 0 fu (doesn't matter as it's a yakuman)
    let mut num_pairs = 0;
    let mut distinct_terminals_and_honors = HashSet::new();
    for tile_group in hand_interpretation.groups.iter() {
        if tile_group.meld_type == MeldType::Pair {
            num_pairs += 1;
        }
        if tile_group.meld_type == MeldType::SingleTile {
            let single_tile = tile_group.tile_ids.get(0).unwrap();
            if single_tile.is_terminal_tile() || single_tile.is_honor_tile() {
                distinct_terminals_and_honors.insert(single_tile.clone());
            }
        }
    }
    if num_pairs == 6 {
        return 25;
    }
    if distinct_terminals_and_honors.len() == 13 {
        return 0;
    }

    // every hand starts with 20 fu
    let mut fu = 20;

    // check hand shape + wait type
    let mut pair_so_far = None;
    for tile_group in hand_interpretation.groups.iter() {
        if tile_group.is_complete() {
            // check for triplets or quadruplets (need to check shanpon wait separately, as neither pair in the shanpon wait is considered a complete group)
            if tile_group.meld_type == MeldType::Triplet
                || tile_group.meld_type == MeldType::Quadruplet
            {
                let meld_fu = compute_fu_for_triplet_or_quad(tile_group);
                fu += meld_fu;
            }
        } else {
            // check wait type (a kanchan, penchan, or tanki wait is worth +2 fu)
            if tile_group.meld_type == MeldType::Pair {
                if pair_so_far.is_none() {
                    pair_so_far = Some(tile_group.clone());
                } else {
                    // it's a shanpon wait, need to compute the fu from the triplet completed by winning tile
                    // and leave the pair_so_far updated to the pair of the hand (not the triplet)

                    // which pair was completed for the win?
                    let mut completed_shanpon_pair = pair_so_far.clone().unwrap();
                    if completed_shanpon_pair.tile_ids.contains(&winning_tile) {
                        // the previous value of `pair_so_far` is the completed triplet, so switch the pair of the hand to `tile_group`
                        pair_so_far = Some(tile_group.clone());
                    } else {
                        // `tile_group` is the completed triplet, so the previous value of `pair_so_far` is indeed the pair of the hand
                        completed_shanpon_pair = tile_group.clone();
                    }
                    let mut new_tile_ids = completed_shanpon_pair.tile_ids.clone();
                    new_tile_ids.push(*completed_shanpon_pair.tile_ids.get(0).unwrap());
                    let is_completed_shanpon_triplet_closed = match winning_tile_info.source {
                        WinningTileSource::SelfDraw { .. } | WinningTileSource::AfterKan => true,
                        WinningTileSource::Discard { .. } | WinningTileSource::RobbingKan => false,
                    };
                    let completed_shanpon_triplet =
                        TileMeld::new(new_tile_ids, is_completed_shanpon_triplet_closed);

                    let meld_fu = compute_fu_for_triplet_or_quad(&completed_shanpon_triplet);
                    fu += meld_fu;
                }
            } else {
                if tile_group.meld_type == MeldType::Kanchan
                    || tile_group.meld_type == MeldType::Penchan
                    || tile_group.meld_type == MeldType::SingleTile
                {
                    fu += 2;
                } else if tile_group.meld_type == MeldType::Ryanmen {
                    // ryanmen wait is worth 0 fu
                    fu += 0;
                } else {
                    // unexpected wait??
                    return 0;
                }
            }
        }
    }

    // check pair (is it a yakuhai tile)
    let pair_so_far = pair_so_far.expect("Should have identified a pair");
    let pair_tile_id = pair_so_far.tile_ids.get(0).unwrap();
    if YakuhaiYaku::is_double_yakuhai_tile(*pair_tile_id, hand_info) {
        fu += 4;
    } else if YakuhaiYaku::is_yakuhai_tile(*pair_tile_id, hand_info) {
        fu += 2;
    }

    // Lastly, check winning condition (we do this check last because we can check for "open pinfu")
    match winning_tile_info.source {
        WinningTileSource::Discard { .. } | WinningTileSource::RobbingKan => {
            match hand_info.hand_state {
                HandState::Closed { .. } => {
                    fu += 10;
                }
                HandState::Open => {
                    if fu == 20 {
                        // if the hand is open, wins by ron, and has no other fu, then it gains 2 fu
                        fu += 2;
                    }
                }
            }
        }
        WinningTileSource::SelfDraw { .. } | WinningTileSource::AfterKan => {
            // winning by tsumo is always +2 fu (whether the hand is open or closed)
            fu += 2;
        }
    }

    fu
}

fn compute_base_points(han: u8, fu: u8) -> u32 {
    match (han, fu) {
        // don't need fu to compute base points for mangan or higher
        (han, _fu) if han >= 13 => (u32::from(han) / 13) * 8000, // kazoe yakuman (i.e. "counted yakuman"), double yakuman is scored as 26 han
        (han, _fu) if han >= 11 => 6000,                         // sanbaiman
        (han, _fu) if han >= 8 => 4000,                          // baiman
        (han, _fu) if han >= 6 => 4000,                          // haneman
        (han, _fu) if (han == 5 || (han == 4 && fu >= 40) || (han == 3 && fu >= 70)) => 2000, // mangan
        // formula for base points below 2000: fu * ( 2^(2+han) ), capped at 2000 points
        (han, fu) => std::cmp::min(u32::from(fu) << (2 + han), 2000),
    }
}

pub fn compute_ron_score(han: u8, fu: u8, hand_info: &HandInfo) -> u32 {
    let base_points = compute_base_points(han, fu);

    match hand_info.seat_wind {
        // ron as dealer = 6x base-points vs. ron as non-dealer = 4x base-points
        MahjongWindOrder::East => base_points * 6,
        _ => base_points * 4,
    }
}

pub fn compute_tsumo_score(han: u8, fu: u8, hand_info: &HandInfo) -> (u32, u32, u32) {
    let base_points = compute_base_points(han, fu);

    match hand_info.seat_wind {
        // tsumo as dealer = 2x base-points from each other player vs. tsumo as non-dealer = 1x base-points from each non-dealer opponent + 2x base-points from dealer opponent
        MahjongWindOrder::East => (base_points * 2, base_points * 2, base_points * 2),
        _ => (base_points * 1, base_points * 1, base_points * 2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mahjong_tile::get_tile_ids_from_string;

    #[test]
    fn jpml_pro_test_part1_hand_scoring_q1() {
        // https://mahjong-ny.com/features/sample-pro-test/
        // 1) 123m11222p23456s, win on 1s, dora = 1p, riichi
        let hand = MahjongTileCountArray::from_text("123m11222p23456s");
        let dora_tiles = get_tile_ids_from_string("1p");
        let win_tile = MahjongTileId::from_text("1s").unwrap();

        // winning methods (ron vs tsumo)
        let ron = WinningTileInfo {
            source: WinningTileSource::Discard {
                is_last_discard: false,
            },
        };
        let tsumo = WinningTileInfo {
            source: WinningTileSource::SelfDraw {
                is_first_draw: false,
                is_last_draw: false,
            },
        };

        // situations (dealer in East-1 vs. south in East-1)
        let as_dealer = HandInfo {
            hand_state: HandState::Closed {
                riichi_info: RiichiInfo::Riichi {
                    is_ippatsu: false,
                    is_double_riichi: false,
                },
            },
            round_wind: MahjongWindOrder::East,
            seat_wind: MahjongWindOrder::East,
            round_number: 1,
            honba_counter: 0,
            dora_tiles: dora_tiles.clone(),
        };
        let mut as_nondealer = as_dealer.clone();
        as_nondealer.seat_wind = MahjongWindOrder::South;
        let as_nondealer = as_nondealer;

        // Dealer Tsumo = 4 han 30 fu
        assert_eq!(
            (4, 30),
            compute_han_and_fu(hand.clone(), win_tile, as_dealer.clone(), tsumo.clone())
        );

        // Dealer Ron = 3 han 40 fu
        assert_eq!(
            (3, 40),
            compute_han_and_fu(hand.clone(), win_tile, as_dealer.clone(), ron.clone())
        );

        // Non-dealer Tsumo = 4 han 30 fu
        assert_eq!(
            (4, 30),
            compute_han_and_fu(hand.clone(), win_tile, as_nondealer.clone(), tsumo.clone())
        );

        // Non-dealer Ron = 3 han 40 fu
        assert_eq!(
            (3, 40),
            compute_han_and_fu(hand.clone(), win_tile, as_nondealer.clone(), ron.clone())
        );

        // let hand_info_options = vec![as_dealer, as_nondealer];
        // let winning_tile_info_options = vec![ron, tsumo];
        // for hand_info in hand_info_options {
        //     for winning_tile_info in winning_tile_info_options {
        //     }
        // }
    }

    #[test]
    fn jpml_pro_test_part1_hand_scoring_q2() {
        // https://mahjong-ny.com/features/sample-pro-test/
        // 2) 56789m344556p88s, win on 4m, dora = 3p, riichi
        let hand = MahjongTileCountArray::from_text("56789m344556p88s");
        let dora_tiles = get_tile_ids_from_string("3p");
        let win_tile = MahjongTileId::from_text("4m").unwrap();

        // winning methods (ron vs tsumo)
        let ron = WinningTileInfo {
            source: WinningTileSource::Discard {
                is_last_discard: false,
            },
        };
        let tsumo = WinningTileInfo {
            source: WinningTileSource::SelfDraw {
                is_first_draw: false,
                is_last_draw: false,
            },
        };

        // situations (dealer in East-1 vs. south in East-1)
        let as_dealer = HandInfo {
            hand_state: HandState::Closed {
                riichi_info: RiichiInfo::Riichi {
                    is_ippatsu: false,
                    is_double_riichi: false,
                },
            },
            round_wind: MahjongWindOrder::East,
            seat_wind: MahjongWindOrder::East,
            round_number: 1,
            honba_counter: 0,
            dora_tiles: dora_tiles.clone(),
        };
        let mut as_nondealer = as_dealer.clone();
        as_nondealer.seat_wind = MahjongWindOrder::South;
        let as_nondealer = as_nondealer;

        // Dealer Tsumo = 4 han 20 fu
        assert_eq!(
            (4, 20),
            compute_han_and_fu(hand.clone(), win_tile, as_dealer.clone(), tsumo.clone())
        );

        // Dealer Ron = 3 han 30 fu
        assert_eq!(
            (3, 30),
            compute_han_and_fu(hand.clone(), win_tile, as_dealer.clone(), ron.clone())
        );

        // Non-dealer Tsumo = 4 han 20 fu
        assert_eq!(
            (4, 20),
            compute_han_and_fu(hand.clone(), win_tile, as_nondealer.clone(), tsumo.clone())
        );

        // Non-dealer Ron = 3 han 30 fu
        assert_eq!(
            (3, 30),
            compute_han_and_fu(hand.clone(), win_tile, as_nondealer.clone(), ron.clone())
        );

        // let hand_info_options = vec![as_dealer, as_nondealer];
        // let winning_tile_info_options = vec![ron, tsumo];
        // for hand_info in hand_info_options {
        //     for winning_tile_info in winning_tile_info_options {
        //     }
        // }
    }
}
