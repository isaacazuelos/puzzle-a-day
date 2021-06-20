# A-Puzzle-A-Day Solver

A solver for [DragonFjord's A-Puzzle-A-Day][puzzle].

[puzzle]: https://www.dragonfjord.com/product/a-puzzle-a-day/

## Install

Clone this repository with `git` and use `cargo` to build and install.

Development targeted stable 1.53, but older versions may work.

```sh
git clone https://github.com/isaacazuelos/puzzle-a-day
cd zenith
cargo install --path=.
```

## Usage

Run the script with no arguments to solve for today's date, or with `--date` to
specify a date in [ISO 8601][date-format].

[date-format][https://en.wikipedia.org/wiki/ISO_8601]

See the `--help` for more information.

## License

The source code's under [MIT](https://choosealicense.com/licenses/mit/) which is
in the included `LICENSE` file, but I'm not going to pretend to understand the
legality of working on a solver for a commercial puzzle.
