/*
my idea for the opening book:
work similarly to first try, however use mongodb instead of plaintext to query the next move
    a query in mongodb could look *SOMETHING* like db.openings.find({"e2e4.e7e5.c1d3.etc.etc":1})
    that outputs the next best move (or most common perhaps)

also could write own pgn parser to process the data? (you know, just for fun)
 */
