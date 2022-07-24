# Wave Function Collapse

This library implements a generalized wave function collapse algorithm.

# Examples

```
cargo run --release --example path

┌┼┘├┘┌┬┐
├┴─┘ ├┘└
└┬┬┐ └─┐
┬┘│├┐  │
└┬┴┘└──┼
 ├──┐ ┌┘
┬┤ ┌┼┬┼┬
└┴┐│├┤└┘

Took 0.5479 ms
```

```
cargo run --release --example sudoku

6 8 3   2 1 7   5 4 9
1 7 5   3 4 9   2 6 8
4 2 9   5 8 6   7 1 3

5 1 8   4 9 2   6 3 7
9 4 7   8 6 3   1 2 5
2 3 6   1 7 5   9 8 4

3 6 2   9 5 8   4 7 1
7 5 1   6 3 4   8 9 2
8 9 4   7 2 1   3 5 6

Took 1.7011 ms
```
