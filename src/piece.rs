//! Descriptions of individual game pieces, and how they can be positioned on
//! the game board.

// TODO: If we position [`Piece::base_mask`] so that their charity is flipped by
//       [`Mask::vertical_flip`], we can use 1- or 2-instruction flips instead
//       of the 20-some instruction [`Mask::transpose`] in [`Piece::positions`]
//       (with some reworking) to reduce the number of transposes, which are
//       relatively costly mask operations. We still have to transpose twice for
//       rotations at least, if we use bit reverse for the 180 rotation.

use lazy_static::lazy_static;

use crate::mask::Mask;

/// Each type of piece that can fit on the board.
///
/// Pieces are loosely named after letters that look like them. I had to go to
/// non-English alphabets for some of the shapes, even still [`Piece::T`] is a
/// bit of a stretch.
///
/// These are just the names of the pieces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    C,
    Gamma,
    L,
    Lamedh,
    O,
    P,
    T,
    Z,
}

lazy_static! {
    static ref POSITIONS: [Vec<Mask>; Piece::COUNT] = [
        Piece::C.calculate_positions(),
        Piece::Gamma.calculate_positions(),
        Piece::L.calculate_positions(),
        Piece::Lamedh.calculate_positions(),
        Piece::O.calculate_positions(),
        Piece::P.calculate_positions(),
        Piece::T.calculate_positions(),
        Piece::Z.calculate_positions(),
    ];
}

impl Piece {
    /// The number of different types of pieces.
    pub const COUNT: usize = 8;

    /// An array containing each piece.
    ///
    /// The pieces are in order, so `ALL[piece as usize] == piece`.
    pub const ALL: [Piece; Piece::COUNT] = {
        use Piece::*;
        [C, Gamma, L, Lamedh, O, P, T, Z]
    };

    /// Each possible position on the board the piece could be placed.
    ///
    /// This includes each rotation, and flipped over if the piece is chiral
    /// (see [`Piece::is_chiral`]).
    pub fn positions(&self) -> &[Mask] {
        &POSITIONS[*self as usize]
    }

    /// Calculates each possible position that a piece could be in on the board.
    ///
    /// This is used to populate the [`POSITIONS`] tables used by the solver.
    fn calculate_positions(self) -> Vec<Mask> {
        let mut positions = Vec::new();
        let (width, height) = self.size();
        let mask = self.base_mask();

        // We need to translate the piece around to each place it could fit.
        for right in 0..=(Mask::WIDTH - width) {
            for down in 0..=(Mask::HEIGHT - height) {
                let mut translated = mask.translate(right, down);

                'rotations: for i in 0..4 {
                    positions.push(translated);
                    if self.is_chiral() {
                        // It seems weird to transpose the whole board
                        // instead of the piece, but since we're doing it
                        // for every position, we still get complete board
                        // coverage.
                        positions.push(translated.transpose())
                    }

                    // We rotate it for the next iteration of the
                    // `'rotations` loop to save. We can skip the rotation
                    // on the last run through
                    if i == 3 {
                        break 'rotations;
                    } else {
                        translated = translated.rotate();
                    }
                }
            }
        }

        // We sort position masks by their bits to (loosely) push them into the
        // top right. This should speed up searching by ruling out a lot of
        // collisions early.
        //
        // In my extremely unscientific test, commenting this out nearly doubles
        // running time.
        positions.sort();
        positions.dedup();
        positions
    }

    /// Produces a mask which looks like the Piece, positioned at the top-left
    /// of the board.
    const fn base_mask(self) -> Mask {
        // If you change these, be sure to update [`Piece::size`]!
        match self {
            Piece::C => Mask::BLANK
                .set(0, 0) // •••
                .set(0, 1) // •-•
                .set(0, 2)
                .set(1, 0)
                .set(1, 2),

            Piece::Gamma => Mask::BLANK
                .set(0, 0) // •••
                .set(0, 1) // •--
                .set(0, 2) // •--
                .set(1, 0)
                .set(2, 0),

            Piece::L => Mask::BLANK
                .set(0, 0) // •-
                .set(1, 0) // •-
                .set(2, 0) // •-
                .set(3, 0) // ••
                .set(3, 1),

            Piece::Lamedh => Mask::BLANK
                .set(0, 0) // •-
                .set(1, 0) // •-
                .set(2, 0) // ••
                .set(2, 1) // -•
                .set(3, 1),

            Piece::O => Mask::BLANK
                .set(0, 0) // •••
                .set(0, 1) // •••
                .set(0, 2)
                .set(1, 0)
                .set(1, 1)
                .set(1, 2),

            Piece::P => Mask::BLANK
                .set(0, 0) // •••
                .set(0, 1) // ••-
                .set(0, 2)
                .set(1, 0)
                .set(1, 1),

            Piece::T => Mask::BLANK
                .set(0, 0) // •-
                .set(1, 0) // •-
                .set(2, 0) // ••
                .set(2, 1) // •-
                .set(3, 0),

            Piece::Z => Mask::BLANK
                .set(0, 0) // ••-
                .set(0, 1) // -•-
                .set(1, 1) // -••
                .set(2, 1)
                .set(2, 2),
        }
    }

    /// The size of the box that can contain the piece's [`Piece::base_mask`],
    /// as a tuple of `(width, height)`.
    ///
    /// This is used to know how much we can translate the piece around the
    /// board before it's out of bounds.
    const fn size(self) -> (usize, usize) {
        match self {
            Piece::C => (3, 2),
            Piece::Gamma => (3, 3),
            Piece::L => (2, 4),
            Piece::Lamedh => (2, 4),
            Piece::O => (3, 2),
            Piece::P => (3, 2),
            Piece::T => (2, 4),
            Piece::Z => (3, 3),
        }
    }

    /// Is the piece [chiral][]? A piece is chiral if it is not the same as its
    /// mirror image, even if you rotate it.
    ///
    /// We need to consider all positions on the board a piece could fit in, but
    /// we don't want to consider the same position twice. If a piece is
    /// _chiral_ we need to consider flipping the piece as well as rotating
    /// it.
    ///
    /// [chiral]: https://en.wikipedia.org/wiki/Chirality_(mathematics)
    pub const fn is_chiral(self) -> bool {
        !matches!(self, Piece::C | Piece::O | Piece::Gamma)
    }

    /// The piece name as a single-character letter.
    pub const fn display_character(self) -> char {
        match self {
            Piece::C => 'C',
            Piece::Gamma => 'Γ',
            Piece::L => 'L',
            Piece::Lamedh => 'ל',
            Piece::O => 'O',
            Piece::P => 'P',
            Piece::T => 'T',
            Piece::Z => 'Z',
        }
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display_character())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Some of the Piece methods are pretty simple `match` lookups, so tests
    // don't make a lot of sense. This is the case for `base_mask`, `size`,
    // `is_chiral`, and `display_character`.

    #[test]
    fn check_positions() {
        // rotation, translation
        // --------
        // --------
        // --------
        // --------
        // ----•---
        // --•••---
        // --•-----
        // --------
        let mask = Mask::BLANK
            .set(4, 4)
            .set(5, 2)
            .set(5, 3)
            .set(5, 4)
            .set(6, 2);
        assert!(Piece::Z.positions().contains(&mask));

        // chiral flip test
        // ••------
        // •-------
        // •-------
        // •-------
        // --------
        // --------
        // --------
        // --------
        let mask2 = Mask::BLANK
            .set(0, 0)
            .set(0, 1)
            .set(1, 0)
            .set(2, 0)
            .set(3, 0);
        assert!(Piece::L.positions().contains(&mask2));
    }

    #[test]
    fn all() {
        for piece in Piece::ALL {
            assert_eq!(piece, Piece::ALL[piece as usize]);
        }
    }
}
