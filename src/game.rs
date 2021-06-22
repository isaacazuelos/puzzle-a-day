//! Representation of game state.
//!
//! Part of the plan was realizing that there are only 8 pieces, with fewer than
//! 64 possible placements for each of 4 rotated orientation, and 2 possible
//! flips. That means there are fewer than 8 * 64 * 4 * 2 = 4096 possible masks
//! we need to choose 8 of to solve. That's not a large search space for a
//! computer.

use crate::mask::Mask;
use crate::piece::Piece;

/// Game state is represented as a collection of [`Mask`]s.
///
/// Each [`Piece`] can only be placed once.
///
/// No bit is set in more than one of the [`Game::piece`] masks.
pub struct Game {
    /// The squares where we can't put pieces, because they mark the date we're
    /// trying to solve for.
    date: Mask,

    /// The position of each piece, if placed. Each [`Piece`] is used `as usize`
    /// to index this and get the [`Mask`] marking it's position on the board. A
    /// piece isn't placed if it's corresponding mask is [`Mask::BLANK`].
    pieces: [Mask; Piece::COUNT],

    /// A mask showing all placed pieces, used to check for collisions when
    /// trying to put down more pieces when solving.
    ///
    /// This is stored instead of doing a bitwise or on each piece in
    /// [`Game::pieces`] as that actually ended up being a significant amount of
    /// the program's execution time in profiling.
    placed: Mask,

    /// The index of the next piece in [`Piece::ALL`].
    next_piece_index: usize,
}

impl Game {
    /// The character used for displaying which cells are filled by the frame.
    const FRAME_DISPLAY: char = ' ';

    /// The character used for displaying cells reserved for the date we're
    /// solving for.
    const DATE_DISPLAY: char = '•';

    /// The character used for displaying cells which aren't filled.
    const BLANK_DISPLAY: char = '-';

    /// Create a new [`Game`] with the given date marked off. The `month` and
    /// `day` should be zero-indexed and reasonable (i.e. no 32nd day of the
    /// 15th month).
    pub fn for_date(month: u32, day: u32) -> Game {
        let date = Mask::for_day(day) | Mask::for_month(month);

        Game {
            date,
            pieces: [Mask::BLANK; 8],
            placed: date | Mask::FRAME,
            next_piece_index: 0,
        }
    }

    /// A recursive, depth-first search to solve the game board.
    pub fn solve(&mut self) {
        if self.next_piece_index < Piece::COUNT {
            let piece = Piece::ALL[self.next_piece_index];
            self.next_piece_index += 1;

            for position in piece.positions() {
                if self.place(piece, *position) {
                    self.solve();

                    if self.all_pieces_placed() {
                        return;
                    } else {
                        self.remove(piece);
                    }
                }
            }

            self.next_piece_index -= 1;
        }
    }

    /// Have all pieces been placed?
    ///
    /// Since each piece can only be placed once, and [`Game::place`] checks for
    /// collisions before placing, we know that if all pieces are placed the
    /// game board is solved.
    fn all_pieces_placed(&self) -> bool {
        self.placed == Mask::FULL
    }

    /// Places the piece in the position given, if there's room to do so.
    /// Returns `true` if the piece was placed, and `false` if it could not be
    /// placed.
    fn place(&mut self, piece: Piece, position: Mask) -> bool {
        if (position & self.placed) == Mask::BLANK {
            self.placed |= position;
            self.pieces[piece as usize] = position;
            true
        } else {
            false
        }
    }

    /// Remove a piece from board.
    fn remove(&mut self, piece: Piece) {
        self.placed -= self.pieces[piece as usize];
        self.pieces[piece as usize] = Mask::BLANK;
    }

    /// The character to use to display a particular row and column of the board
    /// when rendering to the terminal, mostly used by the [`std::fmt::Display`]
    /// `impl`.
    fn display_character(&self, row: usize, column: usize) -> char {
        if Mask::FRAME.get(row, column) {
            return Game::FRAME_DISPLAY;
        }

        if self.date.get(row, column) {
            return Game::DATE_DISPLAY;
        }

        for piece in Piece::ALL {
            if self.pieces[piece as usize].get(row, column) {
                return piece.display_character();
            }
        }

        Game::BLANK_DISPLAY
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for r in 0..7 {
            for c in 0..7 {
                write!(f, "{}", self.display_character(r, c))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_date() {
        let game = Game::for_date(11, 24);
        assert_eq!(game.date, Mask::BLANK.set(1, 5).set(5, 3));
    }

    #[test]
    fn place() {
        let mut game = Game::for_date(11, 24);
        let piece = Piece::ALL[0];
        let positions = piece.positions();

        // game should be blank, no piece is too big to fit in the top right on
        // christmas.
        assert!(game.place(piece, positions[0]));
    }

    #[test]
    fn collide() {
        let mut game = Game::for_date(11, 24);
        assert!(!game.place(Piece::C, Mask::FRAME));
    }

    #[test]
    fn remove() {
        let mut game = Game::for_date(11, 24);
        let piece = Piece::ALL[0];
        let position = piece.positions()[0];

        // game should be blank, no piece is too big to fit in the top right on
        // christmas.
        assert!(game.place(piece, position));
        game.remove(piece);
        assert!(game.pieces[piece as usize] == Mask::BLANK);
    }

    #[test]
    fn solve_test() {
        // Solving takes time in debug builds, so we try to cram a lot of tests
        // in here.
        let mut game = Game::for_date(11, 24);

        game.solve();

        assert!(game.all_pieces_placed());
    }
}
