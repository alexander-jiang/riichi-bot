# riichi-bot

Solver to play riichi mahjong (heuristic-based vs. RL-based methods)

ideas

- start by training RL model with 1player version - there is no tile calling, you just draw and discard
  - reward function accounts for different yaku / han value
  - we should adjust the number of drawn tiles to approximate the 4 player environment (you don't have access to draw every single tile)
  - should we still reveal tiles as if other players are drawing & discarding?
  - should we include exhaustive draw situation? - achieving tenpai at the end is important
