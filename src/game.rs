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
/// An invariant is that no bit is set in more than one of the masks.
pub struct Game {
    /// Our game board is smaller than a 8x8 grid. This mask outlines the
    /// boarders of our game board.
    frame: Mask,

    /// The squares where we can't put pieces, because they mark the date we're
    /// trying to solve for.
    date: Mask,

    /// The position of each piece, if placed. Each [`Piece`] is used `as usize`
    /// to index this and get the [`Mask`] marking it's position on the board. A
    /// piece isn't placed if it's corresponding mask is [`Mask::BLANK`].
    pieces: [Mask; Piece::COUNT],
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
            frame: Mask::FRAME,
            date,
            pieces: [Mask::BLANK; 8],
        }
    }

    /// A recursive, depth-first search to solve the game board.
    pub fn solve(&mut self) {
        if let Some(piece) = self.first_unplaced_piece() {
            for position in piece.positions() {
                if self.place(piece, position) {
                    self.solve();
                    if self.all_pieces_placed() {
                        return;
                    } else {
                        self.remove(piece);
                    }
                }
            }
        }
    }

    /// The first piece which is has not yet been placed on the game board, if
    /// there is one.
    fn first_unplaced_piece(&self) -> Option<Piece> {
        for piece in Piece::ALL {
            if self.pieces[piece as usize] == Mask::BLANK {
                return Some(piece);
            }
        }
        None
    }

    /// Have all pieces been placed?
    ///
    /// Since each piece can only be placed once, and [`Game::place`] checks for
    /// collisions before placing, we know that if all pieces are placed the
    /// game board is solved.
    fn all_pieces_placed(&self) -> bool {
        for piece in self.pieces {
            if piece == Mask::BLANK {
                return false;
            }
        }
        true
    }

    /// Places the piece in the position given, if there's room to do so.
    /// Returns `true` if the piece was placed, and `false` if it could not be
    /// placed.
    fn place(&mut self, piece: Piece, position: Mask) -> bool {
        let currently_filled = self.date
            | self.frame
            | self.pieces.iter().fold(Mask::BLANK, |a, b| a | *b);

        if (position & currently_filled) == Mask::BLANK {
            self.pieces[piece as usize] = position;
            true
        } else {
            false
        }
    }

    /// Remove a piece from board.
    fn remove(&mut self, piece: Piece) {
        self.pieces[piece as usize] = Mask::BLANK;
    }

    /// The character to use to display a particular row and column of the board
    /// when rendering to the terminal, mostly used by the [`std::fmt::Display`]
    /// `impl`.
    fn display_character(&self, row: usize, column: usize) -> char {
        if self.frame.get(row, column) {
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

    // TODO: write tests
}
