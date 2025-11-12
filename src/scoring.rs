use std::collections::{HashMap, HashSet};

use crate::mahjong_tile::{
    tile_ids_to_string, MahjongTileCountArray, MahjongTileId, MahjongTileNumberedSuit,
    MahjongWindOrder,
};
use crate::shanten::{
    get_chiitoi_shanten, get_hand_interpretations_min_shanten, get_kokushi_shanten,
    get_shanten_optimized, HandInterpretation, MeldType, TileMeld,
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

impl HandState {
    pub fn is_closed(&self) -> bool {
        match self {
            Self::Closed { .. } => true,
            Self::Open => false,
        }
    }
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
                RiichiInfo::Riichi {
                    is_double_riichi, ..
                } => {
                    if !(*is_double_riichi) {
                        println!("1 han from riichi");
                        1
                    } else {
                        // double riichi and riichi can't both be counted
                        0
                    }
                }
                RiichiInfo::NoRiichi => 0,
            },
            HandState::Open => 0,
        }
    }
}

// TODO test this function, test how the hand interpretations are generated when there's declared melds, do the hand interpretations include the winning tile?
// TODO should this include the melded tiles? or are melded tiles already in hand_interpretation?
fn get_total_groups_after_winning_tile(
    hand_interpretation: &HandInterpretation,
    _melded_tiles: &Vec<TileMeld>,
    winning_tile: MahjongTileId,
    winning_tile_info: &WinningTileInfo,
) -> Vec<TileMeld> {
    let tile_groups = hand_interpretation.groups.clone();
    // tile_groups.extend(melded_tiles.clone());

    // println!(
    //     "tile groups after winning tile: {}",
    //     tile_groups
    //         .iter()
    //         .map(|grp| grp.to_string())
    //         .collect::<Vec<String>>()
    //         .join(", ")
    // );
    if ChiitoiYaku::is_chiitoi_hand(hand_interpretation, winning_tile) {
        let mut chiitoi_tile_melds = Vec::new();
        let mut num_pairs = 0;
        for tile_meld in tile_groups {
            // TODO test how this works with hand_interpretation on an open hand, and on a chiitoi hand
            if tile_meld.meld_type == MeldType::Pair {
                num_pairs += 1;
                chiitoi_tile_melds.push(tile_meld.clone());
            } else if tile_meld.meld_type == MeldType::SingleTile
                && tile_meld
                    .tile_ids_to_advance_group()
                    .contains(&winning_tile)
            {
                let mut new_tile_ids = tile_meld.tile_ids;
                new_tile_ids.push(winning_tile);
                let new_is_closed = tile_meld.is_closed && winning_tile_info.source.is_closed();
                let new_tile_meld = TileMeld::new(new_tile_ids, new_is_closed);
                // println!(
                //     "winning tile {} completes pair in chiitoi: {}",
                //     winning_tile, new_tile_meld
                // );
                num_pairs += 1;
                chiitoi_tile_melds.push(new_tile_meld);
            }
        }
        if num_pairs != 7 {
            panic!("hand is missing 7 pairs (for chiitoi)");
        }
        return chiitoi_tile_melds;
    }

    let mut new_tile_melds = Vec::new();
    let mut complete_groups = 0;
    let mut num_pairs = 0;
    for tile_meld in tile_groups {
        if tile_meld.is_complete() {
            println!("complete group: {}", tile_meld);
            complete_groups += 1;
            new_tile_melds.push(tile_meld.clone());
        } else if tile_meld.meld_type == MeldType::SingleTile
            && tile_meld
                .tile_ids_to_advance_group()
                .contains(&winning_tile)
        {
            let mut new_tile_ids = tile_meld.tile_ids;
            new_tile_ids.push(winning_tile);

            let new_is_closed = tile_meld.is_closed && winning_tile_info.source.is_closed();
            let new_tile_meld = TileMeld::new(new_tile_ids, new_is_closed);
            num_pairs += 1;
            println!(
                "winning tile {} forms pair: {}",
                winning_tile, new_tile_meld
            );
            new_tile_melds.push(new_tile_meld);
        } else if tile_meld
            .tile_ids_to_complete_group()
            .contains(&winning_tile)
        {
            let mut new_tile_ids = tile_meld.tile_ids;
            new_tile_ids.push(winning_tile);

            let new_is_closed = tile_meld.is_closed && winning_tile_info.source.is_closed();
            let new_tile_meld = TileMeld::new(new_tile_ids, new_is_closed);
            println!(
                "winning tile {} makes group: {}",
                winning_tile, new_tile_meld
            );
            complete_groups += 1;
            new_tile_melds.push(new_tile_meld);
        } else if tile_meld.meld_type == MeldType::Pair {
            println!("a pair: {}", tile_meld);
            num_pairs += 1;
            new_tile_melds.push(tile_meld.clone());
        }
    }
    if complete_groups != 4 || num_pairs != 1 {
        panic!(
            "hand is missing 4 complete groups and 1 pair: found {} complete groups and {} pairs",
            complete_groups, num_pairs
        );
    }
    new_tile_melds
}

pub struct YakuhaiYaku;
impl Yaku for YakuhaiYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        let mut yakuhai_han = 0;
        // check for a triplet or quadruplet of a yakuhai tile (dragon, seat wind, or round wind)
        let total_groups = get_total_groups_after_winning_tile(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        );
        for tile_group in total_groups.iter() {
            if tile_group.meld_type == MeldType::Triplet
                || tile_group.meld_type == MeldType::Quadruplet
            {
                let tile_id = tile_group.tile_ids.get(0).unwrap();
                if tile_id.is_dragon_tile() {
                    yakuhai_han += 1;
                    // println!("group of {} is worth 1 han from yakuhai", *tile_id);
                    continue;
                }
                if tile_id.is_wind_tile() {
                    if YakuhaiYaku::is_double_yakuhai_tile(*tile_id, hand_info) {
                        yakuhai_han += 2;
                        // println!("group of {} is worth 2 han from yakuhai", *tile_id);
                    } else if YakuhaiYaku::is_yakuhai_tile(*tile_id, hand_info) {
                        yakuhai_han += 1;
                        // println!("group of {} is worth 1 han from yakuhai", *tile_id);
                    }
                    continue;
                }
            }
        }
        if yakuhai_han > 0 {
            println!("Hand has {yakuhai_han} han from yakuhai");
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

pub struct ShousangenYaku;
impl Yaku for ShousangenYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        if ShousangenYaku::is_shousangen_hand(hand_interpretation, melded_tiles, winning_tile) {
            2
        } else {
            0
        }
    }
}

impl ShousangenYaku {
    fn is_shousangen_hand(
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
    ) -> bool {
        // TODO should consolidate this?
        let mut hand_tile_count_array = hand_interpretation.total_tile_count_array;
        for tile_meld in melded_tiles {
            hand_tile_count_array = hand_tile_count_array.add_tile_ids(tile_meld.tile_ids.clone());
        }
        hand_tile_count_array = hand_tile_count_array.add_tile_ids(vec![winning_tile]);

        let white_dragon_count =
            hand_tile_count_array.get_tile_id_count(&MahjongTileId::from_text("5z").unwrap());
        let green_dragon_count =
            hand_tile_count_array.get_tile_id_count(&MahjongTileId::from_text("6z").unwrap());
        let red_dragon_count =
            hand_tile_count_array.get_tile_id_count(&MahjongTileId::from_text("7z").unwrap());
        let mut dragon_triplets = 0;
        let mut dragon_pairs = 0;
        if white_dragon_count >= 3 {
            dragon_triplets += 1;
        } else if white_dragon_count == 2 {
            dragon_pairs += 1;
        }

        if green_dragon_count >= 3 {
            dragon_triplets += 1;
        } else if green_dragon_count == 2 {
            dragon_pairs += 1;
        }

        if red_dragon_count >= 3 {
            dragon_triplets += 1;
        } else if red_dragon_count == 2 {
            dragon_pairs += 1;
        }

        dragon_triplets == 2 && dragon_pairs == 1
    }
}

pub struct PinfuYaku;
impl Yaku for PinfuYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        _melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // pinfu must be closed
        if !hand_info.hand_state.is_closed() {
            return 0;
        }

        let mut ryanmen_wait = None;
        let mut pairs: Vec<TileMeld> = Vec::new();
        let mut num_completed_sequences = 0;
        // pinfu must not include triplets or quadruplets (there must be 3 complete sequences, and a ryanmen wait)
        // (note that we can't use the get_total_groups_after_winning_tile function because we need to check the ryanmen wait)
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
        println!("1 han from pinfu");
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
        if !hand_info.hand_state.is_closed() || !winning_tile_info.source.is_closed() {
            // println!(
            //     "not menzentsumo: hand state is_closed = {}, winning_tile_source is_closed = {}",
            //     hand_info.hand_state.is_closed(),
            //     winning_tile_info.source.is_closed()
            // );
            0
        } else {
            println!("1 han from menzentsumo");
            1
        }
    }
}

pub struct IppatsuYaku;
impl Yaku for IppatsuYaku {
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
                RiichiInfo::Riichi { is_ippatsu, .. } if *is_ippatsu => {
                    println!("1 han from ippatsu");
                    1
                }
                _ => 0,
            },
            _ => 0,
        }
    }
}

pub struct HaiteiYaku;
impl Yaku for HaiteiYaku {
    fn han_value(
        &self,
        _hand_interpretation: &HandInterpretation,
        _melded_tiles: &Vec<TileMeld>,
        _winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        match winning_tile_info.source {
            WinningTileSource::SelfDraw { is_last_draw, .. } => {
                if is_last_draw {
                    println!("1 han from haitei (win on last draw)");
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}

pub struct HouteiYaku;
impl Yaku for HouteiYaku {
    fn han_value(
        &self,
        _hand_interpretation: &HandInterpretation,
        _melded_tiles: &Vec<TileMeld>,
        _winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        match winning_tile_info.source {
            WinningTileSource::Discard { is_last_discard } => {
                if is_last_discard {
                    println!("1 han from houtei (win on last discard)");
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}

pub struct ChankanYaku;
impl Yaku for ChankanYaku {
    fn han_value(
        &self,
        _hand_interpretation: &HandInterpretation,
        _melded_tiles: &Vec<TileMeld>,
        _winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        match winning_tile_info.source {
            WinningTileSource::RobbingKan => {
                println!("1 han from chankan (win by robbing kan)");
                1
            }
            _ => 0,
        }
    }
}

pub struct IipeikouYaku;
impl Yaku for IipeikouYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // iipeikou must be closed
        if !hand_info.hand_state.is_closed() {
            return 0;
        }
        // ryanpeikou implies iipeikou, so can't count a hand for iipeikou if it counts for ryanpeikou
        if RyanpeikouYaku::is_ryanpeikou_hand(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            hand_info,
            winning_tile_info,
        ) {
            return 0;
        }

        let total_groups = get_total_groups_after_winning_tile(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        );
        let mut tile_ids_sets: HashSet<Vec<MahjongTileId>> = HashSet::new();
        for tile_group in total_groups {
            if tile_group.meld_type != MeldType::Sequence {
                continue;
            }
            let tile_group_tile_ids = tile_group.tile_ids;
            if tile_ids_sets.contains(&tile_group_tile_ids) {
                println!(
                    "1 han from iipeikou (tiles = {})",
                    tile_ids_to_string(&tile_group_tile_ids)
                );
                return 1;
            } else {
                tile_ids_sets.insert(tile_group_tile_ids);
            }
        }
        0
    }
}

pub struct RyanpeikouYaku;
impl Yaku for RyanpeikouYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        if RyanpeikouYaku::is_ryanpeikou_hand(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            hand_info,
            winning_tile_info,
        ) {
            3
        } else {
            0
        }
    }
}

impl RyanpeikouYaku {
    pub fn is_ryanpeikou_hand(
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> bool {
        // ryanpeikou must be closed
        if !hand_info.hand_state.is_closed() {
            return false;
        }
        let total_groups = get_total_groups_after_winning_tile(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        );
        let mut tile_ids_sets: HashSet<Vec<MahjongTileId>> = HashSet::new();
        let mut matching_sequences: HashSet<Vec<MahjongTileId>> = HashSet::new();
        for tile_group in total_groups {
            if tile_group.meld_type != MeldType::Sequence {
                continue;
            }
            let tile_group_tile_ids = tile_group.tile_ids;
            if tile_ids_sets.contains(&tile_group_tile_ids) {
                matching_sequences.insert(tile_group_tile_ids);
                if matching_sequences.len() == 2 {
                    println!(
                        "3 han from ryanpeikou (tiles = {})",
                        matching_sequences
                            .iter()
                            .map(|tile_ids| tile_ids_to_string(tile_ids))
                            .collect::<Vec<String>>()
                            .join(" and ")
                    );
                    return true;
                }
            } else {
                tile_ids_sets.insert(tile_group_tile_ids);
            }
        }
        false
    }
}

pub struct SanshokuDoukouYaku;
impl Yaku for SanshokuDoukouYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        let total_groups = get_total_groups_after_winning_tile(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        );
        let mut triplet_rank_to_tile_suits: HashMap<u8, HashSet<MahjongTileNumberedSuit>> =
            HashMap::new();
        for tile_group in total_groups {
            if tile_group.meld_type != MeldType::Triplet
                && tile_group.meld_type != MeldType::Quadruplet
            {
                continue;
            }
            if tile_group.tile_ids.is_empty() {
                continue;
            }
            let triplet_tile = tile_group.tile_ids.get(0).unwrap();
            let triplet_tile_rank = triplet_tile.get_num_tile_rank();
            if triplet_tile_rank.is_none() {
                // can't get sanshoku doukou on a triplet for a non-numbered-suit tile
                continue;
            }
            let triplet_tile_rank = triplet_tile_rank.unwrap();
            let sequence_suit = triplet_tile.get_num_tile_suit().unwrap();
            let entry = triplet_rank_to_tile_suits.entry(triplet_tile_rank);
            let set_was_updated = entry.or_default().insert(sequence_suit);

            if set_was_updated {
                // check if we now have sanshoku doukou
                // println!("found triplet for rank {triplet_tile_rank}, suit = {sequence_suit:?}");
                let value = triplet_rank_to_tile_suits.get(&triplet_tile_rank);
                if value.is_some() && value.unwrap().len() == 3 {
                    // sanshoku doukou confirmed
                    println!("2 han from sanshoku doukou");
                    return 2;
                }
            }
        }
        0
    }
}

pub struct SanshokuDoujunYaku;
impl Yaku for SanshokuDoujunYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        let total_groups = get_total_groups_after_winning_tile(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        );
        let mut min_sequence_rank_to_tile_suits: HashMap<u8, HashSet<MahjongTileNumberedSuit>> =
            HashMap::new();
        for tile_group in total_groups {
            if tile_group.meld_type != MeldType::Sequence {
                continue;
            }
            if tile_group.tile_ids.is_empty() {
                continue;
            }
            let min_tile_num_rank = tile_group
                .tile_ids
                .iter()
                .map(|tile_id| tile_id.get_num_tile_rank().unwrap())
                .min()
                .unwrap();
            let sequence_suit = tile_group
                .tile_ids
                .get(0)
                .unwrap()
                .get_num_tile_suit()
                .unwrap();
            let entry = min_sequence_rank_to_tile_suits.entry(min_tile_num_rank);
            let set_was_updated = entry.or_default().insert(sequence_suit);

            if set_was_updated {
                // check if we now have sanshoku
                // println!("found sequence that starts at rank {min_tile_num_rank}, suit = {sequence_suit:?}");
                let value = min_sequence_rank_to_tile_suits.get(&min_tile_num_rank);
                if value.is_some() && value.unwrap().len() == 3 {
                    // sanshoku confirmed
                    if hand_info.hand_state.is_closed() {
                        println!("2 han from sanshoku doujun (closed)");
                        return 2;
                    } else {
                        println!("1 han from sanshoku doujun (open)");
                        return 1;
                    }
                }
            }
        }
        0
    }
}

pub struct IttsuYaku;
impl Yaku for IttsuYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        let total_groups = get_total_groups_after_winning_tile(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        );
        let mut tile_suit_to_ittsu_sequence_ranks: HashMap<MahjongTileNumberedSuit, HashSet<u8>> =
            HashMap::new();
        for tile_group in total_groups {
            if tile_group.meld_type != MeldType::Sequence {
                continue;
            }
            if tile_group.tile_ids.is_empty() {
                continue;
            }
            let min_tile_num_rank = tile_group
                .tile_ids
                .iter()
                .map(|tile_id| tile_id.get_num_tile_rank().unwrap())
                .min()
                .unwrap();
            let sequence_suit = tile_group
                .tile_ids
                .get(0)
                .unwrap()
                .get_num_tile_suit()
                .unwrap();
            let entry = tile_suit_to_ittsu_sequence_ranks.entry(sequence_suit);
            if min_tile_num_rank == 1 || min_tile_num_rank == 4 || min_tile_num_rank == 7 {
                let set_was_updated = entry.or_default().insert(min_tile_num_rank);

                if set_was_updated {
                    // check if we now have ittsu
                    // println!("found sequence in suit = {sequence_suit:?}, min tile rank = {min_tile_num_rank}");
                    let value = tile_suit_to_ittsu_sequence_ranks.get(&sequence_suit);
                    if value.is_some() && value.unwrap().len() == 3 {
                        // ittsu confirmed
                        if hand_info.hand_state.is_closed() {
                            println!("2 han from ittsu (closed)");
                            return 2;
                        } else {
                            println!("1 han from ittsu (open)");
                            return 1;
                        }
                    }
                }
            }
        }
        0
    }
}

pub struct DoubleRiichiYaku;
impl Yaku for DoubleRiichiYaku {
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
                RiichiInfo::Riichi {
                    is_double_riichi, ..
                } if *is_double_riichi => {
                    println!("2 han from double riichi (riichi on first discard and before any tile calls; can't be combined with riichi)");
                    2
                }
                _ => 0,
            },
            _ => 0,
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
            println!("1 han from rinshan (win by kan replacement draw)");
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
        println!("1 han from tanyao");
        1
    }
}

pub struct ChantaYaku;
impl Yaku for ChantaYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // honroutou implies chanta (not allowed to count chanta and honroutou)
        if HonroutouYaku::is_honroutou_hand(hand_interpretation, winning_tile) {
            return 0;
        }
        // junchan implies chanta (not allowed to count chanta and junchan)
        if JunchanYaku::is_junchan_hand(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        ) {
            return 0;
        }

        // every tile group (including the pair) contains a terminal or honor tile
        let total_groups = get_total_groups_after_winning_tile(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        );
        for tile_meld in total_groups {
            let mut num_terminal_or_honor_tiles = 0;
            for tile_in_meld in tile_meld.tile_ids.iter() {
                if tile_in_meld.is_terminal_tile() || tile_in_meld.is_honor_tile() {
                    num_terminal_or_honor_tiles += 1;
                    break;
                }
            }
            if num_terminal_or_honor_tiles == 0 {
                // if any group has no terminal or honor tiles, it's not chanta
                // println!("tile group doesn't have any terminal or honor tiles = {tile_meld:?}");
                return 0;
            }
        }
        if hand_info.hand_state.is_closed() {
            println!("2 han from chanta (closed)");
            2
        } else {
            println!("2 han from chanta (open)");
            1
        }
    }
}

pub struct JunchanYaku;
impl Yaku for JunchanYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        if JunchanYaku::is_junchan_hand(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        ) {
            if hand_info.hand_state.is_closed() {
                println!("3 han from junchan (closed)");
                3
            } else {
                println!("2 han from junchan (open)");
                2
            }
        } else {
            0
        }
    }
}

impl JunchanYaku {
    pub fn is_junchan_hand(
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        winning_tile_info: &WinningTileInfo,
    ) -> bool {
        // every tile group (including the pair) contains a terminal or honor tile
        let total_groups = get_total_groups_after_winning_tile(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        );
        for tile_meld in total_groups {
            let mut num_terminal_tiles = 0;
            for tile_in_meld in tile_meld.tile_ids.iter() {
                if tile_in_meld.is_terminal_tile() {
                    num_terminal_tiles += 1;
                    break;
                }
            }
            if num_terminal_tiles == 0 {
                // if any group has no terminal tiles, it's not junchan
                // println!("tile group doesn't have any terminal tiles = {tile_meld:?}");
                return false;
            }
        }
        true
    }
}

pub struct HonroutouYaku;
impl Yaku for HonroutouYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        _melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // TODO chinroutou (yakuman) implies honroutou
        // every tile is a terminal or honor (effectively 4 han because it can only be achieved with chiitoi or toitoi)
        if HonroutouYaku::is_honroutou_hand(hand_interpretation, winning_tile) {
            println!("2 han from honroutou (every tile is terminal or honor)");
            2
        } else {
            0
        }
    }
}

impl HonroutouYaku {
    pub fn is_honroutou_hand(
        hand_interpretation: &HandInterpretation,
        winning_tile: MahjongTileId,
    ) -> bool {
        let hand_tile_ids = hand_interpretation
            .total_tile_count_array
            .to_distinct_tile_ids();
        for tile_id in hand_tile_ids {
            if !tile_id.is_terminal_tile() && !tile_id.is_honor_tile() {
                return false;
            }
        }
        winning_tile.is_terminal_tile() || winning_tile.is_honor_tile()
    }
}

pub struct ChiitoiYaku;
impl Yaku for ChiitoiYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        _melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // optimization: chiitoi cannot be formed if the hand has any open calls or a closed kan, that might be faster than counting the pairs
        if ChiitoiYaku::is_chiitoi_hand(hand_interpretation, winning_tile) {
            println!("2 han from chiitoi hand: {}", hand_interpretation);
            2
        } else {
            0
        }
    }
}

impl ChiitoiYaku {
    pub fn is_chiitoi_hand(
        hand_interpretation: &HandInterpretation,
        winning_tile: MahjongTileId,
    ) -> bool {
        // after combining the winning tile and the existing hand tiles, there must be 7 pairs (quads don't count)
        let mut num_pairs = 0;
        for tile_group in hand_interpretation.groups.iter() {
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
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // TODO some yakuman imply toitoi: suuankou and chinroutou
        if ToitoiYaku::is_toitoi_hand(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        ) {
            println!("2 han from toitoi");
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
        winning_tile_info: &WinningTileInfo,
    ) -> bool {
        // every group must be a triplet or quad
        let mut num_triplet_or_quad = 0;
        let total_groups = get_total_groups_after_winning_tile(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        );
        for tile_group in total_groups.iter() {
            if tile_group.meld_type == MeldType::Triplet
                || tile_group.meld_type == MeldType::Quadruplet
            {
                num_triplet_or_quad += 1;
            }
        }
        num_triplet_or_quad == 4
    }
}

pub struct SanankouYaku;
impl Yaku for SanankouYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // TODO suuankou implies sanankou

        // count closed triplets
        let mut num_closed_triplets = 0;
        let total_groups = get_total_groups_after_winning_tile(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        );
        for tile_group in total_groups.iter() {
            if (tile_group.meld_type == MeldType::Triplet
                || tile_group.meld_type == MeldType::Quadruplet)
                && tile_group.is_closed
            {
                // println!("found closed triplet {}", *tile_group);
                num_closed_triplets += 1;
            }
        }
        if num_closed_triplets == 3 {
            println!("2 han from sanankou");
            2
        } else {
            0
        }
    }
}

pub struct HonitsuYaku;
impl Yaku for HonitsuYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        if ChinitsuYaku::is_chinitsu_hand(hand_interpretation, melded_tiles, winning_tile) {
            println!("hand is chinitsu, which implies honitsu");
            0 // chinitsu includes honitsu, so you can't count both
        } else if HonitsuYaku::is_honitsu_hand(hand_interpretation, melded_tiles, winning_tile) {
            if hand_info.hand_state.is_closed() {
                println!("3 han from honitsu (closed)");
                3
            } else {
                println!("2 han from honitsu (open)");
                2
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
        hand_info: &HandInfo,
        _winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        if ChinitsuYaku::is_chinitsu_hand(hand_interpretation, melded_tiles, winning_tile) {
            if hand_info.hand_state.is_closed() {
                println!("6 han from chinitsu (closed)");
                6
            } else {
                println!("5 han from chinitsu (open)");
                5
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

// TODO test sankantsu
pub struct SankantsuYaku;
impl Yaku for SankantsuYaku {
    fn han_value(
        &self,
        hand_interpretation: &HandInterpretation,
        melded_tiles: &Vec<TileMeld>,
        winning_tile: MahjongTileId,
        _hand_info: &HandInfo,
        winning_tile_info: &WinningTileInfo,
    ) -> u8 {
        // count closed triplets
        let mut num_quads = 0;
        let total_groups = get_total_groups_after_winning_tile(
            hand_interpretation,
            melded_tiles,
            winning_tile,
            winning_tile_info,
        );
        for tile_group in total_groups.iter() {
            if tile_group.meld_type == MeldType::Quadruplet {
                // println!("found quadruplet {}", *tile_group);
                num_quads += 1;
            }
        }
        for tile_group in melded_tiles {
            if tile_group.meld_type == MeldType::Quadruplet {
                // println!("found quadruplet {}", *tile_group);
                num_quads += 1;
            }
        }
        if num_quads == 3 {
            println!("2 han from sankantsu");
            2
            // 4 quads is suukantsu (yakuman)
        } else {
            0
        }
    }
}

fn get_interpretations_with_valid_tenpai(
    hand_tiles: MahjongTileCountArray,
    melded_tiles: &Vec<TileMeld>,
    winning_tile: MahjongTileId,
) -> Vec<HandInterpretation> {
    let interpretations = get_hand_interpretations_min_shanten(hand_tiles, melded_tiles, 0);
    let mut valid_interpretations = Vec::new();
    for interpretation in interpretations {
        let total_tile_count_array = interpretation.total_tile_count_array;
        let tile_count_array_with_winning_tile =
            total_tile_count_array.add_tile_ids(vec![winning_tile]);

        let total_shanten = get_shanten_optimized(tile_count_array_with_winning_tile, melded_tiles);
        if total_shanten == -1 {
            // check if the winning tile completes the last incomplete group
            let mut num_complete_groups = 0;
            let mut num_incomplete_groups_besides_pairs = 0;
            let mut num_pairs = 0;
            let mut num_incomplete_groups_completed_by_winning_tile = 0;
            let mut num_pairs_completed_by_winning_tile = 0;
            for tile_meld in interpretation.groups.iter() {
                if tile_meld.meld_type == MeldType::Pair {
                    num_pairs += 1;
                    if tile_meld
                        .tile_ids_to_complete_group()
                        .contains(&winning_tile)
                    {
                        println!(
                            "winning tile {} completes incomplete pair {}",
                            winning_tile, tile_meld
                        );
                        num_pairs_completed_by_winning_tile += 1;
                    }
                } else if !tile_meld.is_complete() {
                    num_incomplete_groups_besides_pairs += 1;
                    if tile_meld
                        .tile_ids_to_complete_group()
                        .contains(&winning_tile)
                    {
                        println!(
                            "winning tile {} completes incomplete group {}",
                            winning_tile, tile_meld
                        );
                        num_incomplete_groups_completed_by_winning_tile += 1;
                    }
                } else if tile_meld.is_complete() {
                    num_complete_groups += 1;
                }
            }

            if num_incomplete_groups_besides_pairs == 1
                && num_pairs == 1
                && num_incomplete_groups_completed_by_winning_tile == 1
                && num_pairs_completed_by_winning_tile == 0
                && num_complete_groups == 3
            {
                println!("valid interpretation: {}", interpretation);
                valid_interpretations.push(interpretation);
            } else if num_incomplete_groups_besides_pairs == 0
                && num_pairs == 2
                && num_incomplete_groups_completed_by_winning_tile == 0
                && num_pairs_completed_by_winning_tile == 1
                && num_complete_groups == 3
            {
                println!("valid interpretation: {}", interpretation);
                valid_interpretations.push(interpretation);
            } else if num_incomplete_groups_besides_pairs == 1
                && num_pairs == 0
                && num_incomplete_groups_completed_by_winning_tile == 1
                && num_complete_groups == 4
            {
                println!("valid interpretation: {}", interpretation);
                valid_interpretations.push(interpretation);
            } else {
                println!("invalid interpretation: {}", interpretation);
            }
        }
    }

    // check chiitoi and kokushi separately (as the `get_hand_interpretations_min_shanten` doesn't include chiitoi (seven pairs) or kokushi (thirteen orphans))
    let total_tile_count_array = hand_tiles.clone().add_tile_ids(vec![winning_tile]);
    let chiitoi_shanten = get_chiitoi_shanten(total_tile_count_array, melded_tiles);
    if chiitoi_shanten == -1 {
        let mut chiitoi_groups = Vec::new();
        for tile_id in total_tile_count_array.to_distinct_tile_ids() {
            chiitoi_groups.push(TileMeld {
                meld_type: MeldType::Pair,
                tile_ids: vec![tile_id, tile_id],
                is_closed: true,
            });
        }
        valid_interpretations.push(HandInterpretation {
            total_tile_count_array: total_tile_count_array,
            groups: chiitoi_groups,
        });
    }
    let kokushi_shanten = get_kokushi_shanten(total_tile_count_array, melded_tiles);
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
        valid_interpretations.push(HandInterpretation {
            total_tile_count_array: total_tile_count_array,
            groups: kokushi_groups,
        });
    }
    valid_interpretations
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
    let valid_interpretations =
        get_interpretations_with_valid_tenpai(hand_tiles, &melded_tiles, winning_tile);
    // TODO validate with tests: this function should return hand interpretations without the winning tile

    let mut max_scoring_han_fu = (0u8, 0u8);
    for interpretation in valid_interpretations {
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

// Box<dyn Yaku> as Yaku is a trait (which doesn't have a defined size), so Box<dyn Yaku> is like a pointer
pub fn get_yaku_list() -> Vec<Box<dyn Yaku>> {
    vec![
        // 1 han (closed only)
        Box::new(MenzenTsumoYaku),
        Box::new(RiichiYaku),
        Box::new(IppatsuYaku),
        Box::new(PinfuYaku),
        Box::new(IipeikouYaku),
        // 1 han
        Box::new(HaiteiYaku),
        Box::new(HouteiYaku),
        Box::new(RinshanYaku),
        Box::new(ChankanYaku),
        Box::new(TanyaoYaku),
        Box::new(YakuhaiYaku),
        // 2 han (closed only)
        Box::new(DoubleRiichiYaku),
        // 2 han (1 han if open)
        Box::new(ChantaYaku),
        Box::new(SanshokuDoujunYaku),
        Box::new(IttsuYaku),
        // 2 han (can be open or closed)
        Box::new(ToitoiYaku),
        Box::new(SanankouYaku),
        Box::new(SanshokuDoukouYaku),
        Box::new(SankantsuYaku),
        Box::new(ChiitoiYaku),
        Box::new(HonroutouYaku),
        Box::new(ShousangenYaku),
        // 3 han (closed only)
        Box::new(RyanpeikouYaku),
        // 3 han (2 han if open)
        Box::new(HonitsuYaku),
        Box::new(JunchanYaku),
        // 6 han (5 han if open)
        Box::new(ChinitsuYaku),
    ]
    // yakuman (should be checked separately)
    // TODO thirteen orphans (yakuman)
    // TODO daisangen (three triplets/quads of dragons)
    // TODO suuankou (four concealed triplets)
    // TODO suukantsu (four quads)
    // TODO shousuushi; daisuushi (three triplets/quads of winds + a pair of the fourth; four triplets/quads of winds)
    // TODO tsuuiisou (all tiles are honor tiles)
    // TODO ryuuiisou (all tiles are all-green: 23468s6z)
    // TODO chinroutou (all tiles are terminals: 1 or 9)
    // TODO chuuren poutou (nine gates: hand contains 1112345678999 + any tile in that same suit)
    // TODO tenhou; chiihou (dealer wins on their first draw; non-dealer wins on their first draw, with no tile calls being made)
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
    if ChiitoiYaku::is_chiitoi_hand(hand_interpretation, winning_tile) {
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
    if ChiitoiYaku::is_chiitoi_hand(hand_interpretation, winning_tile) {
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
    let pair_so_far = match pair_so_far {
        Some(p) => p,
        // if no pair, then there should be a tanki wait
        None => {
            let mut tanki_tile_group = None;
            for tile_group in hand_interpretation.groups.iter() {
                if tile_group.meld_type == MeldType::SingleTile {
                    tanki_tile_group = Some(TileMeld {
                        meld_type: MeldType::Pair,
                        tile_ids: vec![
                            *tile_group.tile_ids.get(0).unwrap(),
                            *tile_group.tile_ids.get(0).unwrap(),
                        ],
                        is_closed: true,
                    });
                }
            }
            tanki_tile_group.expect("there should be a tanki wait if there was no pair initially")
        }
    };
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
            if hand_info.hand_state.is_closed() {
                println!("closed hand win by ron: worth 10 fu");
                fu += 10;
            } else {
                if fu == 20 {
                    // if the hand is open, wins by ron, and has no other fu, then it gains 2 fu
                    println!("open hand win by ron with no other fu: worth 2 fu");
                    fu += 2;
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
        // (this is a "tatsumaki" wait, a 4-sided wait on 12345s - as it's effectively two ryantans: 2223-444 and 222-3444 -> win on 3s is always tanki. Other winning tiles can be ryanmen, and 24s can be shanpon)
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

        // Dealer Tsumo = 11 han 40 fu
        assert_eq!(
            (11, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                tsumo.clone()
            )
        );

        // Dealer Ron = 10 han 50 fu
        assert_eq!(
            (10, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Tsumo = 11 han 40 fu
        assert_eq!(
            (11, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                tsumo.clone()
            )
        );

        // Non-dealer Ron = 10 han 50 fu
        assert_eq!(
            (10, 50),
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

    #[test]
    fn menzentsumo_only_hand() {
        // https://zh.wikipedia.org/wiki/%E9%96%80%E5%89%8D%E6%B8%85%E8%87%AA%E6%91%B8
        let hand = MahjongTileCountArray::from_text("345567m111p3368s");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("1z"); // no dora
        let win_tile = MahjongTileId::from_text("7s").unwrap();

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

        for hand_info in vec![as_dealer, as_nondealer] {
            // Tsumo = 1 han 40 fu
            assert_eq!(
                (1, 40),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = no yaku
            assert_eq!(
                (0, 0),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn mentanpin_sanshoku_iipeikou_hand() {
        // https://ja.wikipedia.org/wiki/%E9%96%80%E5%89%8D%E6%B8%85%E8%87%AA%E6%91%B8%E5%92%8C
        // yaku: sanshoku (345) + iipeikou (334455s) + tanyao + pinfu (+ menzentsumo if tsumo)
        let hand = MahjongTileCountArray::from_text("345m345p3344588s");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("1z"); // no dora
        let win_tile = MahjongTileId::from_text("5s").unwrap();

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

        for hand_info in vec![as_dealer, as_nondealer] {
            // Tsumo = 6 han 20 fu
            assert_eq!(
                (6, 20),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 5 han 30 fu
            assert_eq!(
                (5, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn iipeikou_only_guaranteed_hand() {
        // https://ja.wikipedia.org/wiki/%E4%B8%80%E7%9B%83%E5%8F%A3
        // yaku = iipeikou guaranteed
        let hand = MahjongTileCountArray::from_text("33455m111p33789s");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("1z"); // no dora
        let iipeikou_win_tile = MahjongTileId::from_text("4m").unwrap();

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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 2 han 40 fu
            assert_eq!(
                (2, 40),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    iipeikou_win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 1 han 40 fu
            assert_eq!(
                (1, 40),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    iipeikou_win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn iipeikou_not_guaranteed_hand() {
        // https://ja.wikipedia.org/wiki/%E4%B8%80%E7%9B%83%E5%8F%A3
        // yaku = pinfu guaranteed, but iipeikou only if winning on 5m (+ menzen tsumo if win by tsumo)
        let hand = MahjongTileCountArray::from_text("33445m234p33789s");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("1z"); // no dora
        let iipeikou_win_tile = MahjongTileId::from_text("5m").unwrap();
        let no_iipeikou_win_tile = MahjongTileId::from_text("2m").unwrap();

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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        // win on 5m -> iipeikou
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 3 han 20 fu
            assert_eq!(
                (3, 20),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    iipeikou_win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 2 han 30 fu
            assert_eq!(
                (2, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    iipeikou_win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }

        // win on 2m -> no iipeikou
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 2 han 20 fu
            assert_eq!(
                (2, 20),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    no_iipeikou_win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 1 han 30 fu
            assert_eq!(
                (1, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    no_iipeikou_win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn chiitoi_not_iipeikou() {
        // https://ja.wikipedia.org/wiki/%E4%B8%80%E7%9B%83%E5%8F%A3
        // this is chiitoitsu (seven pairs), not iipeikou
        let hand = MahjongTileCountArray::from_text("12233m3377p2277z");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("1z"); // no dora
        let win_tile = MahjongTileId::from_text("1m").unwrap();

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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 3 han 25 fu
            assert_eq!(
                (3, 25),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 2 han 25 fu
            assert_eq!(
                (2, 25),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn sanankou_outscores_iipeikou() {
        // https://ja.wikipedia.org/wiki/%E4%B8%80%E7%9B%83%E5%8F%A3
        // if interpreting the manzu tiles as 333-444-555 -> scores sanshoku
        // but if interpreting the manzu tiles as 345-345-345 -> scores iipeikou + pinfu
        let hand = MahjongTileCountArray::from_text("333444555m34p33s");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("1z"); // no dora
        let win_tile = MahjongTileId::from_text("2p").unwrap();

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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 4 han 40 fu
            assert_eq!(
                (4, 40),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 3 han 50 fu
            assert_eq!(
                (3, 50),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn sanshoku_doujun_closed() {
        // https://riichi.wiki/Sanshoku_doujun
        let hand = MahjongTileCountArray::from_text("123m123p123678s1z");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("2z"); // no dora
        let win_tile = MahjongTileId::from_text("1z").unwrap();

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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 3 han 30 fu
            assert_eq!(
                (3, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 2 han 40 fu
            assert_eq!(
                (2, 40),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn sanshoku_doujun_open() {
        // https://riichi.wiki/Sanshoku_doujun
        let hand = MahjongTileCountArray::from_text("123m123p123s1z");
        let melded_tiles = vec![TileMeld {
            meld_type: MeldType::Sequence,
            tile_ids: get_tile_ids_from_string("678s"),
            is_closed: false,
        }];
        let dora_tiles = get_tile_ids_from_string("2z"); // no dora
        let win_tile = MahjongTileId::from_text("1z").unwrap();

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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 1 han 30 fu
            assert_eq!(
                (1, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 1 han 30 fu
            assert_eq!(
                (1, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn sanshoku_doukou_open_and_guaranteed() {
        // https://riichi.wiki/Sanshoku_doukou
        let hand = MahjongTileCountArray::from_text("333m333s67s33z");
        let melded_tiles = vec![TileMeld {
            meld_type: MeldType::Triplet,
            tile_ids: get_tile_ids_from_string("333p"),
            is_closed: false,
        }];
        let dora_tiles = get_tile_ids_from_string("2z"); // no dora
        let win_tile = MahjongTileId::from_text("8s").unwrap();

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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 2 han 40 fu
            assert_eq!(
                (2, 40),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 2 han 30 fu
            assert_eq!(
                (2, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn sanshoku_doukou_open_not_guaranteed() {
        // https://riichi.wiki/Sanshoku_doukou
        let hand = MahjongTileCountArray::from_text("333m3345666s");
        let melded_tiles = vec![TileMeld {
            meld_type: MeldType::Triplet,
            tile_ids: get_tile_ids_from_string("333p"),
            is_closed: false,
        }];
        let dora_tiles = get_tile_ids_from_string("2z"); // no dora
        let sanshoku_win_tile = MahjongTileId::from_text("3s").unwrap();
        let no_sanshoku_win_tile = MahjongTileId::from_text("6s").unwrap();

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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 3 han 40 fu
            assert_eq!(
                (3, 40),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    sanshoku_win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 3 han 30 fu
            assert_eq!(
                (3, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    sanshoku_win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );

            // but drawing on 6s -> tanyao only
            // Tsumo = 1 han 40 fu
            assert_eq!(
                (1, 40),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    no_sanshoku_win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 1 han 30 fu
            assert_eq!(
                (1, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    no_sanshoku_win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn ittsu_pinfu_closed_guaranteed() {
        // https://riichi.wiki/Ikki_tsuukan
        let hand = MahjongTileCountArray::from_text("123456789m34p55s");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("2z"); // no dora
        let win_tile = MahjongTileId::from_text("2p").unwrap();

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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 4 han 20 fu (ittsu + pinfu + menzen)
            assert_eq!(
                (4, 20),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 3 han 30 fu (ittsu + pinfu)
            assert_eq!(
                (3, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn ittsu_open_guaranteed() {
        // https://riichi.wiki/Ikki_tsuukan
        let hand = MahjongTileCountArray::from_text("222m46s55z");
        let melded_tiles = vec![
            TileMeld {
                meld_type: MeldType::Sequence,
                tile_ids: get_tile_ids_from_string("123s"),
                is_closed: false,
            },
            TileMeld {
                meld_type: MeldType::Sequence,
                tile_ids: get_tile_ids_from_string("789s"),
                is_closed: false,
            },
        ];
        let dora_tiles = get_tile_ids_from_string("2z"); // no dora
        let win_tile = MahjongTileId::from_text("5s").unwrap();

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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 1 han 30 fu (ittsu only)
            assert_eq!(
                (1, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 1 han 30 fu (ittsu only)
            assert_eq!(
                (1, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn not_ittsu() {
        // https://riichi.wiki/Ikki_tsuukan
        let hand = MahjongTileCountArray::from_text("12344m56p");
        let melded_tiles = vec![
            TileMeld {
                meld_type: MeldType::Sequence,
                tile_ids: get_tile_ids_from_string("567m"),
                is_closed: false,
            },
            TileMeld {
                meld_type: MeldType::Sequence,
                tile_ids: get_tile_ids_from_string("789m"),
                is_closed: false,
            },
        ];
        let dora_tiles = get_tile_ids_from_string("2z"); // no dora
        let win_tile = MahjongTileId::from_text("4p").unwrap();

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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // no yaku (hand is open)
            assert_eq!(
                (0, 0),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            assert_eq!(
                (0, 0),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn haitei_or_houtei_hand() {
        // from https://riichi.wiki/Haitei_raoyue_and_houtei_raoyui, replay in tenpai on haitei draw: http://tenhou.net/0/?log=2013122508gm-0009-7447-3e04d9d5&tw=3
        // win on 6s -> haitei+menzen or houtei, tanyao, pinfu, dora 1, sanshoku = 7 han on haitei+tsumo, 6 han otherwise
        let hand = MahjongTileCountArray::from_text("234678m55678p78s");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("7m");
        let win_tile = MahjongTileId::from_text("6s").unwrap();

        // winning methods (ron / houtei vs tsumo / haitei)
        let ron = WinningTileInfo {
            source: WinningTileSource::Discard {
                is_last_discard: true,
            },
        };
        let tsumo = WinningTileInfo {
            source: WinningTileSource::SelfDraw {
                is_first_draw: false,
                is_last_draw: true,
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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 7 han 20 fu
            assert_eq!(
                (7, 20),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 6 han 30 fu
            assert_eq!(
                (6, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn haitei_or_houtei_only_hand() {
        // synthetic example: no yaku (open hand without tanyao, yakuhai, honitsu/chinitsu, toitoi, sanshoku, ittsu)
        // 123666m22p78s + 456p
        let hand = MahjongTileCountArray::from_text("123666m22p78s");
        let melded_tiles = vec![TileMeld {
            meld_type: MeldType::Sequence,
            tile_ids: get_tile_ids_from_string("456p"),
            is_closed: false,
        }];
        let dora_tiles = get_tile_ids_from_string("2z"); // no dora
        let win_tile = MahjongTileId::from_text("9s").unwrap();

        // winning methods (ron vs tsumo)
        let ron = WinningTileInfo {
            source: WinningTileSource::Discard {
                is_last_discard: true,
            },
        };
        let tsumo = WinningTileInfo {
            source: WinningTileSource::SelfDraw {
                is_first_draw: false,
                is_last_draw: true,
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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // Tsumo = 1 han 30 fu (haitei only)
            assert_eq!(
                (1, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    tsumo.clone()
                )
            );

            // Ron = 1 han 30 fu (houtei only)
            assert_eq!(
                (1, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn chankan_hand_example() {
        // from https://riichi.wiki/Chankan, replay: http://tenhou.net/0/?log=2012112813gm-0009-7447-af4e435f&tw=2&ts=8
        // win on 1p -> riichi, ippatsu, chankan, pinfu, dora 2 (no uradora) = 6 han on chankan
        let hand = MahjongTileCountArray::from_text("345m23345678p77s");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("7s5s"); // 5s is uradora
        let win_tile = MahjongTileId::from_text("1p").unwrap();

        // winning methods (ron on chankan)
        let ron = WinningTileInfo {
            source: WinningTileSource::RobbingKan,
        };

        // situations (dealer in East-1 vs. south in East-1)
        let as_dealer = HandInfo {
            hand_state: HandState::Closed {
                riichi_info: RiichiInfo::Riichi {
                    is_ippatsu: true,
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

        let hand_info_choices = vec![as_dealer, as_nondealer];
        for hand_info in hand_info_choices.iter() {
            // Ron = 6 han 30 fu
            assert_eq!(
                (6, 30),
                compute_han_and_fu(
                    hand.clone(),
                    melded_tiles.clone(),
                    win_tile,
                    hand_info.clone(),
                    ron.clone()
                )
            );
        }
    }

    #[test]
    fn honroutou_toitoi_hand_example() {
        // from https://riichi.wiki/Honroutou
        // win on 1z -> yakuhai (potentially double yakuhai if dealer in east round), honroutou, toitoi (and sanankou if tsumo)
        // this hand is scored incorrectly by https://mahjongo.com/tools/riichi-calculator: it scores it as "Half Outside Hand" for 1 han, instead of honroutou for 2 han
        let hand = MahjongTileCountArray::from_text("111p999s11z11m");
        let melded_tiles = vec![TileMeld {
            meld_type: MeldType::Triplet,
            tile_ids: get_tile_ids_from_string("222z"),
            is_closed: false,
        }];
        let dora_tiles = get_tile_ids_from_string("3z"); // no dora
        let win_tile = MahjongTileId::from_text("1z").unwrap();

        // winning method
        let ron = WinningTileInfo {
            source: WinningTileSource::Discard {
                is_last_discard: false,
            },
        };
        let tsumo: WinningTileInfo = WinningTileInfo {
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
        as_nondealer.seat_wind = MahjongWindOrder::West;
        let as_nondealer = as_nondealer;

        // Dealer Ron = 6 han 50 fu (double yakuhai)
        assert_eq!(
            (6, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Dealer Tsumo = 8 han 50 fu (double yakuhai, also sanankou)
        assert_eq!(
            (8, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                tsumo.clone()
            )
        );

        // Non-dealer Ron = 5 han 50 fu (yakuhai)
        assert_eq!(
            (5, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Tsumo = 7 han 50 fu (yakuhai, also sanankou)
        assert_eq!(
            (7, 50),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_nondealer.clone(),
                tsumo.clone()
            )
        );
    }

    #[test]
    fn honroutou_chiitoi_hand_example() {
        // from https://riichi.wiki/Honroutou
        // win on 4z -> honroutou, chiitoi
        // this hand is scored incorrectly by https://mahjongo.com/tools/riichi-calculator: it scores it as chanta ("Half Outside Hand") for 1 han, instead of chinroutou for 2 han
        let hand = MahjongTileCountArray::from_text("99m11p1199s22334z");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("1z"); // no dora
        let win_tile = MahjongTileId::from_text("4z").unwrap();

        // winning method
        let ron = WinningTileInfo {
            source: WinningTileSource::Discard {
                is_last_discard: false,
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
        as_nondealer.seat_wind = MahjongWindOrder::West;
        let as_nondealer = as_nondealer;

        // Dealer Ron = 4 han 25 fu
        assert_eq!(
            (4, 25),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Ron = 4 han 25 fu
        assert_eq!(
            (4, 25),
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
    fn chanta_hand_example() {
        // from https://riichi.wiki/Chanta
        // win on 1m -> chanta only (the triplet of 9s blocks pinfu)
        // win on 4m -> no yaku
        let hand = MahjongTileCountArray::from_text("123p999s44z23789m");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("1z"); // no dora
        let chanta_win_tile = MahjongTileId::from_text("1m").unwrap();
        let no_chanta_win_tile = MahjongTileId::from_text("4m").unwrap();

        // winning method
        let ron = WinningTileInfo {
            source: WinningTileSource::Discard {
                is_last_discard: false,
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
        as_nondealer.seat_wind = MahjongWindOrder::West;
        let as_nondealer = as_nondealer;

        // Dealer Ron (on 1m) = 2 han 40 fu
        assert_eq!(
            (2, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                chanta_win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Ron (on 1m) = 2 han 40 fu
        assert_eq!(
            (2, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                chanta_win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );

        // Dealer Ron (on 4m) = no yakue
        assert_eq!(
            (0, 0),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                no_chanta_win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Ron (on 4m) = no yakue
        assert_eq!(
            (0, 0),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                no_chanta_win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }

    #[test]
    fn shousangen_hand_example() {
        // from https://riichi.wiki/Shousangen
        // shousangen + yakuhai 2 = 4 han, 8 fu (closed 6z) + 4 fu (open 5z) + 2 fu (yakuhai pair)
        let hand = MahjongTileCountArray::from_text("23m345p77z666z");
        let melded_tiles = vec![TileMeld {
            meld_type: MeldType::Triplet,
            tile_ids: get_tile_ids_from_string("555z"),
            is_closed: false,
        }];
        let dora_tiles = get_tile_ids_from_string("1z"); // no dora
        let win_tile = MahjongTileId::from_text("1m").unwrap();

        // winning method
        let ron = WinningTileInfo {
            source: WinningTileSource::Discard {
                is_last_discard: false,
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
        as_nondealer.seat_wind = MahjongWindOrder::West;
        let as_nondealer = as_nondealer;

        // Dealer Ron = 4 han 40 fu
        assert_eq!(
            (4, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Ron = 4 han 40 fu
        assert_eq!(
            (4, 40),
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
    fn ryanpeikou_hand_example() {
        // from https://riichi.wiki/Ryanpeikou
        // ryanpeikou + pinfu
        let hand = MahjongTileCountArray::from_text("223344p66778m11s");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("1z"); // no dora
        let ryanpeikou_win_tile = MahjongTileId::from_text("8m").unwrap();

        // winning method
        let ron = WinningTileInfo {
            source: WinningTileSource::Discard {
                is_last_discard: false,
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
        as_nondealer.seat_wind = MahjongWindOrder::West;
        let as_nondealer = as_nondealer;

        // Dealer Ron = 4 han 30 fu
        assert_eq!(
            (4, 30),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                ryanpeikou_win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Ron = 4 han 30 fu
        assert_eq!(
            (4, 30),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                ryanpeikou_win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }

    #[test]
    fn junchan_hand_example() {
        // from https://riichi.wiki/Junchantaiyaochuu
        // win on 9s -> junchan only (the triplet of 9m blocks pinfu), 10 fu for ron by closed hand + 8 fu (closed 9m)
        // win on 6s -> no yaku
        let hand = MahjongTileCountArray::from_text("123999m789p1178s");
        let melded_tiles = Vec::new();
        let dora_tiles = get_tile_ids_from_string("1z"); // no dora
        let junchan_win_tile = MahjongTileId::from_text("9s").unwrap();
        let no_junchan_win_tile = MahjongTileId::from_text("6s").unwrap();

        // winning method
        let ron = WinningTileInfo {
            source: WinningTileSource::Discard {
                is_last_discard: false,
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
        as_nondealer.seat_wind = MahjongWindOrder::West;
        let as_nondealer = as_nondealer;

        // Dealer Ron (on 9s) = 3 han 40 fu
        assert_eq!(
            (3, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                junchan_win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Ron (on 9s) = 3 han 40 fu
        assert_eq!(
            (3, 40),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                junchan_win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );

        // Dealer Ron (on 6s) = no yaku
        assert_eq!(
            (0, 0),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                no_junchan_win_tile,
                as_dealer.clone(),
                ron.clone()
            )
        );

        // Non-dealer Ron (on 6s) = no yaku
        assert_eq!(
            (0, 0),
            compute_han_and_fu(
                hand.clone(),
                melded_tiles.clone(),
                no_junchan_win_tile,
                as_nondealer.clone(),
                ron.clone()
            )
        );
    }
}
