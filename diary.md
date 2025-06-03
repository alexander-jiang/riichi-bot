# Diary

### Jun 2 2025

Finished the new, simplified implementation of the shanten / ukiere calculation in `shanten.rs` - should probably remove the old code in `mahjong_meld.rs` and whatever we don't need from `mahjong_hand.rs` as well

next steps:
* benchmarking / optimization - how fast is the shanten/ukiere computation? Can we optimize it? some ideas:
    * as we build all possible hand interpretations, build in a depth-first & greedy way, and keep track of the minimum-shanten achieved by a complete interpretation thus far. Then you can discard/exit early from other interpretations that won't be better?
    * ordering heuristics: consider suits with fewer tiles first? try to find "stable" groups i.e. those that have unambiguous interpretation (like how honor tiles must all be used in a meld together, or if there is a single group of 3 tiles that is > 2 tiles away from all other tiles in that suit)
    * splitting subproblem / caching: split by suit (manzu, pinzu, souzu) or when tiles are > 2 tiles away from all other tiles -> maybe these subproblems can be pre-computed into a file that is loaded into memory for faster lookup? it doesn't matter which suit the tiles are in (for pure shanten/ukiere calculations). Also there is symmetry: if you know the shanten/ukiere for the group 3445567, then you also know the shanten/ukiere for the group 3455667 (there is symmetry around the 5 tile, except for the relationship between dora indicator and dora, but that is only useful for the expected value computation, and only if you will do something advanced like estimating expected value with taking potential uradora into account.)
* shanten computation after draw - naive approach is to first check if the drawn tile makes a complete hand. and if not, try discarding each distinct tile value from the hand, and see what the resulting hand's shanten is. But is there a better way?
* expected speed to tenpai / win - to simplify, assume we can only rely on self-draws, no calling chii/pon/kan or ron. Can simplify further if we ignore furiten. What is the probability of getting to tenpai in the next X draws? This seems complicated to find a closed-form solution for, as it can fluctuate as more tiles are discarded and if there are upgrade opportunities.
* expected value of hand - to compute this, we need additional information: seat wind & round wind, dora indicator(s). We could also incorporate other player's discards to get an even better sense of which tiles remain. We could continue to assume that the hand will remain closed until tenpai. We would need to be able to identify and score han (yaku) and fu.

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
