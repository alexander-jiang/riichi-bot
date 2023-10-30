## Experiment 1:

Fixed initial hand of 22679, 100k trials per strategy

### naive strategy #1: random discard

100000 trials: win % = 73.41, avg draws = 16.8842
avg draws (wins only) = 12.196418

### naive strategy #2: discard lowest tile

100000 trials: win % = 86.62, avg draws = 9.42664
avg draws (wins only) = 8.001524

### naive strategy #3: discard highest tile

100000 trials: win % = 83.948, avg draws = 7.72024
avg draws (wins only) = 6.280233

### naive strategy #3: discard isolated tiles first (otherwise random)

"isolated tile" = there are no tiles of the same rank, or of neighboring rank(s)

100000 trials: win % = 94.756, avg draws = 7.0655
avg draws (wins only) = 6.6419225

### brute force strategy #1: hold the 2267 tenpai

100000 trials: win % = 100, avg draws = 3.56863
avg draws (wins only) = 3.56863

1000000 trials: win % = 100, avg draws = 3.558634
avg draws (wins only) = 3.558634

### pick the discard that results in tenpai with most remaining tiles (otherwise discard randomly)

100000 trials: win % = 100, avg draws = 3.54575
avg draws (wins only) = 3.54575

### statistical analysis

Starting hand can achieve tenpai by discarding the 9, and waiting on either the 5 or 8 -> 8 winning tiles out of 31 remaining tiles.

P(# of draws = 1) = 8/31
P(# of draws = 2) = 23/31 \* 8/30
P(# of draws = 3) = 23/31 \* 22/30 \* 8/29
P(# of draws = k) = (23! / (23-k+1)!) / (31! / (31-k+1)!) \* 8 / (31-k+1)

Alternatively, use the negative hypergeometric distribution, which models the number of unmarked objects pulled (if sampling without replacement), before pulling m=1 marked objects (i.e. winning tiles). If N=31 total elements, and M=8 marked objects, then the mean of X (number of unmarked objects pulled) is:
`m * (N - M) / (M + 1) = 1 * 23 / 9 = 2.5556`

Therefore, the average total number of draws is the mean + m, or 3.5556

## Experiment 1.1:

Fixed initial hand of 12257, 100k trials per strategy

### Holding the initial tenpai wait

Only way to get to tenpai from the initial hand is to discard the 1, so the resulting tiles are 2257, waiting for the 6 (4 copies left). Using negative hypergeometric distribution (as above), with M=4 marked tiles, N=31 tiles left, and m=1, then the average number of total draws needed to win on the 6 is 1 + (27 / 5) = 6.4 draws.

100000 trials: win % = 100, avg draws = 6.37479
avg draws (wins only) = 6.37479

1000000 trials: win % = 100, avg draws = 6.401124
avg draws (wins only) = 6.401124

But we don't have to only wait on the 6. For example, if we first draw a 4 (resulting hand is 22457), instead of discarding the 4 (to continue waiting on the 6), we can instead discard the 7. This leaves us with the tiles 2245, which can win on both the 3 and 6, improving the likelihood of winning.

### pick the discard that results in tenpai with most remaining tiles (otherwise discard randomly)

This strategy will always always prioritize getting to tenpai - but among the discards that result in tenpai, the strategy looks to discard the tile that results in the tenpai with the most winning tiles that remain.

100000 trials: win % = 100, avg draws = 4.84159
avg draws (wins only) = 4.84159

However, if there are multiple such tiles (that result in tenpai with the same number of winning tiles remaining), the current strategy discards the lowest ranked tile. This can result in slightly suboptimal decisions. For example, from 22579, discarding either the 5 or the 9 will result in a tenpai that waits on the maximum number of tiles: 4 tiles (the 6 or the 8). However, discarding the 5 (to become tenpai with 2279) results in a shape that can improve the wait (to 8 tiles) by drawing a 6, whereas discarding the 9 (to become tenpai with 2257) results in a shape that can improve the wait to 7 or 8 tiles by drawing either a 8 or a 4, respectively, as one copy of the 9 is already dead, so going from 2257 -> 22578 -> 2278 waits on 7 remaining tiles).

## Experiment 2:

Starting with the same set of N initial hands (generated randomly)

### naive strategy #1: random discard

### naive strategy #2: discard lowest tile
