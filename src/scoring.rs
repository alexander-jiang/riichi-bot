use std::collections::HashSet;

use crate::mahjong_tile::{
    MahjongTileCountArray, MahjongTileId, MahjongTileNumberedSuit, MahjongWindOrder,
};
use crate::shanten::{
    get_chiitoi_shanten, get_hand_interpretations_min_shanten, get_kokushi_shanten,
    HandInterpretation, MeldType, TileMeld,
};

const CHIITOI_FU: u8 = 25;

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

impl WinningTileSource {
    /// Whether the winning tile keeps the hand / tile group closed (only self-draw from live wall or draw from dead-wall after kan are closed)
    pub fn is_closed(&self) -> bool {
        match self {
            Self::SelfDraw { .. } | Self::AfterKan => true,
            Self::Discard { .. } | Self::RobbingKan => false,
        }
    }
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

pub trait Yaku {
    /// Returns the number of han earned from this yaku (0 if the hand is not eligible for yaku).
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
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
        _melded_tiles: &Vec<TileMeld>,
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

fn get_total_groups(
    hand_interpretation: &HandInterpretation,
    _tile_melds: &Vec<TileMeld>,
) -> Vec<TileMeld> {
    // let mut total_groups = hand_interpretation.groups.clone();
    // total_groups.extend(tile_melds.clone());
    // println!(
    //     "total groups = [{}]",
    //     total_groups
    //         .iter()
    //         .map(|arg0| ToString::to_string(arg0))
    //         .collect::<Vec<String>>()
    //         .join(", ")
    // );
    // total_groups
    hand_interpretation.groups.clone()
}

fn get_total_num_pairs_before_winning_tile(
    hand_interpretation: &HandInterpretation,
    melded_tiles: &Vec<TileMeld>,
) -> u8 {
    let mut num_pairs = 0;
    let total_groups = get_total_groups(hand_interpretation, melded_tiles);
    for tile_group in total_groups.iter() {
        if tile_group.meld_type == MeldType::Pair {
            num_pairs += 1;
        }
    }
    num_pairs
}

pub struct YakuhaiYaku;
impl Yaku for YakuhaiYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        let mut yakuhai_han = 0;
        let num_pairs = get_total_num_pairs_before_winning_tile(hand_interpretation, melded_tiles);
        // check for a triplet or quadruplet of a yakuhai tile (dragon, seat wind, or round wind)
        let total_groups = get_total_groups(hand_interpretation, melded_tiles);
        for tile_group in total_groups.iter() {
            if tile_group.meld_type == MeldType::Triplet
                || tile_group.meld_type == MeldType::Quadruplet
            {
                let tile_id = tile_group.tile_ids.get(0).unwrap();
                if tile_id.is_dragon_tile() {
                    yakuhai_han += 1;
                    println!("group of {} is worth 1 han from yakuhai", *tile_id);
                    continue;
                }
                if tile_id.is_wind_tile() {
                    if YakuhaiYaku::is_double_yakuhai_tile(*tile_id, hand_info) {
                        yakuhai_han += 2;
                        println!("group of {} is worth 2 han from yakuhai", *tile_id);
                    } else if YakuhaiYaku::is_yakuhai_tile(*tile_id, hand_info) {
                        yakuhai_han += 1;
                        println!("group of {} is worth 1 han from yakuhai", *tile_id);
                    }
                    continue;
                }
            } else if num_pairs == 2 && tile_group.meld_type == MeldType::Pair {
                // shanpon wait, then check if the winning tile completes a yakuhai triplet
                let pair_tile_id = tile_group.tile_ids.get(0).unwrap();
                if winning_tile == *pair_tile_id {
                    if YakuhaiYaku::is_double_yakuhai_tile(winning_tile, hand_info) {
                        yakuhai_han += 2;
                        println!(
                            "completing triplet of {} (from shanpon) is worth 2 han from yakuhai",
                            winning_tile
                        );
                    } else if YakuhaiYaku::is_yakuhai_tile(winning_tile, hand_info) {
                        yakuhai_han += 1;
                        println!(
                            "completing triplet of {} (from shanpon) is worth 1 han from yakuhai",
                            winning_tile
                        );
                    }
                    continue;
                }
            }
        }
        println!("Hand has {yakuhai_han} han from yakuhai");
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
        melded_tiles: &Vec<TileMeld>,
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
        let total_groups = get_total_groups(hand_interpretation, melded_tiles);
        for tile_group in total_groups.iter() {
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

pub struct MenzenTsumoYaku;
impl Yaku for MenzenTsumoYaku {
    fn han_value(
        &self,
        _hand_interpretation: &HandInterpretation,
        _melded_tiles: &Vec<TileMeld>,
        _winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // menzen tsumo means the hand was closed and it won by self-draw (including draw from dead-wall after kan i.e. rinshan kaihou)
        if hand_info.hand_state == HandState::Open || !winning_tile_info.source.is_closed() {
            0
        } else {
            1
        }
    }
}

pub struct RinshanYaku;
impl Yaku for RinshanYaku {
    fn han_value(
        &self,
        _hand_interpretation: &HandInterpretation,
        _melded_tiles: &Vec<TileMeld>,
        _winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // rinshan kaihou means the winning tile was the tile that was drawn immediately after declaring a kan
        if winning_tile_info.source == WinningTileSource::AfterKan {
            1
        } else {
            0
        }
    }
}

pub struct TanyaoYaku;
impl Yaku for TanyaoYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // every tile (including winning tile) is a simple tile (i.e. not a terminal or an honor tile, i.e. only numbered tiles from 2-8 are allowed)
        let total_tile_count_array = hand_interpretation
            .clone()
            .total_tile_count_array
            .add_tile_ids(vec![winning_tile])
            .add_tile_ids(
                melded_tiles
                    .iter()
                    .map(|tile_meld| tile_meld.clone().tile_ids)
                    .flatten()
                    .collect(),
            );

        let total_tile_ids = total_tile_count_array.to_tile_ids();
        for tile_id in total_tile_ids.iter() {
            if !tile_id.is_simple_tile() {
                return 0;
            }
        }
        1
    }
}

pub struct ChiitoiYaku;
impl Yaku for ChiitoiYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        if ChiitoiYaku::is_chiitoi_hand(hand_interpretation, melded_tiles, winning_tile) {
            println!("chiitoi hand: {}", hand_interpretation);
            2
        } else {
            0
        }
    }
}

impl ChiitoiYaku {
    pub fn is_chiitoi_hand(
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
    ) -> bool {
        // after combining the winning tile and the existing hand tiles, there must be 7 pairs (quads don't count)
        let mut num_pairs = 0;
        let total_groups = get_total_groups(hand_interpretation, melded_tiles);
        for tile_group in total_groups.iter() {
            if tile_group.meld_type == MeldType::Pair {
                num_pairs += 1;
            } else if tile_group.meld_type == MeldType::SingleTile {
                let single_tile_id = tile_group.tile_ids.get(0).unwrap();
                if *single_tile_id == winning_tile {
                    num_pairs += 1;
                }
            }
        }
        num_pairs == 7
    }
}

pub struct ToitoiYaku;
impl Yaku for ToitoiYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // TODO some yakuman imply toitoi: suuankou and chinroutou
        if ToitoiYaku::is_toitoi_hand(hand_interpretation, melded_tiles, winning_tile) {
            2
        } else {
            0
        }
    }
}

impl ToitoiYaku {
    pub fn is_toitoi_hand(
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
    ) -> bool {
        // every group must be a triplet or quad
        let mut num_triplet_or_quad = 0;
        let total_groups = get_total_groups(hand_interpretation, melded_tiles);
        for tile_group in total_groups.iter() {
            if tile_group.meld_type == MeldType::Triplet
                || tile_group.meld_type == MeldType::Quadruplet
            {
                num_triplet_or_quad += 1;
            } else if tile_group.meld_type == MeldType::Pair {
                let pair_tile_id = tile_group.tile_ids.get(0).unwrap();
                if *pair_tile_id == winning_tile {
                    num_triplet_or_quad += 1;
                }
            } else if tile_group.meld_type == MeldType::SingleTile {
                // this should be a tanki wait (do we just assume the hand is a valid winning hand?)
                let single_tile_id = tile_group.tile_ids.get(0).unwrap();
                if *single_tile_id != winning_tile {
                    return false;
                }
            }
        }
        num_triplet_or_quad == 4
    }
}

pub struct HonitsuYaku;
impl Yaku for HonitsuYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        if ChinitsuYaku::is_chinitsu_hand(hand_interpretation, melded_tiles, winning_tile) {
            println!("hand is chinitsu, which implies honitsu");
            0 // chinitsu includes honitsu, so you can't count both
        } else if HonitsuYaku::is_honitsu_hand(hand_interpretation, melded_tiles, winning_tile) {
            match _hand_info.hand_state {
                HandState::Closed { .. } => 3,
                HandState::Open => 2,
            }
        } else {
            // println!("hand is neither chinitsu nor honitsu");
            0
        }
    }
}

impl HonitsuYaku {
    pub fn is_honitsu_hand(
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
    ) -> bool {
        let total_tile_count_array = hand_interpretation
            .clone()
            .total_tile_count_array
            .add_tile_ids(vec![winning_tile])
            .add_tile_ids(
                melded_tiles
                    .iter()
                    .map(|tile_meld| tile_meld.clone().tile_ids)
                    .flatten()
                    .collect(),
            );
        let total_tile_ids = total_tile_count_array.to_tile_ids();

        // the hand must contain tiles from exactly one numbered suit (honor tiles are okay)
        let mut numbered_suit: Option<MahjongTileNumberedSuit> = None;
        for tile_id in total_tile_ids.iter() {
            if tile_id.is_numbered_tile() {
                let tile_suit = tile_id.get_num_tile_suit().unwrap();
                if numbered_suit.is_none() {
                    numbered_suit = Some(tile_suit);
                } else if numbered_suit.unwrap() != tile_suit {
                    return false;
                }
            }
        }
        numbered_suit.is_some()
    }
}

pub struct ChinitsuYaku;
impl Yaku for ChinitsuYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        if ChinitsuYaku::is_chinitsu_hand(hand_interpretation, melded_tiles, winning_tile) {
            match _hand_info.hand_state {
                HandState::Closed { .. } => 6,
                HandState::Open => 5,
            }
        } else {
            0
        }
    }
}

impl ChinitsuYaku {
    pub fn is_chinitsu_hand(
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
    ) -> bool {
        let total_tile_count_array = hand_interpretation
            .clone()
            .total_tile_count_array
            .add_tile_ids(vec![winning_tile])
            .add_tile_ids(
                melded_tiles
                    .iter()
                    .map(|tile_meld| tile_meld.clone().tile_ids)
                    .flatten()
                    .collect(),
            );
        let total_tile_ids = total_tile_count_array.to_tile_ids();

        // the hand must contain tiles from exactly one numbered suit (honor tiles are not allowed)
        let mut numbered_suit: Option<MahjongTileNumberedSuit> = None;
        let mut has_honor_tile = false;
        for tile_id in total_tile_ids.iter() {
            if tile_id.is_numbered_tile() {
                let tile_suit = tile_id.get_num_tile_suit().unwrap();
                if numbered_suit.is_none() {
                    numbered_suit = Some(tile_suit);
                } else if numbered_suit.unwrap() != tile_suit {
                    return false;
                }
            } else if tile_id.is_honor_tile() {
                has_honor_tile = true;
            }
        }
        numbered_suit.is_some() && !has_honor_tile
    }
}

/// Note: maximum possible fu is 110 fu, so using a u8 to represent fu is okay
/// Note: this isn't checking for yakuman
pub fn compute_han_and_fu(
    hand_tiles: MahjongTileCountArray,
    melded_tiles: Vec<TileMeld>,
    winning_tile: MahjongTileId,
    hand_info: HandInfo,
    winning_tile_info: WinningTileInfo,
) -> (u8, u8) {
    // First, get the possible hand shape interpretations
    // TODO eliminate some interpretations based on winning tile (some interpretations aren't possible)
    let mut interpretations = get_hand_interpretations_min_shanten(hand_tiles, &melded_tiles, 0);

    // check chiitoi and kokushi separately (as the `get_hand_interpretations_min_shanten` doesn't include chiitoi (seven pairs) or kokushi (thirteen orphans))
    let total_tile_count_array = hand_tiles.clone().add_tile_ids(vec![winning_tile]);
    let chiitoi_shanten = get_chiitoi_shanten(total_tile_count_array, &melded_tiles);
    if chiitoi_shanten == -1 {
        let mut chiitoi_groups = Vec::new();
        for tile_id in total_tile_count_array.to_distinct_tile_ids() {
            chiitoi_groups.push(TileMeld {
                meld_type: MeldType::Pair,
                tile_ids: vec![tile_id, tile_id],
                is_closed: true,
            });
        }
        interpretations.push(HandInterpretation {
            total_tile_count_array: total_tile_count_array,
            groups: chiitoi_groups,
        });
    }
    let kokushi_shanten = get_kokushi_shanten(total_tile_count_array, &melded_tiles);
    if kokushi_shanten == -1 {
        let mut kokushi_groups: Vec<TileMeld> = Vec::new();
        for tile_id in total_tile_count_array.to_distinct_tile_ids() {
            if total_tile_count_array.get_tile_id_count(&tile_id) == 1 {
                kokushi_groups.push(TileMeld {
                    meld_type: MeldType::SingleTile,
                    tile_ids: vec![tile_id],
                    is_closed: true,
                });
            } else if total_tile_count_array.get_tile_id_count(&tile_id) == 2 {
                kokushi_groups.push(TileMeld {
                    meld_type: MeldType::Pair,
                    tile_ids: vec![tile_id, tile_id],
                    is_closed: true,
                });
            }
        }
        interpretations.push(HandInterpretation {
            total_tile_count_array: total_tile_count_array,
            groups: kokushi_groups,
        });
    }

    let mut max_scoring_han_fu = (0u8, 0u8);
    for interpretation in interpretations {
        // for each hand shape interpretation, compute han and fu
        let interpretation_scoring = compute_han_and_fu_hand_interpretation(
            &interpretation,
            &melded_tiles,
            winning_tile,
            &hand_info,
            &winning_tile_info,
        );
        // then pick the hand shape interpretation with the highest score (sort by han, then fu)
        if interpretation_scoring.0 > max_scoring_han_fu.0
            || (interpretation_scoring.0 == max_scoring_han_fu.0
                && interpretation_scoring.1 > max_scoring_han_fu.1)
        {
            max_scoring_han_fu = interpretation_scoring;
        }
    }

    println!(
        "overall hand scoring result: {} han, {} fu",
        max_scoring_han_fu.0, max_scoring_han_fu.1
    );
    max_scoring_han_fu
}

pub fn get_yaku_list() -> Vec<Box<dyn Yaku>> {
    // Box<dyn Yaku> as Yaku is a trait (which doesn't have a defined size), so Box<dyn Yaku> is like a pointer
    vec![
        Box::new(RiichiYaku),
        Box::new(PinfuYaku),
        Box::new(YakuhaiYaku),
        Box::new(MenzenTsumoYaku),
        Box::new(TanyaoYaku),
        // TODO iipeikou (1 han, closed only), ittsu (2 han if closed, 1 han if opened)
        Box::new(ChiitoiYaku),
        // TODO shousangen (2 yaku, not including the yakuhai) / daisangen (yakuman)
        // TODO thirteen orphans (yakuman)
        Box::new(ToitoiYaku),
        // TODO sanshoku doujun (2 han, can be open)
        Box::new(HonitsuYaku),
        Box::new(ChinitsuYaku),
        // TODO chankan (robbing kan), haitei / houtei
        Box::new(RinshanYaku),
        // TODO ippatsu
        // TODO ryanpeikou (3 han, closed only), sanshoku (2 han if closed, 1 han if open)
        // TODO double riichi (2 han)
        // TODO chanta (2 han if closed, 1 han if opened: every group has honor or terminal), junchan (3 han if closed, 2 han if opened: all groups have a terminal)
        // TODO honroutou (2 han: every tile is a terminal or honor, but effectively 4 han because it requires chiitoi or toitoi)
        // TODO sanankou (2 han, can be open), suuankou (yakuman)
        // TODO sankantsu (2 han, can be open), suukantsu (yakuman)
    ]
}

pub fn compute_han_and_fu_hand_interpretation(
    hand_interpretation: &HandInterpretation,
    melded_tiles: &Vec<TileMeld>,
    winning_tile: MahjongTileId,
    hand_info: &HandInfo,
    winning_tile_info: &WinningTileInfo,
) -> (u8, u8) {
    println!(
        "computing han and fu for hand interpretation: {}, winning tile {}",
        hand_interpretation, winning_tile
    );
    // for each yaku, does this hand meet the conditions for yaku? if so, add the correct number of han
    let mut total_han = 0;
    for yaku in get_yaku_list() {
        total_han += yaku.han_value(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            hand_info,
            winning_tile_info,
        );
    }
    if total_han == 0 {
        // if you don't have at least 1 han from yaku, don't bother
        return (0, 0);
    }

    // add han from dora
    let mut han_from_dora = 0;
    let mut total_hand_tiles = hand_interpretation.total_tile_count_array;
    total_hand_tiles.0[usize::from(winning_tile)] += 1;
    for dora_tile in hand_info.dora_tiles.iter() {
        han_from_dora += total_hand_tiles.get_tile_id_count(dora_tile);
    }
    total_han += han_from_dora;
    // TODO what about red fives?? (red fives aren't considered in the MahjongTileCountArray)

    // chiitoi is locked to 25 fu (no rounding)
    if ChiitoiYaku::is_chiitoi_hand(hand_interpretation, melded_tiles, winning_tile) {
        println!("-> hand is chiitoi: {} han, {} fu", total_han, CHIITOI_FU);
        return (total_han, CHIITOI_FU);
    }

    let mut fu = compute_raw_fu(
        hand_interpretation,
        melded_tiles,
        winning_tile,
        hand_info,
        winning_tile_info,
    );
    // round up the fu to nearest 10
    let fu_remainder = fu % 10;
    if fu_remainder != 0 {
        fu += 10 - fu_remainder;
    }

    println!(
        "-> hand interpretation scoring result: {} han, {} fu",
        total_han, fu
    );
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
    println!("meld of {} is worth {} fu", *melded_tile_id, meld_fu);
    meld_fu
}

/// computes fu without rounding up, even if the hand already has enough han to make fu irrelevant for normal scoring
pub fn compute_raw_fu(
    hand_interpretation: &HandInterpretation,
    melded_tiles: &Vec<TileMeld>,
    winning_tile: MahjongTileId,
    hand_info: &HandInfo,
    winning_tile_info: &WinningTileInfo,
) -> u8 {
    // special cases: chiitoi is always 25 fu. and thirteen orphans doesn't have a defined fu, so I will assign it 0 fu (doesn't matter as it's a yakuman)
    if ChiitoiYaku::is_chiitoi_hand(hand_interpretation, melded_tiles, winning_tile) {
        println!("chiitoi is always 25 fu");
        return CHIITOI_FU;
    }
    // TODO move this logic into a impl ThirteenOrphansYaku
    let mut distinct_terminals_and_honors = HashSet::new();
    for tile_group in hand_interpretation.groups.iter() {
        if tile_group.meld_type == MeldType::SingleTile {
            let single_tile = tile_group.tile_ids.get(0).unwrap();
            if single_tile.is_terminal_tile() || single_tile.is_honor_tile() {
                distinct_terminals_and_honors.insert(single_tile.clone());
            }
        }
    }
    if distinct_terminals_and_honors.len() == 13 {
        println!("kokushi musou (thirteen orphans) doesn't need to count fu");
        return 0;
    }

    // every hand starts with 20 fu
    let mut fu = 20;

    // check hand shape + wait type
    let mut pair_so_far = None;
    for tile_group in hand_interpretation.groups.iter() {
        // check for triplets or quadruplets (need to check shanpon wait separately, as neither pair in the shanpon wait is considered a complete group)
        if tile_group.meld_type == MeldType::Triplet || tile_group.meld_type == MeldType::Quadruplet
        {
            let meld_fu = compute_fu_for_triplet_or_quad(tile_group);
            fu += meld_fu;
            continue;
        }

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
                let is_completed_shanpon_triplet_closed = winning_tile_info.source.is_closed();
                let completed_shanpon_triplet =
                    TileMeld::new(new_tile_ids, is_completed_shanpon_triplet_closed);

                let meld_fu = compute_fu_for_triplet_or_quad(&completed_shanpon_triplet);
                fu += meld_fu;
            }
        } else if tile_group.meld_type == MeldType::Kanchan
            || tile_group.meld_type == MeldType::Penchan
            || tile_group.meld_type == MeldType::SingleTile
        {
            // check wait type (a kanchan, penchan, or tanki wait is worth +2 fu)
            println!("wait is kanchan, penchan or tanki: worth 2 fu");
            fu += 2;
        } else if tile_group.meld_type == MeldType::Ryanmen {
            // ryanmen wait is worth 0 fu
        }
    }

    // check pair (is it a yakuhai tile)
    let pair_so_far = pair_so_far.expect("Should have identified a pair");
    let pair_tile_id = pair_so_far.tile_ids.get(0).unwrap();
    if YakuhaiYaku::is_double_yakuhai_tile(*pair_tile_id, hand_info) {
        println!("pair is yakuhai tile & double wind: worth 4 fu");
        fu += 4;
    } else if YakuhaiYaku::is_yakuhai_tile(*pair_tile_id, hand_info) {
        println!("pair is yakuhai tile: worth 2 fu");
        fu += 2;
    }

    // Lastly, check winning condition (we do this check last because we can check for "open pinfu")
    match winning_tile_info.source.is_closed() {
        false => {
            match hand_info.hand_state {
                HandState::Closed { .. } => {
                    println!("closed hand win by ron: worth 10 fu");
                    fu += 10;
                }
                HandState::Open => {
                    if fu == 20 {
                        // if the hand is open, wins by ron, and has no other fu, then it gains 2 fu
                        println!("open hand win by ron with no other fu: worth 2 fu");
                        fu += 2;
                    }
                }
            }
        }
        true => {
            // winning by tsumo is +2 fu (whether the hand is open or closed) - unless the hand is scored for pinfu
            if PinfuYaku.han_value(
                hand_interpretation,
                melded_tiles,
                winning_tile,
                hand_info,
                winning_tile_info,
            ) == 0
            {
                println!("win by tsumo (and not pinfu): worth 2 fu");
                fu += 2;
            }
        }
    }

    println!("total fu (before rounding) = {fu}");
    fu
}

// TODO we can look this up in an array
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

// TODO we can look this up in a table
pub fn compute_ron_score(han: u8, fu: u8, hand_info: &HandInfo) -> u32 {
    let base_points = compute_base_points(han, fu);

    match hand_info.seat_wind {
        // ron as dealer = 6x base-points vs. ron as non-dealer = 4x base-points
        MahjongWindOrder::East => base_points * 6,
        _ => base_points * 4,
    }
}

// TODO we can look this up in a table
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
        let melded_tiles = Vec::new();
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
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                tsumo.clone()
            )
        );

        // Dealer Ron = 3 han 40 fu
        assert_eq!(
            (3, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Tsumo = 4 han 30 fu
        assert_eq!(
            (4, 30),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                tsumo.clone()
            )
        );

        // Non-dealer Ron = 3 han 40 fu
        assert_eq!(
            (3, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }

    #[test]
    fn jpml_pro_test_part1_hand_scoring_q2() {
        // https://mahjong-ny.com/features/sample-pro-test/
        // 2) 56789m344556p88s, win on 4m, dora = 3p, riichi
        let hand = MahjongTileCountArray::from_text("56789m344556p88s");
        let melded_tiles = Vec::new();
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
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                tsumo.clone()
            )
        );

        // Dealer Ron = 3 han 30 fu
        assert_eq!(
            (3, 30),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Tsumo = 4 han 20 fu
        assert_eq!(
            (4, 20),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                tsumo.clone()
            )
        );

        // Non-dealer Ron = 3 han 30 fu
        assert_eq!(
            (3, 30),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }

    #[test]
    fn jpml_pro_test_part1_hand_scoring_q3() {
        // https://mahjong-ny.com/features/sample-pro-test/
        // 3) 44556m4488p3399s, win on 6m, dora = 1m, not riichi
        let hand = MahjongTileCountArray::from_text("44556m4488p3399s");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("1m");
        let win_tile = MahjongTileId::from_text("6m").unwrap();

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
                riichi_info: RiichiInfo::NoRiichi,
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

        // Dealer Tsumo = 3 han 25 fu
        assert_eq!(
            (3, 25),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                tsumo.clone()
            )
        );

        // Dealer Ron = 2 han 25 fu
        assert_eq!(
            (2, 25),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Tsumo = 3 han 25 fu
        assert_eq!(
            (3, 25),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                tsumo.clone()
            )
        );

        // Non-dealer Ron = 2 han 25 fu
        assert_eq!(
            (2, 25),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }

    #[test]
    fn jpml_pro_test_part1_hand_scoring_q4() {
        // https://mahjong-ny.com/features/sample-pro-test/
        // 4) 334455p1166z + 333z open, win on 6z, dora = 8p
        let hand = MahjongTileCountArray::from_text("334455p1166z");
        let melded_tiles = vec![TileMeld {
            meld_type: MeldType::Triplet,
            tile_ids: get_tile_ids_from_string("333z"),
            is_closed: false,
        }];
        let dora_tiles = get_tile_ids_from_string("8p");
        let win_tile = MahjongTileId::from_text("6z").unwrap();

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
            hand_state: HandState::Open,
            round_wind: MahjongWindOrder::East,
            seat_wind: MahjongWindOrder::East,
            round_number: 1,
            honba_counter: 0,
            dora_tiles: dora_tiles.clone(),
        };
        let mut as_nondealer = as_dealer.clone();
        as_nondealer.seat_wind = MahjongWindOrder::South;
        let as_nondealer = as_nondealer;

        // Dealer Tsumo = 3 han 40 fu
        assert_eq!(
            (3, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                tsumo.clone()
            )
        );

        // Dealer Ron = 3 han 40 fu
        assert_eq!(
            (3, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Tsumo = 3 han 40 fu
        assert_eq!(
            (3, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                tsumo.clone()
            )
        );

        // Non-dealer Ron = 3 han 30 fu
        assert_eq!(
            (3, 30),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }

    #[test]
    fn jpml_pro_test_part1_hand_scoring_q5() {
        // https://mahjong-ny.com/features/sample-pro-test/
        // 5) 111m111p1112s + 777z open, win on 3s, dora = 4m
        let hand = MahjongTileCountArray::from_text("111m111p1112s");
        let melded_tiles = vec![TileMeld {
            meld_type: MeldType::Triplet,
            tile_ids: get_tile_ids_from_string("777z"),
            is_closed: false,
        }];
        let dora_tiles = get_tile_ids_from_string("4m");
        let win_tile = MahjongTileId::from_text("3s").unwrap();

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
            hand_state: HandState::Open,
            round_wind: MahjongWindOrder::East,
            seat_wind: MahjongWindOrder::East,
            round_number: 1,
            honba_counter: 0,
            dora_tiles: dora_tiles.clone(),
        };
        let mut as_nondealer = as_dealer.clone();
        as_nondealer.seat_wind = MahjongWindOrder::South;
        let as_nondealer = as_nondealer;

        // Dealer Tsumo = 2 han 50 fu
        assert_eq!(
            (2, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                tsumo.clone()
            )
        );

        // Dealer Ron = 2 han 50 fu
        assert_eq!(
            (2, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Tsumo = 2 han 50 fu
        assert_eq!(
            (2, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                tsumo.clone()
            )
        );

        // Non-dealer Ron = 2 han 50 fu
        assert_eq!(
            (2, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }

    #[test]
    fn jpml_pro_test_part1_hand_scoring_q6() {
        // https://mahjong-ny.com/features/sample-pro-test/
        // 6) 2223344m + 678m open + 3333s closed, win on 4m, dora = 8s
        let hand = MahjongTileCountArray::from_text("2223344m");
        let melded_tiles = vec![
            TileMeld {
                meld_type: MeldType::Sequence,
                tile_ids: get_tile_ids_from_string("678m"),
                is_closed: false,
            },
            TileMeld {
                meld_type: MeldType::Quadruplet,
                tile_ids: get_tile_ids_from_string("3333s"),
                is_closed: true,
            },
        ];
        let dora_tiles = get_tile_ids_from_string("8s");
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
            hand_state: HandState::Open,
            round_wind: MahjongWindOrder::East,
            seat_wind: MahjongWindOrder::East,
            round_number: 1,
            honba_counter: 0,
            dora_tiles: dora_tiles.clone(),
        };
        let mut as_nondealer = as_dealer.clone();
        as_nondealer.seat_wind = MahjongWindOrder::South;
        let as_nondealer = as_nondealer;

        // Dealer Tsumo = 3 han 50 fu
        assert_eq!(
            (3, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                tsumo.clone()
            )
        );

        // Dealer Ron = 1 han 50 fu
        assert_eq!(
            (1, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Tsumo = 3 han 50 fu
        assert_eq!(
            (3, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                tsumo.clone()
            )
        );

        // Non-dealer Ron = 1 han 50 fu
        assert_eq!(
            (1, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }

    #[test]
    fn jpml_pro_test_part1_hand_scoring_q7() {
        // https://mahjong-ny.com/features/sample-pro-test/
        // 7) 45567p11s + 222z open + 111z open, win on 6p, dora = 2m
        let hand = MahjongTileCountArray::from_text("45567p11s");
        let melded_tiles = vec![
            TileMeld {
                meld_type: MeldType::Triplet,
                tile_ids: get_tile_ids_from_string("222z"),
                is_closed: false,
            },
            TileMeld {
                meld_type: MeldType::Triplet,
                tile_ids: get_tile_ids_from_string("111z"),
                is_closed: false,
            },
        ];
        let dora_tiles = get_tile_ids_from_string("2m");
        let win_tile = MahjongTileId::from_text("6p").unwrap();

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
            hand_state: HandState::Open,
            round_wind: MahjongWindOrder::East,
            seat_wind: MahjongWindOrder::East,
            round_number: 1,
            honba_counter: 0,
            dora_tiles: dora_tiles.clone(),
        };
        let mut as_nondealer = as_dealer.clone();
        as_nondealer.seat_wind = MahjongWindOrder::South;
        let as_nondealer = as_nondealer;

        // Dealer Tsumo = 2 han 40 fu
        assert_eq!(
            (2, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                tsumo.clone()
            )
        );

        // Dealer Ron = 2 han 30 fu
        assert_eq!(
            (2, 30),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Tsumo = 2 han 40 fu
        assert_eq!(
            (2, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                tsumo.clone()
            )
        );

        // Non-dealer Ron = 2 han 30 fu
        assert_eq!(
            (2, 30),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }

    #[test]
    fn jpml_pro_test_part1_hand_scoring_q8() {
        // https://mahjong-ny.com/features/sample-pro-test/
        // 8) 2223444777888s, win on 4s, dora = 3s
        let hand = MahjongTileCountArray::from_text("2223444777888s");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("3s");
        let win_tile = MahjongTileId::from_text("4s").unwrap();

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
                riichi_info: RiichiInfo::NoRiichi,
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

        // Dealer Tsumo = 10 han 40 fu
        assert_eq!(
            (10, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                tsumo.clone()
            )
        );

        // Dealer Ron = 8 han 30 fu
        assert_eq!(
            (8, 30),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Tsumo = 10 han 40 fu
        assert_eq!(
            (10, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                tsumo.clone()
            )
        );

        // Non-dealer Ron = 8 han 30 fu
        assert_eq!(
            (8, 30),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }

    #[test]
    fn jpml_pro_test_part1_hand_scoring_q9() {
        // https://mahjong-ny.com/features/sample-pro-test/
        // 9) 79m678p888s66z + 1111m (closed kan), win on 8m, dora = 4p
        let hand = MahjongTileCountArray::from_text("79m678p888s66z");
        let melded_tiles = vec![TileMeld {
            meld_type: MeldType::Quadruplet,
            tile_ids: get_tile_ids_from_string("1111m"),
            is_closed: true,
        }];
        let dora_tiles = get_tile_ids_from_string("4p");
        let win_tile = MahjongTileId::from_text("8m").unwrap();

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

        // Dealer Tsumo = 2 han 70 fu
        assert_eq!(
            (2, 70),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                tsumo.clone()
            )
        );

        // Dealer Ron = 1 han 70 fu
        assert_eq!(
            (1, 70),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Tsumo = 2 han 70 fu
        assert_eq!(
            (2, 70),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                tsumo.clone()
            )
        );

        // Non-dealer Ron = 1 han 70 fu
        assert_eq!(
            (1, 70),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }

    #[test]
    fn jpml_pro_test_part1_hand_scoring_q10() {
        // https://mahjong-ny.com/features/sample-pro-test/
        // 10) 4466p678s + 5555m open + 8888p open, win on 4p, dora = 4m
        let hand = MahjongTileCountArray::from_text("4466p678s");
        let melded_tiles = vec![
            TileMeld {
                meld_type: MeldType::Quadruplet,
                tile_ids: get_tile_ids_from_string("5555m"),
                is_closed: false,
            },
            TileMeld {
                meld_type: MeldType::Quadruplet,
                tile_ids: get_tile_ids_from_string("8888p"),
                is_closed: false,
            },
        ];
        let dora_tiles = get_tile_ids_from_string("4m");
        let win_tile = MahjongTileId::from_text("4p").unwrap();

        // winning methods (ron vs tsumo)
        let ron = WinningTileInfo {
            source: WinningTileSource::Discard {
                is_last_discard: false,
            },
        };
        let tsumo = WinningTileInfo {
            source: WinningTileSource::AfterKan,
        };

        // situations (dealer in East-1 vs. south in East-1)
        let as_dealer = HandInfo {
            hand_state: HandState::Open,
            round_wind: MahjongWindOrder::East,
            seat_wind: MahjongWindOrder::East,
            round_number: 1,
            honba_counter: 0,
            dora_tiles: dora_tiles.clone(),
        };
        let mut as_nondealer = as_dealer.clone();
        as_nondealer.seat_wind = MahjongWindOrder::South;
        let as_nondealer = as_nondealer;

        // Dealer Tsumo = 2 han 50 fu (assuming that win by rinshan counts +2 fu just like tsumo)
        assert_eq!(
            (2, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                tsumo.clone()
            )
        );

        // Dealer Ron = 1 han 40 fu
        assert_eq!(
            (1, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Tsumo = 2 han 50 fu (assuming that win by rinshan counts +2 fu just like tsumo)
        assert_eq!(
            (2, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                tsumo.clone()
            )
        );

        // Non-dealer Ron = 1 han 40 fu
        assert_eq!(
            (1, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }
}
