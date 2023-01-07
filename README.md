# simple_minesweeper
A 0 dependency, prompt based, simple minesweeper clone for the terminal.

## Installation
First ensure you have Rust installed (best to use rustup).

Then:
```
git clone https://github.com/auronandace/simple_minesweeper
cd simple_minesweeper
cargo install --path .
```

## Playing
Launch `simple_minesweeper` in a terminal and follow the on-screen prompts. No arguments are required when launching the program.

First you set the width and the height of the minefield. Minimum is 2 and maximum is 26. Each row and column is marked by a letter of the alphabet. Rows are lowercase letters and columns are uppercase letters.

To open a square you type the lowercase letter o followed by a space and the co-ordinate of the square (example: `o aA`). To flag a square you type a lowercase f followed by a space and the co-ordinate of the square (example: `f bB`). To quit you simply type a lowercase q.

When all mines have been flagged and all remaining squares opened you have won the game.
