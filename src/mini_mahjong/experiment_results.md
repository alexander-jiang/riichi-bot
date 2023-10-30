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

### statistical analysis

Starting hand can achieve tenpai by discarding the 9, and waiting on either the 5 or 8 -> 8 winning tiles out of 31 remaining tiles.

P(# of draws = 1) = 8/31
P(# of draws = 2) = 23/31 \* 8/30
P(# of draws = 3) = 23/31 \* 22/30 \* 8/29
P(# of draws = k) = (23! / (23-k+1)!) / (31! / (31-k+1)!) \* 8 / (31-k+1)

Alternatively, use the negative hypergeometric distribution, which models the number of unmarked objects pulled (if sampling without replacement), before pulling m=1 marked objects (i.e. winning tiles). If N=31 total elements, and M=8 marked objects, then the mean of X (number of unmarked objects pulled) is:
`m * (N - M) / (M + 1) = 1 * 23 / 9 = 2.5556`

Therefore, the average total number of draws is the mean + m, or 3.5556

## Experiment 2:

Starting with the same set of N initial hands (generated randomly)

### naive strategy #1: random discard

### naive strategy #2: discard lowest tile
