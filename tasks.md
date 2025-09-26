end goal: evaluate and analyse mahjong plays (like a chess engine) with interpretable commentary

### milestone 1: Fundamentals

- recognizing a winning hand
- recognizing whether a hand is tenpai or not (which is needed for riichi)
  - not strictly necessary to know which tiles are winning, but that will be needed for han/fu scoring (e.g. if the hand can be grouped to have a ryanmen wait, a non-ryanmen wait, or both)
- recognizing all yaku

### milestone 2: Tenpai Value and Shanten

- point counting for a winning hand based on han and fu
  - need to be able to determine all possible groupings of the winning hand's tiles, and then pick the one with the maximum value (e.g. in a choice between pinfu + ryanmen wait or non-ryanmen wait -- should score as pinfu + ryanmen wait e.g. 456m123s3456678p -> winning on 6p could be considered a tanki wait: 345-6-678p or a ryanmen wait: 345-66-78p)
  - need to track many other things besides the tiles in the hand: which tile groups are open/declared, what is the player's seat wind, what is the round win, where was the winning tile drawn from (self-draw, dead-wall-draw after a kan, or claimed from discard)
- in tenpai, what are the winning tiles, and what is the point value for each winning tile? (to get an expected value of tenpai hand winning) -> should the player call for riichi or is that call unlikely to win?
- how close is the player's hand to tenpai? (N-shanten i.e. N away from tenpai)
  - For 1-shanten hands, what is the likelihood of getting to tenpai, and then winning (once you get to tenpai, you can win off of any player's discard -- some yaku aren't required to be closed, so they can progress off of other player's discards as well)
- what yaku are available for the player given the current game state? (round wind, seat wind, dead tiles, the player's tiles, whether the player's hand is open / closed)
  - given the current hand, what is expected likelihood of winning with a specific yaku?

### milestone 3: Getting to Tenpai

- What is the likelihood of a player reaching tenpai with a closed hand i.e. being able to call riichi? (assuming player only can progress their hand through draws) - (how to balance between pushing for a win in the mid-game vs. abandoning a hopeless hand and avoiding the deal-in penalty)
  - if you consider tenpai with open hands too, the player could open their hand and progress towards tenpai more quickly (especially if the tile they are calling for is rare & abandoning that meld would slow down their hand) while sacrificing the ability to win with certain yaku for that hand
- what is the likelihood that a player will reach tenpai before exhaustive draw? (assuming player calls for all possible discards) - (i.e. in end-game situations, avoiding the noten penalty vs. avoiding deal-in)

## Timeline

### Milestone 1

- Recognizing a winning hand + yaku is surprisingly complicated - I expect 2 weeks (~10 hrs) to implement and test this
- Also spend some time thinking ahead about the interface for later components, and we may need to refactor - 1 week (~5 hrs)

Total estimate: 3 weeks (~15 hrs)

### Milestone 2

- Tenpai winning tiles should be easy, point counting should be easy too - I estimate 1 week (~5 hours) will be more than enough to implement and test
- Counting shanten is easy in brute-force, but I'd like to optimize performance here if possible. Brute force implementation should be ~2 hours, but performance optimization will be more time (I'll limit to 5 hours, as the problems & solutions are unknown at this time and could easily get out of control)
- understanding yaku is important & also can answer the question of what yaku to play for (weighted by risk-reward, maybe you need a large win to improve your standing) - I estimate 2 weeks (~10 hrs) since there are many edge cases and lots of testing to be done

Total estimate: 4 weeks (~20 hrs)

### Milestone 3

- Likelihood of reaching tenpai with a closed hand is a very helpful question to answer, the closed-hands-tenpai-only edge case seems a bit easier than the open-hands allowed case - I estimate 1 week (~5 hrs)
- Likelihood of reaching tenpai with any calls is helpful in end-game situations - I estimate 1 week (~5 hrs)

Total estimate: 2 weeks (~10 hrs)
