# Diary


### May 16 2023
* Draft list of tasks
* set a timeline - 9 weeks total
* what i tried for the "is_winning_hand" function
    * initially, i just tried counting tiles by suit and rank - this helps for honor tiles, but number tiles can be tricky, especially with overlapping sequences.
    * My initial idea was to use the counts by rank for each suit (since the suits are independent of each other) - and try to identify isolated tiles, but this is not strict enough to catch situations when there are non-winning hands that just have tiles that are close/neighboring & this also misses hands where the end of the sequence is deemed "isolated" since no sequence could start with that -> the missing piece is to remove the tiles from the hand for consideration, which starts to seem like a recursive solution
    * I was concerned about a recursive solution, but I think that it should be safe due to the low maximum depth - each recursive call will remove at least 2 (and usually at least 3) tiles away from the list of remaining tiles, and the fanout is not high - there are at most 4 options if a single tile has 4 of a kind: make a meld with all four tiles together, make a meld with three of the tiles, use two of the tiles for the pair, or make a meld with one of the tiles for a sequence

* spent ~2 hours on the winning hand function today (before setting the timeline) & probably ~4 hours before that on the existing code
* spent ~1.5 hours on the recursive approach for hand grouping - added a few new unit tests as well.

### May 17 2023
* spent ~1 hour trying to add the sequence detection for number suits - I want to add a helper to remove a single tile from a Vector of Strings, but it isn't working in my unit test?


### May 18 2023
* spent ~0.75 hr fixing the unit test - i got the _remove_one_copy helper function to work (not sure why it wasn't working with the splitn() approach I tried previously) - but now I'm facing an error where I'm returning duplicate winning hands in the nine gates test
* my plan to handle this is to make the `WinningHand` & `PartialWinningHand` structs include `HashSet<HandMeld>`, and then we can do an equality check, however, this requires implementing the Hash, PartialEq, and Eq traits on the `HandMeld` struct
    * but is there another (faster/more efficient) option? maybe we can set some ordering on the hand melds (try adding sequences first, then if you can make it work with a sequence, then when you recurse back, you can't use a triplet or quad of that same tile -- but we need to formalize this notion)


### May 19 2023
* spent ~1 hour fixing the duplicated winning hands problem - I was going to try to implement Hash trait for the WinningHand, but that would require hashing the HashSet of tiles in the HandMeld struct, which I read is not trivial to do in with high performance. I could sort the tiles, but that seems like a lot of work. I came up with a simpler solution: before making a recursive call for a pair, triplet, or quad, check if any of the existing winning hands have that pattern already. If so, don't make the recursive call as that would produce a false duplicate winning hand.
* I also noticed from debugging that the triplet / quad HandMeld structs only have one copy of the tile. I suppose this is because it's a HashSet. But it might make things tricky later on when the HandMeld might not count as three of the same tile (e.g. red fives, or when there is an open meld, tracking which player the discarded tile was called from, etc.)


### May 20 2023
* my plan for today is to add more unit tests for the hand grouping function
* if I have time, I can try to implement the yaku checks, as a hand with no yaku is not technically a winning hand



