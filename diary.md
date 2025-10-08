# Diary

### up next

- make sure we have fully tested the tenpai check, as well as determining which tiles are the winning tiles, the potential groupings, and the shanten + upgrade calculations

### Oct 6 2025

Thinking about collections of mahjong tiles. What we usually want is a multi-set (neither a `Vec` / list, where order matters, nor a set, which doesn't allow duplicates).

The main representations are: `MahjongTileCountArray`, MSPZ string, and a `Vec<MahjongTileId>` (or `Vec<MahjongTile>`).

```
tile count array functions to add: (will need to refactor existing usages)
[x] MahjongTileCountArray::get_tile_id_count(MahjongTileId) -> u8

conversion functions to add:
[x] MahjongTileCountArray::from_tile_ids(Vec<MahjongTileId>) -> MahjongTileCountArray
[x] MahjongTileCountArray::from_text(String) -> MahjongTileCountArray
    * (this already existed: shanten::tiles_to_count_array -> remove that function and rename the usages of that function to MahjongTileCountArray::from_text)
[x] MahjongTileCountArray::to_text() -> String
    * (this already existed: mahjong_hand::tile_count_array_to_string -> remove that function and rename the usages of that function to MahjongTileCountArray::to_text)
[x] tile_ids_to_string(&Vec<MahjongTileId>) -> String
    * (this already existed in shanten::tile_ids_to_string -> remove that function and rename the usages of that function to mahjong_tile::tile_ids_to_string)

functions that exist already:
get_tile_ids_from_string(String) -> Vec<MahjongTileId>
get_tiles_from_string(String) -> Vec<MahjongTile>

rename:
[x] get_tile_ids_from_count_array(MahjongTileCountArray) -> Vec<MahjongTileId> --> rename to MahjongTileCountArray::to_tile_ids -> Vec<MahjongTileId> (this already existed in shanten::tile_count_array_to_tile_ids)
[x] get_distinct_tile_ids_from_count_array(MahjongTileCountArray) -> Vec<MahjongTileId> --> rename to MahjongTileCountArray::to_distinct_tile_ids -> Vec<MahjongTileId>
[x] get_total_tiles_from_count_array(MahjongTileCountArray) -> usize --> rename to MahjongTileCountArray::total_tiles() -> usize
```

### Oct 1 2025

Updated table of the various types used to represent a single tile:

```
v from  \\  to > | MahjongTileId/u8         | MahjongTile            | String
MahjongTileId/u8 | MahjongTileId::new(u8)   | MahjongTile::from      | MahjongTileId::to_text
MahjongTile      | MahjongTile.value        | n/a                    | MahjongTile::to_text
String           | MahjongTileId::from_text | MahjongTile::from_text | n/a
```

There is also `mahjong_tile::get_tile_text_from_u8` (from `u8` to `String`) function for additional conversion (`MahjongTileId::from_text` replaced `mahjong_tile::get_id_from_tile_text`).

checking on results of refactors: https://github.com/alexander-jiang/riichi-bot/compare/f2304150767924c29f0bc620159f3e0c0d18950a...f7ca8fc27f45cc937828c1776795fd9ac64cb719

### Oct 1 2025

table of the various types used to represent a single tile:

```
v from  \\  to > | MahjongTileValue          | MahjongTileId/u8        | MahjongTile            | String
MahjongTileValue | n/a                       | MahjongTileValue::to_id | MahjongTile { ... }    | MahjongTileValue::to_text
MahjongTileId/u8 | MahjongTileValue::from_id | n/a                     | MahjongTile::from_id   | get_tile_text_from_id & MahjongTileId Display impl
MahjongTile      | MahjongTile.value         | MahjongTile::get_id     | n/a                    | ??
String           | ??                        | get_id_from_tile_text   | MahjongTile::from_text | n/a
```

note that `u8` and `MahjongTileId` are interchangeable (1-to-1).

### Sep 29 2025

- (done) doing lots of refactoring: replacing `u8` with `MahjongTileId` where applicable, replacing the hardcoded/magic number 34 with `mahjong_tile::NUM_DISTINCT_TILE_VALUES`
- (done) next up: replace `[u8; 34]` with a custom type
- (done) fix up `shanten::tiles_to_tile_ids` to be consistent and accept shorthand MSPZ notation e.g. "123m" instead of just "1m2m3m"

### Sep 26 2025

- Fixed bug with `mahjong_hand::MahjongHand::get_tenpai_tiles_build_shapes` function falsely saying that completing a pair into a triplet would be a winning tile (even if the hand has 3 complete groups and 1 pair and 1 non-pair incomplete group)
- Fixed bug with tenpai `tile_grouping::get_all_tenpai_wait_tiles` function allowing to be tenpai on a tile when all four copies of that tile are in your hand

up next / TODOs:

- (done) cleanup old / unused code - start from the tile class, then work up to the tile groups
- (done) consolidate the functions for converting between different representations for tiles (e.g. tile "id", which is a u8 from 0 to 33 inclusive, and a string) and for tile sets/lists (e.g. list of tile ids, list of tiles as strings, or a count-array: a 34-length array of counts for each tile type)
- make sure we have fully tested the tenpai check, as well as determining which tiles are the winning tiles, the potential groupings, and the shanten + upgrade calculations

### Jul 7 2025

Fixed the bug with the monte carlo basic analysis: need to include the tiles that are in the starting hand from the remaining tile count array when initializing the pool of tiles to draw from, which means the tenpai rate on the first turn matches the expectation (should be 15/93 = 16.13%).

Re-running the basic analysis (took about 3.74 seconds)

```
initial hand: 3m4m5m1p1p5p6p4s6s6s7s7s8s
initial ukiere tiles: 4p7p5s8s
num tiles visible initially: 43
initially visible tiles: 6s4z2z3z9m9p7z6m2p8p4z1z2z3p3p9s4z5z6z3z1s3z7z4z1z6z5z1s4m9m

summary:
100000 total trials, 89396 reached tenpai in at most 12 draws
average 4.56 turns to reach tenpai (among success)
average 7.47 ukiere tiles at tenpai (choosing discard that maximizes ukiere)
count of number of trials that we reached tenpai: (starting from first draw)
[16172, 13632, 11605, 9804, 8082, 6950, 5737, 4834, 3981, 3382, 2846, 2371]
```

From this [calculator](https://gkmtg.pro/tools/calculator/), the % chance of reaching tenpai in X draws vs. simulation estimate is much closer now:

- calculator % chance of reaching tenpai in 1 draw : 16.13% vs. simulation 16172/100000 = 16.17%
- calculator % chance of reaching tenpai in 2 draws: 29.80% vs. simulation (16172+13632)/100000 = 29.80%
- calculator % chance of reaching tenpai in 3 draws: 41.37% vs. simulation (16172+13632+11605)/100000 = 41.41%
- calculator % chance of reaching tenpai in 12 draws: 89.57% vs. simulation 89396/100000 = 89.40%

### Jun 21 2025

optimized the monte carlo simulation. We know what the possible improvement tiles are, so we pre-compute the resulting (discard_tile, ukiere_tiles_after_discard) for each possible discard, and we can look up how many ukiere tiles are actually remaining at the time we draw the tile.

```
initial hand: 345m1156466778s
initial ukiere tiles: 47p58s (to reach tenpai)
```

The result is ~30x faster: ~0.05 seconds for 1000 trials (instead of 1.58 seconds):

```
$ cargo test monte_carlo_analysis::tests::test_basic_analysis -- --show-output
...
summary:
1000 total trials, 864 reached tenpai in at most 12 draws
average 4.72 turns to reach tenpai (among success)
average 7.53 ukiere tiles at tenpai (choosing discard that maximizes ukiere)
count of number of trials that we reached tenpai: (starting from first draw)
[145, 118, 113, 109, 74, 67, 55, 46, 47, 38, 30, 22]

successes:
    monte_carlo_analysis::tests::test_basic_analysis

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 172 filtered out; finished in 0.05s
```

Running with 100k trials: (takes about 4.11 seconds)

```
$ cargo test monte_carlo_analysis::tests::test_basic_analysis -- --show-output
...
summary:
100000 total trials, 87677 reached tenpai in at most 12 draws
average 4.70 turns to reach tenpai (among success)
average 7.50 ukiere tiles at tenpai (choosing discard that maximizes ukiere)
count of number of trials that we reached tenpai: (starting from first draw)
[15112, 12833, 11089, 9480, 8095, 6863, 5833, 5083, 4144, 3559, 3049, 2537]

successes:
    monte_carlo_analysis::tests::test_basic_analysis

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 172 filtered out; finished in 4.11s
```

From this [calculator](https://gkmtg.pro/tools/calculator/), the % chance of reaching tenpai in X draws vs. simulation estimate:

- calculator % chance of reaching tenpai in 1 draw : 16.13% vs. simulation 15112/100000 = 15.11%
- calculator % chance of reaching tenpai in 2 draws: 29.80% vs. simulation (15112+12833)/100000 = 27.95%
- calculator % chance of reaching tenpai in 3 draws: 41.37% vs. simulation (15112+12833+11089)/100000 = 39.03%

### Jun 20 2025

testing the monte carlo simulation. Starting with hand 345m1156466778s (the situation on turn 8, if we hypothetically discarded 2p). If we just accept ukiere (and not upgrades), we expect that:

- given there are 136 total tiles - 43 total visible tiles = 93 remaining tiles. (13 in hand + 29 discarded tiles + 1 dora indicator tile = 43 visible tiles)
- and the ukiere tiles are 47p58s, of which only 1 is visible (one 8s in hand already), there are 15 "outs"
- naive calculation says we need 93/15 = 6.2 draws to reach tenpai. But is that accurate?
- based on calculator [here](https://gkmtg.pro/tools/calculator/), with 93 in the "deck", 15 "outs" and we only need to draw 1 "out" (to advance to tenpai), after 6 draws, we have a 66.30% chance of reaching tenpai.
- I set the max number of draws to 12, based on the calculator above, we have a 89.57% chance of reaching tenpai within that many draws.

```
initial hand: 345m1156466778s
initial ukiere tiles: 47p58s (to reach tenpai)
trial 100
trial 200
trial 300
trial 400
trial 500
trial 600
trial 700
trial 800
trial 900
trial 1000
summary:
1000 total trials, 878 reached tenpai in at most 12 draws
average 4.76 turns to reach tenpai (among success)
average 7.51 ukiere tiles at tenpai (choosing discard that maximizes ukiere)


(subsequent runs:)
summary:
1000 total trials, 872 reached tenpai in at most 12 draws
average 4.68 turns to reach tenpai (among success)
average 7.53 ukiere tiles at tenpai (choosing discard that maximizes ukiere)

summary:
1000 total trials, 878 reached tenpai in at most 12 draws
average 4.73 turns to reach tenpai (among success)
average 7.51 ukiere tiles at tenpai (choosing discard that maximizes ukiere)
```

Unfortunately, this took about 1.58 seconds, which is quite slow (if we wanted to run 1 million trials, that would take ~26.3 mins).

### Jun 18 2025

revisited the tenhou hand replay, East-1 round, seat: east. I felt I had a pretty good & fast hand but somehow lost in the race to tenpai/riichi to kami (player to my left). I think the real mistake was on the turn before the decision to discard 4s vs. 2p:

```
hand: 345m11256p46778s6m
dora indicator: 6s (dora: 7s)
round: east-1, seat: east, points: 25k points all
turn: 7
discard pools:
self   (east): 4z2z3z9m9p7z
shimo (south): 8p4z1z2z3p3p
toimen (west): 4z5z6z3z1s3z
kami  (north): 4z1z6z5z1s4m

Analysis:
- In game, I chose to cut 6m -- I think my logic was that I wanted to keep the two dora (7s) and I saw the souzu as two groups: 4s-67s-78s. But in hindsight, the souzu can be considered as one group (678s-4s-7s) for the sake of speed, and I don't even have to cut a souzu tile right now - I could cut 2p (note that shimo has discarded two 3p already). I should consider the 6m a lucky draw: it is pretty much the optimal draw for forming a second group in manzu (3456m can accept 12345678m), and ~half of those manzu tiles (2457m) guarantee a ryanmen wait or better, which further increases speed at 1-shanten.

Also, accounting for game state, it's the first hand of a tonpuusen game (east-1, all players at 25k points), and we are dealer, so we should be willing to sacrifice some value (i.e. cut one of the two dora tiles) in exchange for reaching tenpai more quickly (better chance to retain dealership and puts pressure on the non-dealers).



ukiere map after discard: (improving to 1 shanten)
discard 7s: 12345678m12347p23456  9s
discard 2p: 12345678m1  47p23456789s
discard 4s: 12345678m12347p   56789s
discard 3m:          12347p23456789s
discard 6m:          12347p23456789s
discard 1p:   3  6m    347p  45 7s
discard 8s:          1 347p   5 7s


shanten + ukiere after each discard: 345m11256p46778s6m
discard 7s -> 2 shanten, 67 ukiere tiles: 12345678m12347p234569s
  after advancing shanten:
    draw 4p -> cut 2p => 52 ukiere: 1m2m3m4m5m6m7m8m1p2s3s4s5s6s9s
    draw 7p -> cut 2p => 52 ukiere: 1m2m3m4m5m6m7m8m1p2s3s4s5s6s9s
    draw 1p -> cut 4s => 21 ukiere: 3m6m2p3p4p7p
    draw 5s -> cut 3m => 19 ukiere: 4p7p3s6s9s; cut 6m => 19 ukiere: 4p7p3s6s9s; cut 2p => 19 ukiere: 4p7p3s6s9s
    draw 7m -> cut 2p => 19 ukiere: 2m5m8m4p7p; cut 4s => 19 ukiere: 2m5m8m4p7p
    draw 2m -> cut 2p => 19 ukiere: 1m4m7m4p7p; cut 4s => 19 ukiere: 1m4m7m4p7p
    draw 3p -> cut 1p => 17 ukiere: 3m6m4p7p4s
    draw 6m -> cut 4s => 16 ukiere: 6m1p3p4p7p
    draw 4s -> cut 3m => 16 ukiere: 1p3p4p7p4s; cut 6m => 16 ukiere: 1p3p4p7p4s
    draw 3s -> cut 3m => 16 ukiere: 4p7p2s5s; cut 6m => 16 ukiere: 4p7p2s5s; cut 2p => 16 ukiere: 4p7p2s5s
    draw 3m -> cut 4s => 16 ukiere: 3m1p3p4p7p
    draw 5m -> cut 2p => 15 ukiere: 4m7m4p7p; cut 4s => 15 ukiere: 4m7m4p7p
    draw 4m -> cut 2p => 15 ukiere: 2m5m4p7p; cut 4s => 15 ukiere: 2m5m4p7p
    draw 6s -> cut 3m => 12 ukiere: 4p7p5s; cut 6m => 12 ukiere: 4p7p5s; cut 2p => 12 ukiere: 4p7p5s
    draw 9s -> cut 3m => 12 ukiere: 4p7p5s; cut 6m => 12 ukiere: 4p7p5s; cut 2p => 12 ukiere: 4p7p5s
    draw 8m -> cut 2p => 12 ukiere: 7m4p7p; cut 4s => 12 ukiere: 7m4p7p
    draw 2p -> cut 3m => 12 ukiere: 1p2p4p7p; cut 6m => 12 ukiere: 1p2p4p7p; cut 4s => 12 ukiere: 1p2p4p7p
    draw 2s -> cut 3m => 12 ukiere: 4p7p3s; cut 6m => 12 ukiere: 4p7p3s; cut 2p => 12 ukiere: 4p7p3s
    draw 1m -> cut 2p => 12 ukiere: 2m4p7p; cut 4s => 12 ukiere: 2m4p7p
  upgrades:
    draw 8p -> cut 2p => 77 ukiere: 1m2m3m4m5m6m7m8m1p3p4p5p6p7p8p9p2s3s4s5s6s9s
    draw 6p -> cut 2p => 73 ukiere: 1m2m3m4m5m6m7m8m1p3p4p5p6p7p8p2s3s4s5s6s9s
    draw 5p -> cut 2p => 73 ukiere: 1m2m3m4m5m6m7m8m1p3p4p5p6p7p8p2s3s4s5s6s9s; cut 1p => 71 ukiere: 1m2m3m4m5m6m7m8m3p4p5p6p7p8p2s3s4s5s6s9s
discard 2p -> 2 shanten, 65 ukiere tiles: 12345678m147p23456789s
  after advancing shanten:
    draw 4p -> cut 7s => 52 ukiere: 1m2m3m4m5m6m7m8m1p2s3s4s5s6s9s
    draw 7p -> cut 7s => 52 ukiere: 1m2m3m4m5m6m7m8m1p2s3s4s5s6s9s
    draw 5s -> cut 3m => 23 ukiere: 1p4p7p3s6s7s9s; cut 6m => 23 ukiere: 1p4p7p3s6s7s9s
    draw 7m -> cut 4s => 19 ukiere: 2m5m8m4p7p; cut 7s => 19 ukiere: 2m5m8m4p7p
    draw 2m -> cut 4s => 19 ukiere: 1m4m7m4p7p; cut 7s => 19 ukiere: 1m4m7m4p7p
    draw 1p -> cut 3m => 17 ukiere: 4p7p4s5s7s; cut 6m => 17 ukiere: 4p7p4s5s7s; cut 7s => 17 ukiere: 3m6m4p7p4s
    draw 3s -> cut 3m => 16 ukiere: 4p7p2s5s; cut 6m => 16 ukiere: 4p7p2s5s; cut 7s => 16 ukiere: 4p7p2s5s
    draw 6s -> cut 3m => 15 ukiere: 4p7p5s8s; cut 6m => 15 ukiere: 4p7p5s8s; cut 4s => 15 ukiere: 4p7p5s8s
    draw 9s -> cut 3m => 15 ukiere: 4p7p5s8s; cut 6m => 15 ukiere: 4p7p5s8s; cut 4s => 15 ukiere: 4p7p5s8s
    draw 7s -> cut 3m => 15 ukiere: 1p4p7p5s7s; cut 6m => 15 ukiere: 1p4p7p5s7s
    draw 5m -> cut 4s => 15 ukiere: 4m7m4p7p; cut 7s => 15 ukiere: 4m7m4p7p
    draw 4m -> cut 4s => 15 ukiere: 2m5m4p7p; cut 7s => 15 ukiere: 2m5m4p7p
    draw 8s -> cut 3m => 15 ukiere: 4p7p6s9s; cut 6m => 15 ukiere: 4p7p6s9s; cut 4s => 15 ukiere: 4p7p6s9s
    draw 6m -> cut 4s => 12 ukiere: 6m1p4p7p; cut 7s => 12 ukiere: 6m1p4p7p
    draw 8m -> cut 4s => 12 ukiere: 7m4p7p; cut 7s => 12 ukiere: 7m4p7p
    draw 4s -> cut 3m => 12 ukiere: 1p4p7p4s; cut 6m => 12 ukiere: 1p4p7p4s; cut 7s => 12 ukiere: 1p4p7p4s
    draw 2s -> cut 3m => 12 ukiere: 4p7p3s; cut 6m => 12 ukiere: 4p7p3s; cut 7s => 12 ukiere: 4p7p3s
    draw 3m -> cut 4s => 12 ukiere: 3m1p4p7p; cut 7s => 12 ukiere: 3m1p4p7p
    draw 1m -> cut 4s => 12 ukiere: 2m4p7p; cut 7s => 12 ukiere: 2m4p7p
  upgrades:
    draw 8p -> cut 7s => 77 ukiere: 1m2m3m4m5m6m7m8m1p3p4p5p6p7p8p9p2s3s4s5s6s9s; cut 4s => 71 ukiere: 1m2m3m4m5m6m7m8m1p3p4p5p6p7p8p9p5s6s7s8s9s
    draw 3p -> cut 7s => 77 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p5p6p7p8p2s3s4s5s6s9s; cut 4s => 71 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p5p6p7p8p5s6s7s8s9s
    draw 6p -> cut 7s => 73 ukiere: 1m2m3m4m5m6m7m8m1p3p4p5p6p7p8p2s3s4s5s6s9s; cut 4s => 67 ukiere: 1m2m3m4m5m6m7m8m1p3p4p5p6p7p8p5s6s7s8s9s
    draw 5p -> cut 7s => 73 ukiere: 1m2m3m4m5m6m7m8m1p3p4p5p6p7p8p2s3s4s5s6s9s; cut 4s => 67 ukiere: 1m2m3m4m5m6m7m8m1p3p4p5p6p7p8p5s6s7s8s9s
    draw 2p -> cut 7s => 67 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p2s3s4s5s6s9s
    draw 9p -> cut 7s => 67 ukiere: 1m2m3m4m5m6m7m8m1p4p7p8p9p2s3s4s5s6s9s
discard 4s -> 2 shanten, 61 ukiere tiles: 12345678m12347p56789s
  after advancing shanten:
    draw 4p -> cut 2p => 46 ukiere: 1m2m3m4m5m6m7m8m1p5s6s7s8s9s
    draw 7p -> cut 2p => 46 ukiere: 1m2m3m4m5m6m7m8m1p5s6s7s8s9s
    draw 1p -> cut 7s => 21 ukiere: 3m6m2p3p4p7p
    draw 7m -> cut 2p => 19 ukiere: 2m5m8m4p7p; cut 7s => 19 ukiere: 2m5m8m4p7p
    draw 2m -> cut 2p => 19 ukiere: 1m4m7m4p7p; cut 7s => 19 ukiere: 1m4m7m4p7p
    draw 6m -> cut 7s => 16 ukiere: 6m1p3p4p7p
    draw 3p -> cut 1p => 16 ukiere: 3m6m4p7p7s; cut 7s => 16 ukiere: 3m6m1p4p7p
    draw 3m -> cut 7s => 16 ukiere: 3m1p3p4p7p
    draw 7s -> cut 3m => 15 ukiere: 1p3p4p7p7s; cut 6m => 15 ukiere: 1p3p4p7p7s
    draw 5m -> cut 2p => 15 ukiere: 4m7m4p7p; cut 7s => 15 ukiere: 4m7m4p7p
    draw 4m -> cut 2p => 15 ukiere: 2m5m4p7p; cut 7s => 15 ukiere: 2m5m4p7p
    draw 8s -> cut 3m => 15 ukiere: 4p7p6s9s; cut 6m => 15 ukiere: 4p7p6s9s; cut 2p => 15 ukiere: 4p7p6s9s
    draw 9s -> cut 3m => 15 ukiere: 4p7p5s8s; cut 6m => 15 ukiere: 4p7p5s8s; cut 2p => 15 ukiere: 4p7p5s8s
    draw 6s -> cut 3m => 15 ukiere: 4p7p5s8s; cut 6m => 15 ukiere: 4p7p5s8s; cut 2p => 15 ukiere: 4p7p5s8s
    draw 5s -> cut 3m => 15 ukiere: 4p7p6s9s; cut 6m => 15 ukiere: 4p7p6s9s; cut 2p => 15 ukiere: 4p7p6s9s
    draw 8m -> cut 2p => 12 ukiere: 7m4p7p; cut 7s => 12 ukiere: 7m4p7p
    draw 2p -> cut 3m => 12 ukiere: 1p2p4p7p; cut 6m => 12 ukiere: 1p2p4p7p; cut 7s => 12 ukiere: 1p2p4p7p
    draw 1m -> cut 2p => 12 ukiere: 2m4p7p; cut 7s => 12 ukiere: 2m4p7p
  upgrades:
    draw 8p -> cut 2p => 71 ukiere: 1m2m3m4m5m6m7m8m1p3p4p5p6p7p8p9p5s6s7s8s9s
    draw 2s -> cut 2p => 69 ukiere: 1m2m3m4m5m6m7m8m1p4p7p1s2s3s4s5s6s7s8s9s
    draw 3s -> cut 2p => 69 ukiere: 1m2m3m4m5m6m7m8m1p4p7p1s2s3s4s5s6s7s8s9s; cut 7s => 64 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p1s2s3s4s5s
    draw 5p -> cut 2p => 67 ukiere: 1m2m3m4m5m6m7m8m1p3p4p5p6p7p8p5s6s7s8s9s; cut 1p => 65 ukiere: 1m2m3m4m5m6m7m8m3p4p5p6p7p8p5s6s7s8s9s
    draw 4s -> cut 7s => 67 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 65 ukiere: 1m2m3m4m5m6m7m8m1p4p7p2s3s4s5s6s7s8s9s
    draw 6p -> cut 2p => 67 ukiere: 1m2m3m4m5m6m7m8m1p3p4p5p6p7p8p5s6s7s8s9s
    draw 1s -> cut 2p => 65 ukiere: 1m2m3m4m5m6m7m8m1p4p7p1s2s3s5s6s7s8s9s
discard 3m -> 2 shanten, 44 ukiere tiles: 12347p23456789s
  after advancing shanten:
    draw 4p -> cut 7s => 38 ukiere: 1p2p3p4p7p2s3s4s5s6s9s
    draw 7p -> cut 7s => 35 ukiere: 1p2p3p4p2s3s4s5s6s9s
    draw 5s -> cut 2p => 23 ukiere: 1p4p7p3s6s7s9s
    draw 1p -> cut 7s => 18 ukiere: 2p3p4p7p4s
    draw 3p -> cut 1p => 17 ukiere: 4p7p4s5s7s
    draw 4s -> cut 7s => 16 ukiere: 1p3p4p7p4s
    draw 3s -> cut 2p => 16 ukiere: 4p7p2s5s; cut 7s => 16 ukiere: 4p7p2s5s
    draw 6s -> cut 2p => 15 ukiere: 4p7p5s8s; cut 4s => 15 ukiere: 4p7p5s8s
    draw 9s -> cut 2p => 15 ukiere: 4p7p5s8s; cut 4s => 15 ukiere: 4p7p5s8s
    draw 7s -> cut 2p => 15 ukiere: 1p4p7p5s7s; cut 4s => 15 ukiere: 1p3p4p7p7s
    draw 8s -> cut 2p => 15 ukiere: 4p7p6s9s; cut 4s => 15 ukiere: 4p7p6s9s
    draw 2p -> cut 4s => 12 ukiere: 1p2p4p7p; cut 7s => 12 ukiere: 1p2p4p7p
    draw 2s -> cut 2p => 12 ukiere: 4p7p3s; cut 7s => 12 ukiere: 4p7p3s
  upgrades:
    draw 3m -> cut 7s => 67 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 65 ukiere: 1m2m3m4m5m6m7m8m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 61 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p5s6s7s8s9s
    draw 7m -> cut 7s => 67 ukiere: 2m3m4m5m6m7m8m9m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 65 ukiere: 2m3m4m5m6m7m8m9m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 61 ukiere: 2m3m4m5m6m7m8m9m1p2p3p4p7p5s6s7s8s9s
    draw 6m -> cut 7s => 59 ukiere: 3m4m5m6m7m8m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 57 ukiere: 3m4m5m6m7m8m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 53 ukiere: 3m4m5m6m7m8m1p2p3p4p7p5s6s7s8s9s
    draw 4m -> cut 7s => 59 ukiere: 2m3m4m5m6m7m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 57 ukiere: 2m3m4m5m6m7m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 53 ukiere: 2m3m4m5m6m7m1p2p3p4p7p5s6s7s8s9s
    draw 8m -> cut 7s => 57 ukiere: 3m6m7m8m9m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 55 ukiere: 3m6m7m8m9m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 51 ukiere: 3m6m7m8m9m1p2p3p4p7p5s6s7s8s9s
    draw 2m -> cut 7s => 57 ukiere: 1m2m3m4m7m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 55 ukiere: 1m2m3m4m7m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 51 ukiere: 1m2m3m4m7m1p2p3p4p7p5s6s7s8s9s
    draw 5m -> cut 7s => 55 ukiere: 3m4m5m6m7m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 53 ukiere: 3m4m5m6m7m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 49 ukiere: 3m4m5m6m7m1p2p3p4p7p5s6s7s8s9s
    draw 8p -> cut 2p => 54 ukiere: 1p3p4p5p6p7p8p9p2s3s4s5s6s7s8s9s; cut 7s => 52 ukiere: 1p2p3p4p5p6p7p8p9p2s3s4s5s6s9s; cut 4s => 46 ukiere: 1p2p3p4p5p6p7p8p9p5s6s7s8s9s
    draw 1m -> cut 7s => 50 ukiere: 1m2m3m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 48 ukiere: 1m2m3m1p4p7p2s3s4s5s6s7s8s9s
    draw 6p -> cut 2p => 50 ukiere: 1p3p4p5p6p7p8p2s3s4s5s6s7s8s9s; cut 7s => 48 ukiere: 1p2p3p4p5p6p7p8p2s3s4s5s6s9s
    draw 9m -> cut 7s => 50 ukiere: 7m8m9m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 48 ukiere: 7m8m9m1p4p7p2s3s4s5s6s7s8s9s
    draw 5p -> cut 2p => 50 ukiere: 1p3p4p5p6p7p8p2s3s4s5s6s7s8s9s; cut 1p => 48 ukiere: 3p4p5p6p7p8p2s3s4s5s6s7s8s9s; cut 7s => 48 ukiere: 1p2p3p4p5p6p7p8p2s3s4s5s6s9s
    draw 9p -> cut 7s => 46 ukiere: 1p2p3p4p7p8p9p2s3s4s5s6s9s
discard 6m -> 2 shanten, 44 ukiere tiles: 12347p23456789s
  after advancing shanten:
    draw 4p -> cut 7s => 38 ukiere: 1p2p3p4p7p2s3s4s5s6s9s
    draw 7p -> cut 7s => 35 ukiere: 1p2p3p4p2s3s4s5s6s9s
    draw 5s -> cut 2p => 23 ukiere: 1p4p7p3s6s7s9s
    draw 1p -> cut 7s => 18 ukiere: 2p3p4p7p4s
    draw 3p -> cut 1p => 17 ukiere: 4p7p4s5s7s
    draw 4s -> cut 7s => 16 ukiere: 1p3p4p7p4s
    draw 3s -> cut 2p => 16 ukiere: 4p7p2s5s; cut 7s => 16 ukiere: 4p7p2s5s
    draw 6s -> cut 2p => 15 ukiere: 4p7p5s8s; cut 4s => 15 ukiere: 4p7p5s8s
    draw 9s -> cut 2p => 15 ukiere: 4p7p5s8s; cut 4s => 15 ukiere: 4p7p5s8s
    draw 7s -> cut 2p => 15 ukiere: 1p4p7p5s7s; cut 4s => 15 ukiere: 1p3p4p7p7s
    draw 8s -> cut 2p => 15 ukiere: 4p7p6s9s; cut 4s => 15 ukiere: 4p7p6s9s
    draw 2p -> cut 4s => 12 ukiere: 1p2p4p7p; cut 7s => 12 ukiere: 1p2p4p7p
    draw 2s -> cut 2p => 12 ukiere: 4p7p3s; cut 7s => 12 ukiere: 4p7p3s
  upgrades:
    draw 6m -> cut 7s => 67 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 65 ukiere: 1m2m3m4m5m6m7m8m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 61 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p5s6s7s8s9s
    draw 2m -> cut 7s => 63 ukiere: 1m2m3m4m5m6m7m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 61 ukiere: 1m2m3m4m5m6m7m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 57 ukiere: 1m2m3m4m5m6m7m1p2p3p4p7p5s6s7s8s9s
    draw 7m -> cut 7s => 61 ukiere: 2m5m6m7m8m9m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 59 ukiere: 2m5m6m7m8m9m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 55 ukiere: 2m5m6m7m8m9m1p2p3p4p7p5s6s7s8s9s
    draw 3m -> cut 7s => 59 ukiere: 1m2m3m4m5m6m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 57 ukiere: 1m2m3m4m5m6m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 53 ukiere: 1m2m3m4m5m6m1p2p3p4p7p5s6s7s8s9s
    draw 5m -> cut 7s => 59 ukiere: 2m3m4m5m6m7m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 57 ukiere: 2m3m4m5m6m7m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 53 ukiere: 2m3m4m5m6m7m1p2p3p4p7p5s6s7s8s9s
    draw 4m -> cut 7s => 55 ukiere: 2m3m4m5m6m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 53 ukiere: 2m3m4m5m6m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 49 ukiere: 2m3m4m5m6m1p2p3p4p7p5s6s7s8s9s
    draw 8p -> cut 2p => 54 ukiere: 1p3p4p5p6p7p8p9p2s3s4s5s6s7s8s9s; cut 7s => 52 ukiere: 1p2p3p4p5p6p7p8p9p2s3s4s5s6s9s; cut 4s => 46 ukiere: 1p2p3p4p5p6p7p8p9p5s6s7s8s9s
    draw 8m -> cut 7s => 54 ukiere: 6m7m8m9m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 52 ukiere: 6m7m8m9m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 48 ukiere: 6m7m8m9m1p2p3p4p7p5s6s7s8s9s
    draw 1m -> cut 7s => 53 ukiere: 1m2m3m6m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 51 ukiere: 1m2m3m6m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 47 ukiere: 1m2m3m6m1p2p3p4p7p5s6s7s8s9s
    draw 9m -> cut 7s => 50 ukiere: 7m8m9m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 48 ukiere: 7m8m9m1p4p7p2s3s4s5s6s7s8s9s
    draw 6p -> cut 2p => 50 ukiere: 1p3p4p5p6p7p8p2s3s4s5s6s7s8s9s; cut 7s => 48 ukiere: 1p2p3p4p5p6p7p8p2s3s4s5s6s9s
    draw 5p -> cut 2p => 50 ukiere: 1p3p4p5p6p7p8p2s3s4s5s6s7s8s9s; cut 1p => 48 ukiere: 3p4p5p6p7p8p2s3s4s5s6s7s8s9s; cut 7s => 48 ukiere: 1p2p3p4p5p6p7p8p2s3s4s5s6s9s
    draw 9p -> cut 7s => 46 ukiere: 1p2p3p4p7p8p9p2s3s4s5s6s9s
discard 1p -> 2 shanten, 27 ukiere tiles: 36m347p457s
  after advancing shanten:
    draw 3p -> cut 3m => 17 ukiere: 4p7p4s5s7s; cut 6m => 17 ukiere: 4p7p4s5s7s; cut 7s => 17 ukiere: 3m6m4p7p4s
    draw 4p -> cut 3m => 13 ukiere: 3p4s5s7s; cut 6m => 13 ukiere: 3p4s5s7s; cut 7s => 13 ukiere: 3m6m3p4s
    draw 7p -> cut 3m => 13 ukiere: 3p4s5s7s; cut 6m => 13 ukiere: 3p4s5s7s; cut 7s => 13 ukiere: 3m6m3p4s
    draw 5s -> cut 3m => 12 ukiere: 3p4p7p; cut 6m => 12 ukiere: 3p4p7p; cut 8s => 12 ukiere: 3p4p7p
    draw 6m -> cut 4s => 12 ukiere: 3p4p7p; cut 7s => 12 ukiere: 3p4p7p
    draw 4s -> cut 3m => 12 ukiere: 3p4p7p; cut 6m => 12 ukiere: 3p4p7p; cut 7s => 12 ukiere: 3p4p7p
    draw 7s -> cut 3m => 12 ukiere: 3p4p7p; cut 6m => 12 ukiere: 3p4p7p; cut 4s => 12 ukiere: 3p4p7p
    draw 3m -> cut 4s => 12 ukiere: 3p4p7p; cut 7s => 12 ukiere: 3p4p7p
  upgrades:
    draw 5p -> cut 7s => 71 ukiere: 1m2m3m4m5m6m7m8m3p4p5p6p7p8p2s3s4s5s6s9s; cut 4s => 65 ukiere: 1m2m3m4m5m6m7m8m3p4p5p6p7p8p5s6s7s8s9s; cut 6p => 61 ukiere: 1m2m3m4m5m6m7m8m3p5p2s3s4s5s6s7s8s9s; cut 6m => 48 ukiere: 3p4p5p6p7p8p2s3s4s5s6s7s8s9s; cut 3m => 48 ukiere: 3p4p5p6p7p8p2s3s4s5s6s7s8s9s
    draw 1p -> cut 7s => 67 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 65 ukiere: 1m2m3m4m5m6m7m8m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 61 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p5s6s7s8s9s; cut 6m => 44 ukiere: 1p2p3p4p7p2s3s4s5s6s7s8s9s; cut 3m => 44 ukiere: 1p2p3p4p7p2s3s4s5s6s7s8s9s
    draw 2p -> cut 7s => 67 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p2s3s4s5s6s9s; cut 1p => 65 ukiere: 1m2m3m4m5m6m7m8m2p4p7p2s3s4s5s6s7s8s9s; cut 4s => 61 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p5s6s7s8s9s; cut 6m => 44 ukiere: 1p2p3p4p7p2s3s4s5s6s7s8s9s; cut 3m => 44 ukiere: 1p2p3p4p7p2s3s4s5s6s7s8s9s
    draw 6p -> cut 7s => 67 ukiere: 1m2m3m4m5m6m7m8m3p4p5p6p7p2s3s4s5s6s9s; cut 5p => 61 ukiere: 1m2m3m4m5m6m7m8m3p6p2s3s4s5s6s7s8s9s; cut 4s => 61 ukiere: 1m2m3m4m5m6m7m8m3p4p5p6p7p5s6s7s8s9s; cut 3m => 44 ukiere: 3p4p5p6p7p2s3s4s5s6s7s8s9s; cut 6m => 44 ukiere: 3p4p5p6p7p2s3s4s5s6s7s8s9s
    draw 7m -> cut 7s => 50 ukiere: 2m3m4m5m6m7m8m1p2p3p4p5p6p7p4s; cut 4s => 49 ukiere: 2m3m4m5m6m7m8m1p2p3p4p5p6p7p7s; cut 1p => 31 ukiere: 2m5m8m2p4p7p4s5s7s; cut 2p => 31 ukiere: 2m5m8m1p4p7p4s5s7s
    draw 2m -> cut 7s => 50 ukiere: 1m2m3m4m5m6m7m1p2p3p4p5p6p7p4s; cut 4s => 49 ukiere: 1m2m3m4m5m6m7m1p2p3p4p5p6p7p7s; cut 1p => 31 ukiere: 1m4m7m2p4p7p4s5s7s; cut 2p => 31 ukiere: 1m4m7m1p4p7p4s5s7s
    draw 4m -> cut 7s => 46 ukiere: 2m3m4m5m6m7m1p2p3p4p5p6p7p4s; cut 4s => 45 ukiere: 2m3m4m5m6m7m1p2p3p4p5p6p7p7s
    draw 5m -> cut 7s => 46 ukiere: 2m3m4m5m6m7m1p2p3p4p5p6p7p4s; cut 4s => 45 ukiere: 2m3m4m5m6m7m1p2p3p4p5p6p7p7s
    draw 6s -> cut 4s => 45 ukiere: 3m6m1p2p3p4p5p6p7p5s6s7s8s9s; cut 7s => 43 ukiere: 3m6m1p2p3p4p5p6p7p4s5s6s9s; cut 6m => 42 ukiere: 1p2p3p4p5p6p7p4s5s6s7s8s9s; cut 3m => 42 ukiere: 1p2p3p4p5p6p7p4s5s6s7s8s9s; cut 1p => 29 ukiere: 3m6m2p4p7p4s5s7s8s; cut 2p => 29 ukiere: 3m6m1p4p7p4s5s7s8s
    draw 8s -> cut 4s => 45 ukiere: 3m6m1p2p3p4p5p6p7p5s6s7s8s9s; cut 3m => 42 ukiere: 1p2p3p4p5p6p7p4s5s6s7s8s9s; cut 6m => 42 ukiere: 1p2p3p4p5p6p7p4s5s6s7s8s9s; cut 2p => 35 ukiere: 3m6m1p4p7p4s5s6s7s8s9s; cut 1p => 35 ukiere: 3m6m2p4p7p4s5s6s7s8s9s; cut 6p => 31 ukiere: 3m6m3p5p4s5s6s7s8s9s; cut 5p => 31 ukiere: 3m6m3p6p4s5s6s7s8s9s
    draw 9s -> cut 4s => 45 ukiere: 3m6m1p2p3p4p5p6p7p5s6s7s8s9s; cut 7s => 43 ukiere: 3m6m1p2p3p4p5p6p7p4s5s6s9s; cut 3m => 42 ukiere: 1p2p3p4p5p6p7p4s5s6s7s8s9s; cut 6m => 42 ukiere: 1p2p3p4p5p6p7p4s5s6s7s8s9s; cut 1p => 29 ukiere: 3m6m2p4p7p4s5s7s8s; cut 2p => 29 ukiere: 3m6m1p4p7p4s5s7s8s
    draw 3s -> cut 7s => 44 ukiere: 3m6m1p2p3p4p5p6p7p2s3s4s5s; cut 6m => 40 ukiere: 1p2p3p4p5p6p7p2s3s4s5s7s; cut 3m => 40 ukiere: 1p2p3p4p5p6p7p2s3s4s5s7s
    draw 1m -> cut 7s => 40 ukiere: 1m2m3m6m1p2p3p4p5p6p7p4s; cut 4s => 39 ukiere: 1m2m3m6m1p2p3p4p5p6p7p7s
    draw 2s -> cut 3m => 40 ukiere: 1p2p3p4p5p6p7p2s3s4s5s7s; cut 6m => 40 ukiere: 1p2p3p4p5p6p7p2s3s4s5s7s; cut 7s => 40 ukiere: 3m6m1p2p3p4p5p6p7p2s3s4s
    draw 8m -> cut 7s => 40 ukiere: 3m6m7m8m1p2p3p4p5p6p7p4s; cut 4s => 39 ukiere: 3m6m7m8m1p2p3p4p5p6p7p7s
discard 8s -> 2 shanten, 20 ukiere tiles: 1347p57s
  after advancing shanten:
    draw 5s -> cut 3m => 16 ukiere: 1p3p4p7p7s; cut 6m => 16 ukiere: 1p3p4p7p7s
    draw 3p -> cut 3m => 12 ukiere: 4p7p5s; cut 6m => 12 ukiere: 4p7p5s; cut 1p => 12 ukiere: 4p7p5s
    draw 4p -> cut 3m => 12 ukiere: 1p3p5s7s; cut 6m => 12 ukiere: 1p3p5s7s
    draw 7p -> cut 3m => 12 ukiere: 1p3p5s7s; cut 6m => 12 ukiere: 1p3p5s7s
    draw 1p -> cut 3m => 12 ukiere: 4p7p5s; cut 6m => 12 ukiere: 4p7p5s; cut 2p => 12 ukiere: 4p7p5s
    draw 7s -> cut 3m => 12 ukiere: 4p7p5s; cut 6m => 12 ukiere: 4p7p5s; cut 2p => 12 ukiere: 4p7p5s
  upgrades:
    draw 8s -> cut 7s => 67 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p2s3s4s5s6s9s; cut 2p => 65 ukiere: 1m2m3m4m5m6m7m8m1p4p7p2s3s4s5s6s7s8s9s; cut 4s => 61 ukiere: 1m2m3m4m5m6m7m8m1p2p3p4p7p5s6s7s8s9s; cut 6m => 44 ukiere: 1p2p3p4p7p2s3s4s5s6s7s8s9s; cut 3m => 44 ukiere: 1p2p3p4p7p2s3s4s5s6s7s8s9s; cut 1p => 27 ukiere: 3m6m3p4p7p4s5s7s
    draw 2m -> cut 4s => 35 ukiere: 1m4m7m1p3p4p7p5s7s8s; cut 2p => 31 ukiere: 1m4m7m1p4p7p5s7s8s; cut 1p => 27 ukiere: 1m4m7m3p4p7p5s; cut 6s => 27 ukiere: 1m4m7m1p3p4p7p7s; cut 7s => 27 ukiere: 1m4m7m4p7p5s8s; cut 5p => 23 ukiere: 1m4m7m1p3p5s7s; cut 6p => 23 ukiere: 1m4m7m1p3p5s7s
    draw 7m -> cut 4s => 35 ukiere: 2m5m8m1p3p4p7p5s7s8s; cut 2p => 31 ukiere: 2m5m8m1p4p7p5s7s8s; cut 6s => 27 ukiere: 2m5m8m1p3p4p7p7s; cut 7s => 27 ukiere: 2m5m8m4p7p5s8s; cut 1p => 27 ukiere: 2m5m8m3p4p7p5s; cut 6p => 23 ukiere: 2m5m8m1p3p5s7s; cut 5p => 23 ukiere: 2m5m8m1p3p5s7s
    draw 4m -> cut 4s => 31 ukiere: 2m5m1p3p4p7p5s7s8s; cut 2p => 27 ukiere: 2m5m1p4p7p5s7s8s; cut 1p => 23 ukiere: 2m5m3p4p7p5s; cut 6s => 23 ukiere: 2m5m1p3p4p7p7s; cut 7s => 23 ukiere: 2m5m4p7p5s8s
    draw 5m -> cut 4s => 31 ukiere: 4m7m1p3p4p7p5s7s8s; cut 2p => 27 ukiere: 4m7m1p4p7p5s7s8s; cut 6s => 23 ukiere: 4m7m1p3p4p7p7s; cut 7s => 23 ukiere: 4m7m4p7p5s8s; cut 1p => 23 ukiere: 4m7m3p4p7p5s
    draw 1m -> cut 4s => 28 ukiere: 2m1p3p4p7p5s7s8s; cut 2p => 24 ukiere: 2m1p4p7p5s7s8s
    draw 8m -> cut 4s => 28 ukiere: 7m1p3p4p7p5s7s8s; cut 2p => 24 ukiere: 7m1p4p7p5s7s8s
    draw 3s -> cut 3m => 28 ukiere: 1p3p4p7p2s5s7s8s; cut 6m => 28 ukiere: 1p3p4p7p2s5s7s8s; cut 2p => 24 ukiere: 1p4p7p2s5s7s8s; cut 6s => 24 ukiere: 1p3p4p7p2s5s7s
    draw 2s -> cut 3m => 28 ukiere: 1p3p4p7p3s5s7s8s; cut 6m => 28 ukiere: 1p3p4p7p3s5s7s8s; cut 2p => 24 ukiere: 1p4p7p3s5s7s8s
    draw 2p -> cut 4s => 26 ukiere: 1p2p3p4p7p5s7s8s; cut 3m => 26 ukiere: 1p2p3p4p7p5s7s8s; cut 6m => 26 ukiere: 1p2p3p4p7p5s7s8s
    draw 6m -> cut 4s => 26 ukiere: 6m1p3p4p7p5s7s8s; cut 7s => 24 ukiere: 6m1p3p4p7p5s8s; cut 1p => 24 ukiere: 6m3p4p7p5s7s8s; cut 2p => 22 ukiere: 6m1p4p7p5s7s8s
    draw 6s -> cut 3m => 26 ukiere: 1p3p4p7p5s6s7s8s; cut 6m => 26 ukiere: 1p3p4p7p5s6s7s8s; cut 4s => 26 ukiere: 1p3p4p7p5s6s7s8s; cut 2p => 22 ukiere: 1p4p7p5s6s7s8s
    draw 3m -> cut 4s => 26 ukiere: 3m1p3p4p7p5s7s8s; cut 7s => 24 ukiere: 3m1p3p4p7p5s8s; cut 1p => 24 ukiere: 3m3p4p7p5s7s8s; cut 2p => 22 ukiere: 3m1p4p7p5s7s8s
    draw 4s -> cut 6m => 26 ukiere: 1p3p4p7p4s5s7s8s; cut 3m => 26 ukiere: 1p3p4p7p4s5s7s8s; cut 1p => 24 ukiere: 3p4p7p4s5s7s8s; cut 7s => 24 ukiere: 1p3p4p7p4s5s8s; cut 2p => 22 ukiere: 1p4p7p4s5s7s8s
    draw 9s -> cut 6m => 24 ukiere: 1p3p4p7p5s7s8s; cut 3m => 24 ukiere: 1p3p4p7p5s7s8s
    draw 5p -> cut 3m => 22 ukiere: 1p3p4p5p7p5s7s; cut 6m => 22 ukiere: 1p3p4p5p7p5s7s
    draw 6p -> cut 3m => 22 ukiere: 1p3p4p6p7p5s7s; cut 6m => 22 ukiere: 1p3p4p6p7p5s7s
discard 5p -> 3 shanten, 82 ukiere tiles: 1m2m3m4m5m6m7m8m1p2p3p4p5p6p7p8p2s3s4s5s6s7s8s9s
discard 6p -> 3 shanten, 78 ukiere tiles: 1m2m3m4m5m6m7m8m1p2p3p4p5p6p7p2s3s4s5s6s7s8s9s
discard 4m -> 3 shanten, 72 ukiere tiles: 1m2m3m4m5m6m7m8m1p2p3p4p7p2s3s4s5s6s7s8s9s
discard 5m -> 3 shanten, 72 ukiere tiles: 1m2m3m4m5m6m7m8m1p2p3p4p7p2s3s4s5s6s7s8s9s
discard 6s -> 3 shanten, 72 ukiere tiles: 1m2m3m4m5m6m7m8m1p2p3p4p7p2s3s4s5s6s7s8s9s
```

### Jun 17 2025

fixed a bug in generating the hand interpretations, when a triplet of tiles could be considered as three separate tile groups e.g.
666778s -> 678s-67s-6s (in addition to 666s-778s and 66s-6778s)

redoing the benchmarks:

```
test shanten::tests::bench_standard_shanten           ... bench:   1,965,185.90 ns/iter (+/- 67,461.67)
test shanten::tests::bench_standard_shanten_optimized ... bench:     191,675.43 ns/iter (+/- 9,942.51)

(on a subsequent run:)
test shanten::tests::bench_standard_shanten           ... bench:   1,956,415.40 ns/iter (+/- 56,258.87)
test shanten::tests::bench_standard_shanten_optimized ... bench:     192,097.47 ns/iter (+/- 6,602.36)
```

this hasn't changed much - but because this change only impacts when there are 3 copies of a tile in the hand and several nearby tiles.

an interesting analysis of this situation from one of my tenhou replays:

```
hand: 345m11256p46778s6s
dora indicator: 6s (dora: 7s)
round: east-1, seat: east, points: 25k points all
turn: 8
discard pools:
self   (east): 4z2z3z9m9p7z6m
shimo (south): 8p4z1z2z3p3p9s
toimen (west): 4z5z6z3z1s3z7z
kami  (north): 4z1z6z5z1s4m9m

Analysis:
- The intuitive decision is between cut 2p and cut 4s (the hand is 1-shanten: 345m-11p-56p-678s-67s, with 2p and 4s as floating tiles). Both of these options result in 15 ukiere tiles: 47p58s, and the resulting wait at tenpai is guaranteed to be ryanmen wait.

upgrades after cut 2p: 1 56p234679s
upgrades after cut 4s: 1356p   679s

discard 2p -> 1 shanten, 15 ukiere tiles: 47p58s
  upgrades:
    draw 1p -> cut 4s => 29 ukiere
  - (no upgrade on 3p)
    draw 5p -> cut 4s => 19 ukiere
    draw 6p -> cut 4s => 19 ukiere
  + draw 2s -> cut 7s => 16 ukiere
  + draw 3s -> cut 6s => 16 ukiere
  + draw 4s -> cut 7s => 16 ukiere
    draw 6s -> cut 4s => 24 ukiere
    draw 7s -> cut 4s => 20 ukiere
    draw 9s -> cut 4s => 19 ukiere

discard 4s -> 1 shanten, 15 ukiere tiles: 47p58s
  upgrades:
    draw 1p -> cut 2p => 29 ukiere
  + draw 3p -> cut 1p => 29 ukiere
    draw 5p -> cut 2p => 19 ukiere
    draw 6p -> cut 2p => 19 ukiere
  - (no upgrade on 2s)
  - (no upgrade on 3s)
  - (no upgrade on 4s)
    draw 6s -> cut 2p => 24 ukiere
    draw 7s -> cut 2p => 20 ukiere
    draw 9s -> cut 2p => 19 ukiere

I think in the end discarding 4s is better: we want to keep the dora that we already have, and the sacrificed upgrades on 234s are not that significant (from 15 -> 16 ukiere tiles, in exchange for giving up the 7s dora). Also, drawing 3p is a major upgrade: from 15 -> 29 ukiere tiles.

But factoring in the discards: we see two 3p discarded by shimo.
```

### Jun 7 2025

Added chiitoi and kokushi shanten + ukiere calculation. Redoing the benchmarks:

```
test shanten::tests::bench_standard_shanten           ... bench:   1,943,684.20 ns/iter (+/- 40,921.91)
test shanten::tests::bench_standard_shanten_optimized ... bench:     188,982.62 ns/iter (+/- 573.10)

(on a subsequent run:)
test shanten::tests::bench_standard_shanten           ... bench:   1,943,132.95 ns/iter (+/- 103,465.39)
test shanten::tests::bench_standard_shanten_optimized ... bench:     195,453.67 ns/iter (+/- 2,544.07)

(running the optimized version on its own:)
test shanten::tests::bench_standard_shanten_optimized ... bench:     189,643.83 ns/iter (+/- 9,849.01)
```

### Jun 5 2025

Improved the optimizations: we order the queue items based on number of tiles remaining (to be melded/grouped)
and also compute the best possible shanten that you could reach (given the remaining ungrouped tiles) to
determine if we should even bother recursively breaking down the rest of the tiles into groups.

Also fixed a bug caused by over-optimizing (the bug was that the algorithm would not make isolated tiles,
even in cases when doing so wouldn't increase the shanten - causing it to miss potential ukiere tiles)

```
test shanten::tests::bench_standard_shanten           ... bench:   1,971,224.10 ns/iter (+/- 124,494.48)
test shanten::tests::bench_standard_shanten_optimized ... bench:     284,417.75 ns/iter (+/- 27,971.77)

(on a subsequent run:)
test shanten::tests::bench_standard_shanten           ... bench:   2,000,364.80 ns/iter (+/- 52,829.04)
test shanten::tests::bench_standard_shanten_optimized ... bench:     282,541.50 ns/iter (+/- 12,435.83)

(running the optimized version on its own:)
test shanten::tests::bench_standard_shanten_optimized ... bench:     281,369.45 ns/iter (+/- 17,551.03)
```

The result: we're now processing the standard shanten + ukiere in ~280-285 microseconds, which is ~7x faster than previously!

### Jun 4 2025

trying the benchmarking again: seems about 1930-1995 microseconds

```
test shanten::tests::bench_standard_shanten ... bench:   1,995,319.98 ns/iter (+/- 176,700.20)
(on a subsequent run:)
test shanten::tests::bench_standard_shanten ... bench:   1,932,268.70 ns/iter (+/- 46,140.96)
```

benchmarking after some optimizations: the optimized version is around 1240 microseconds

```
test shanten::tests::bench_standard_shanten           ... bench:   1,938,866.70 ns/iter (+/- 18,189.99)
test shanten::tests::bench_standard_shanten_optimized ... bench:   1,243,399.85 ns/iter (+/- 10,993.34)
(subsequent run:)
test shanten::tests::bench_standard_shanten           ... bench:   1,939,448.70 ns/iter (+/- 46,928.38)
test shanten::tests::bench_standard_shanten_optimized ... bench:   1,245,522.35 ns/iter (+/- 31,017.02)

(subsequent run with just the optimized function, to see if running it separately has similar performance)
test shanten::tests::bench_standard_shanten_optimized ... bench:   1,240,828.05 ns/iter (+/- 38,928.19)
```

### Jun 3 2025

benchmarking the get_shanten and get_ukiere function calls: on the hand 12234455s345p11z (using `cargo bench`)

```
test shanten::tests::bench_standard_shanten ... bench:   1,934,480.60 ns/iter (+/- 20,149.04)
```

also started implementing shanten & ukiere for chiitoi & kokushi (got shanten working, but need to add in the logic for ukiere)

### Jun 2 2025

Finished the new, simplified implementation of the shanten / ukiere calculation in `shanten.rs` - should probably remove the old code in `mahjong_meld.rs` and whatever we don't need from `mahjong_hand.rs` as well

next steps:

- benchmarking / optimization - how fast is the shanten/ukiere computation? Can we optimize it? some ideas:
  - as we build all possible hand interpretations, build in a depth-first & greedy way, and keep track of the minimum-shanten achieved by a complete interpretation thus far. Then you can discard/exit early from other interpretations that won't be better?
  - ordering heuristics: consider suits with fewer tiles first? try to find "stable" groups i.e. those that have unambiguous interpretation (like how honor tiles must all be used in a meld together, or if there is a single group of 3 tiles that is > 2 tiles away from all other tiles in that suit)
  - splitting subproblem / caching: split by suit (manzu, pinzu, souzu) or when tiles are > 2 tiles away from all other tiles -> maybe these subproblems can be pre-computed into a file that is loaded into memory for faster lookup? it doesn't matter which suit the tiles are in (for pure shanten/ukiere calculations). Also there is symmetry: if you know the shanten/ukiere for the group 3445567, then you also know the shanten/ukiere for the group 3455667 (there is symmetry around the 5 tile, except for the relationship between dora indicator and dora, but that is only useful for the expected value computation, and only if you will do something advanced like estimating expected value with taking potential uradora into account.)
- shanten computation after draw - naive approach is to first check if the drawn tile makes a complete hand. and if not, try discarding each distinct tile value from the hand, and see what the resulting hand's shanten is. But is there a better way?
- expected speed to tenpai / win - to simplify, assume we can only rely on self-draws, no calling chii/pon/kan or ron. Can simplify further if we ignore furiten. What is the probability of getting to tenpai in the next X draws? This seems complicated to find a closed-form solution for, as it can fluctuate as more tiles are discarded and if there are upgrade opportunities.
- expected value of hand - to compute this, we need additional information: seat wind & round wind, dora indicator(s). We could also incorporate other player's discards to get an even better sense of which tiles remain. We could continue to assume that the hand will remain closed until tenpai. We would need to be able to identify and score han (yaku) and fu.

### May 20 2025

comparing the is_winning_shape check using the `build_shapes` function (which is iterative) - compare to the ~12.5-15 microseconds from the optimized recursive heuristic implementation below:

```
elapsed time ~50-89 microseconds (for winning shape)
elapsed time ~40-57 microseconds (for not winning shape)
```

but for the tenpai check (i.e. determining whether the hand is in tenpai and if so, which tiles it will win on), using the `build_shapes` function:

```
elapsed time 55-70 microseconds (for a tenpai shape)
```

### May 17 2025

benchmark for tenpai evaluation: i.e. the problem of identifying if the hand is in tenpai (and if so, which tiles it will win on) -- the problem of hand scoring (which requires determining the maximum-scoring hand shape & wait) is deferred to a later point

the benchmark test: hand 22s111234p34789m (wins on 25m)

the "brute force" approach: try adding each of the possible 34 tiles to the hand, and check whether the resulting hand is a winning shape (using the recursive approach which was found below)

```
elapsed time ~150-155 microseconds (to find all possible winning tiles)
```

We should aim to do better than this, if we use a "smarter" approach.

For a chuuren poutou / pure nine gates tenpai hand, the elapsed time is a bit longer:

```
Elapsed time on chuuren poutou hand: 168-175 microseconds (to find all possible winning tile)
```

### May 3 2025

Replacing the `tile_id % 9` with a helper function that uses if conditions to determine how much to subtract from the `tile_id` (e.g. `(tile_id - 9)` for pinzu), seems to be slightly faster:

```
after replacing the % operator with a call to get_num_tile_rank function: ~12.5-13.5 microseconds (for not winning hand)
~14-15 microseconds for winning hand
```

### May 2 2025

tuning the recursive implementation: added check for quads, added initial check for isolated tiles + honor tiles (before initial recursive call)

```
after adding check for quads: 51-84 microseconds
after moving the check for honor tiles to initial check only (not on each recursive call): ~47 microseconds
after passing in next tile_id with non-zero tile count as an argument for each recursive call: ~47 microseconds
after commenting out the println (which were added for debugging): ~15 microseconds
```

### May 1 2025

- seems the recursive implementation of is_winning_shape is faster than the iterative implementation:

```
(running each one independently)
Elapsed time for is_winning_shape_iterative: ~123 - 190 microseconds
Elapsed time for is_winning_shape_recursive: ~118 - 145 microseconds
Elapsed time for is_winning_shape_recursive_heuristic: ~85 - 110 microseconds
```

- ~~on multiple attempts, the elapsed time for `is_winning_shape_iterative` is ~120 microseconds, and the elapsed time for `is_winning_shape_recursive` is ~90 microseconds~~
- ~~why? maybe because building and updating the `VecDeque` is slower than just making recursive calls?~~
- We can also do some heuristic checks / optimizations: instead of naively iterating through the indexes in the array, we can try checking for isolated tiles first, then checking honors, before checking the number suits.
- oh maybe it's due to the compiler optimizations / ordering? if i run the `is_winning_shape_recursive_heuristic` function on its own, i get a higher time than if i run it after the `is_winning_shape_iterative` and `is_winning_shape_recursive` functions. Also, running the `is_winning_shape_recursive_heuristic` function multiple times in a row shows different times: usually, the first attempt is slower, around 150 microseconds, and then the subsequent attempts are faster, around 80 microseconds.

can use this `cargo test mahjong_hand::tests::time_is_winning_shape -- --show-output` command to run multiple tests (based on the prefix test name filter)

when running on hand that is not a winning shape:

```
(running each one independently)
Elapsed time for is_winning_shape_iterative: ~46 - 65 microseconds
Elapsed time for is_winning_shape_recursive: ~38 - 57 microseconds
Elapsed time for is_winning_shape_recursive_heuristic: ~37 - 54 microseconds
```

### Apr 29 2025

- reading through the Rust implementation [here](https://github.com/harphield/riichi-tools-rs/blob/master/src/riichi/hand.rs) for inspiration / tips in Rust
- learned about the Default trait, and practiced basic Rust coding (iterating using .iter() to avoid a move, handling Result types, defining enums with subtypes)
- spent about 2 hrs today after starting from scratch on the tile class -- can represent each tile value as one of 34 integers (corresponding to index positions in an array, instead of a more complicated hash map of suit -> array of counts per rank)
- next up: implement winning-hand-shape checker - can worry about performance/optimization, computing tenpai/shanten/yaku, etc. later

### Apr 26 2025

- new goal: build a "mahjong solitaire" / singleplayer mahjong website: simulate the training mode of getting to tenpai as quickly as possible from random starting hand
  - subproblem 1: build riichi mahjong game rules library (can reference existing rust library: https://github.com/harphield/riichi-tools-rs)
    - 1.a: define hand state, drawing tiles, and valid hand winning shape & valid yaku
    - 1.b: build in logic for furiten (i.e. the final wait cannot include a tile that you've already discarded)
  - subproblem 2: build a server to handle web requests, store a new "problem" (i.e. starting hand, and hidden random sequence of tiles to be drawn)
  - subproblem 3: build the UI for the website
  - subproblem 4: try to optimize the performance of the riichi library
- next goal: AI to analyze the discards for single-player mahjong (similar to WordleBot - it offers explanation of what it would have done, and why)
- next goal: scrape Tenhou game logs

### May 16 2023

- Draft list of tasks
- set a timeline - 9 weeks total
- what i tried for the "is_winning_hand" function

  - initially, i just tried counting tiles by suit and rank - this helps for honor tiles, but number tiles can be tricky, especially with overlapping sequences.
  - My initial idea was to use the counts by rank for each suit (since the suits are independent of each other) - and try to identify isolated tiles, but this is not strict enough to catch situations when there are non-winning hands that just have tiles that are close/neighboring & this also misses hands where the end of the sequence is deemed "isolated" since no sequence could start with that -> the missing piece is to remove the tiles from the hand for consideration, which starts to seem like a recursive solution
  - I was concerned about a recursive solution, but I think that it should be safe due to the low maximum depth - each recursive call will remove at least 2 (and usually at least 3) tiles away from the list of remaining tiles, and the fanout is not high - there are at most 4 options if a single tile has 4 of a kind: make a meld with all four tiles together, make a meld with three of the tiles, use two of the tiles for the pair, or make a meld with one of the tiles for a sequence

- spent ~2 hours on the winning hand function today (before setting the timeline) & probably ~4 hours before that on the existing code
- spent ~1.5 hours on the recursive approach for hand grouping - added a few new unit tests as well.

### May 17 2023

- spent ~1 hour trying to add the sequence detection for number suits - I want to add a helper to remove a single tile from a Vector of Strings, but it isn't working in my unit test?

### May 18 2023

- spent ~0.75 hr fixing the unit test - i got the \_remove_one_copy helper function to work (not sure why it wasn't working with the splitn() approach I tried previously) - but now I'm facing an error where I'm returning duplicate winning hands in the nine gates test
- my plan to handle this is to make the `WinningHand` & `PartialWinningHand` structs include `HashSet<HandMeld>`, and then we can do an equality check, however, this requires implementing the Hash, PartialEq, and Eq traits on the `HandMeld` struct
  - but is there another (faster/more efficient) option? maybe we can set some ordering on the hand melds (try adding sequences first, then if you can make it work with a sequence, then when you recurse back, you can't use a triplet or quad of that same tile -- but we need to formalize this notion)

### May 19 2023

- spent ~1 hour fixing the duplicated winning hands problem - I was going to try to implement Hash trait for the WinningHand, but that would require hashing the HashSet of tiles in the HandMeld struct, which I read is not trivial to do in with high performance. I could sort the tiles, but that seems like a lot of work. I came up with a simpler solution: before making a recursive call for a pair, triplet, or quad, check if any of the existing winning hands have that pattern already. If so, don't make the recursive call as that would produce a false duplicate winning hand.
- I also noticed from debugging that the triplet / quad HandMeld structs only have one copy of the tile. I suppose this is because it's a HashSet. But it might make things tricky later on when the HandMeld might not count as three of the same tile (e.g. red fives, or when there is an open meld, tracking which player the discarded tile was called from, etc.)

### May 20 2023

- my plan for today is to add more unit tests for the hand grouping function
- if I have time, I can try to implement the yaku checks, as a hand with no yaku is not technically a winning hand

### May 23 2023

- spent ~0.5 hours on plane thinking about the different entities needed to represent game state: tiles, player hands, winning hand groupings, etc.
- wrote these ideas down for later implementation / refactor of existing code

### May 26 2023

- spent ~0.75 hours on refactor to use Rust structs to represent tiles, hand melds, hand groups, winning hands, etc.
- started with the tile struct, read the Rust book section on structs, implementation blocks, methods, etc. [here](https://doc.rust-lang.org/book/ch05-01-defining-structs.html)

### May 27 2023

- spent ~3.5 hours
- day 2 of refactor to use Rust structs & enums
- learning about why the rust compiler complains about unused functions when they are used within the test modules, source: [stackoverflow](https://stackoverflow.com/questions/68836263/why-is-rust-complaining-about-an-unused-function-when-it-is-only-used-from-tests)
- learning about From/Into and TryFrom/TryInto traits - see [stackoverflow](https://stackoverflow.com/questions/35283736/whats-the-closest-i-can-get-to-discriminating-an-enum-by-a-char), and [rust by example section on Try and From](https://doc.rust-lang.org/rust-by-example/conversion/from_into.html)
- getting more comfortable with enums, see above stackoverflow link and the rust programming language book [section on enums](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html) and the rust by example [section on enums](https://doc.rust-lang.org/rust-by-example/custom_types/enum.html)

### May 29 2023

- spent ~1 hour
- day 3 of refactor to use Rust structs & enums: started on the HandMeld (now called TileGroup) data entity -> chose to represent as an enum

### May 30 2023

- spent ~0.5 hour
- day 4 of refactor to use Rust structs & enums: trying to implement the hand_grouping function

### May 31 2023

- spent ~1 hour
- day 5 of refactor to use Rust structs & enums: still implementing the hand_grouping function, got some progress but there are a few compiler errors I need to resolve
- I also will need to add a lot of test cases to handle the red-five tiles as the same rank as normal five tiles -> perhaps need to update `count_tiles_by_suit_rank` to count red-five and normal-five as the same? or just consider them as the same when calling `count_tiles_by_suit_rank` in `hand_grouping` function?

### Jun 3 2023

- spent ~2 hours
- day 6 of refactor to use Rust structs & enums: fixing up the hand_grouping function to handle red-fives appropriately, did some trait implementations on the TileSuit / TileRank enums and the Tile struct
- some learning of the "cannot move out of XYZ which is behind a shared reference" error: https://stackoverflow.com/questions/61995143/cannot-move-out-of-which-is-behind-a-shared-reference

### Jun 4 2023

- spent ~2.5 hours
- refactor of the base Tile, TileGroup, etc. structs / entities is complete, I've moved onto adding new structs to represent the player state, the state of the game/round/hand, which we can use to identify yaku
- I'm also starting to split up the code into separate source files as the single main.rs file was getting a bit too large
