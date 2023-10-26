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

100000 trials: win % = 94.756, avg draws = 7.0655
avg draws (wins only) = 6.6419225

### statistical analysis

Starting hand can achieve tenpai by discarding the 9, and waiting on either the 5 or 8 -> 8 winning tiles out of 31 remaining tiles.

P(# of draws = 1) = 8/31
P(# of draws = 2) = 23/31 * 8/30

## Experiment 2:

Starting with the same set of N initial hands (generated randomly)

### naive strategy #1: random discard

### naive strategy #2: discard lowest tile
