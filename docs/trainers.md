# trainers

high-level overview of our bot trainers and their bot training strategies

## bots

### exploration range

bots have an exploration range. it defines how much a move can deviate from the maximum

### bot memory

bots remember their moves as a list of weights. there is one weight per column, which is to say 7 weights in total. they associate their list of weights with a board, which is to say there is one board per list of weights. when picking a move, it looks up the weights for that board state, and picks a random column who's weight is within the exploration range.

to calculate which columns are within the exploration rate, it takes the max weight between the columns, i.e. the optimal choice, and filters away any columns which are below `max_weight - exploration_rate`, i.e. `weight >= max_weight - exploration_rate`

### played choices

in order to update the weights, as the bot plays, it saves their choices

at the end of the match, the bot then applies a reinforcement to the weights depending on the current board state. 

the reinforcement is based on whether:

- it won (+10)
- it lost (-10)
- it tied, while placing the starting brick (-1)
- it tied, while not placing the starting brick (+1)

and the board value after the move was played

### board value

board value is calculated based on:

- if the player has won (+10_000)
- if the player has lost (-10_000)
- how many opportunities the opponent has (-1 per opportunity)
- and how many opportunities the player has (+1 per opportunity)

what do we mean by opportunities? well, take this board as an example:

```
|       |
|       |
|       |
|X  O OO|
|X OX OX|
```

the opportunities are calculated by looking at each piece of the board, and going through this algorithm:

```
_________________________
| is the spot occupied? |
'"""""""""""""""""""""""'
   /               \
______           _______
| no |           | yes |
'""""'           '"""""'
  |                 |
_____     ________________________
| 0 |     | count_opportunites() |
'"""'     '""""""""""""""""""""""'
```

to count the opportunites for i.e., this chip `0`:

```
|       |
|       |
|       |
|X  0 OO|
|X OX OX|
```

we check a stripe with a offset of `[-3; 0]` in the four directions `[1, 1], [1, 0], [0, 1], [-1, 1]`, and a length of 4, which looks as follows:

directions: 

```
[ ][ ][ ]
   [x][ ]
```

expanded: 

```
[ ]      [ ]      [ ]
   [ ]   [ ]   [ ]
      [ ][ ][ ]
[ ][ ][ ][x][ ][ ][ ]
      [ ][ ][ ]
   [ ]   [ ]   [ ]
[ ]      [ ]      [ ]
```

it checks each of the 4 permutations of a stripe:

```
[ ]      [ ]      [x]
   [ ]   [ ]   [x]
      [ ][ ][x]
[ ][ ][ ][x][ ][ ][ ]
      [ ][ ][ ]
   [ ]   [ ]   [ ]
[ ]      [ ]      [ ]

---

[ ]      [ ]      [ ]
   [ ]   [ ]   [x]
      [ ][ ][x]
[ ][ ][ ][x][ ][ ][ ]
      [x][ ][ ]
   [ ]   [ ]   [ ]
[ ]      [ ]      [ ]

---

[ ]      [ ]      [ ]
   [ ]   [ ]   [ ]
      [ ][ ][x]
[ ][ ][ ][x][ ][ ][ ]
      [x][ ][ ]
   [x]   [ ]   [ ]
[ ]      [ ]      [ ]

---

[ ]      [ ]      [ ]
   [ ]   [ ]   [ ]
      [ ][ ][ ]
[ ][ ][ ][x][ ][ ][ ]
      [x][ ][ ]
   [x]   [ ]   [ ]
[x]      [ ]      [ ]
```

in all 4 directions.

if all 4 positions in one stripe is:

- within the bounds of the playing area
- either the same chip as the origin, or a blank space

it is counted.
which means, for this chip, `0`:

```
|       |
|       |
|       |
|X  0 OO|
|X OX OX|
```

it would look like this:

```
|-  -  -|
| - - - |
|  ---  |
|X--0---|
|X -X-OX|
```

which has

```
|1  8  3|
| 2 - 4 |
|  ---  |
|X--0765|
|X -X-OX|
```

8 possible valid stripes

this is then done for each chip on the board, to calculate the board value

## minmax
