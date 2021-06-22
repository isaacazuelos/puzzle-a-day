//! Game state is stored as a collection of bit masks which track which spots
//! are filled by pieces.

// It would be nice to go and add some `debug_assert` checks into `Mask::set`
// and `Mask::get`, but we can't yet `panic!` on a failed assertion while
// they're `const`. I wanted [`Mask::FRAME`] to be `const` so I went that route.
//
// Most of the operations on masks are `#[inline]` (which the compiler would
// probably do anyway) because the whole operation will probably fit in
// registers while they're a lot of instructions, they're all quick and linear.

use std::ops::{BitAnd, BitOr};

/// A mask is an 8x8 bit board.
///
/// Bit 0 is the top left, progressing in English reading order.
///
/// This is used to represent how pieces might overlap, and quickly test for
/// collisions.
#[derive(Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct Mask(u64);

impl Mask {
    /// The number of columns on the board.
    pub const WIDTH: usize = 8;

    /// The number of rows in the board.
    pub const HEIGHT: usize = 8;

    /// A mask with no squares set.
    pub const BLANK: Mask = Mask(0);

    /// Get a bit at a specific row and column.
    #[inline]
    pub const fn get(&self, row: usize, column: usize) -> bool {
        let bit = 1 << (row * 8 + column);

        (self.0 & bit) != 0
    }

    /// Set a bit (i.e. the bit is 1) at a specific row and column.
    #[inline]
    pub const fn set(self, row: usize, column: usize) -> Mask {
        let bit = 1 << (row * 8 + column);
        Mask(self.0 | bit)
    }

    /// Translate the bits right and down by some number of pieces.
    ///
    /// Note that this will result in incorrect results if it any set bits would
    /// be pushed off the board.
    #[inline]
    pub const fn translate(self, right: usize, down: usize) -> Mask {
        Mask(self.0 << (down * 8 + right))
    }

    /// Flip a mask vertically, along the horizontal axis between the middle
    /// rows of the board.
    ///
    /// Here's a ASCII map to help visualize what happens.
    ///
    /// ```txt
    /// 1 2 3    7 8 9
    /// 4 5 6 -> 4 5 6
    /// 7 8 9    1 2 3
    /// ```
    #[inline]
    const fn flip_vertical(self) -> Mask {
        Mask(self.0.swap_bytes())
    }

    /// Flip the board along the axis from the top left to the bottom right,
    /// like a matrix transpose in linear algebra.
    ///
    /// We could have flipped the boards in a few different ways (vertically,
    /// horizontally, etc.), but I did this one so that so that when we do it to
    /// a [`crate::piece::Piece`] it always swaps the handedness of chiral
    /// pieces.
    ///
    /// Here's a ASCII map to help visualize what happens.
    ///
    /// ```txt
    /// 1 2 3    1 4 7
    /// 4 5 6 -> 2 5 8
    /// 7 8 9    3 6 9
    /// ```
    ///
    /// The implementation is from the [chess programming wiki][cpw] as
    /// `flipDiagA8H1`.
    ///
    /// [cpw]: https://www.chessprogramming.org/Flipping_Mirroring_and_Rotating
    #[rustfmt::skip]
    #[inline]
    pub const fn transpose(self) -> Mask {
        // These are constant masks used to pull out specific bytes. The wiki
        // link in the function's documentation has a breakdown of what they're
        // for.
        const K1: u64 = 0xaa00aa00aa00aa00;
        const K2: u64 = 0xcccc0000cccc0000;
        const K4: u64 = 0xf0f0f0f00f0f0f0f;

        let mut x: u64 = self.0;
        let mut t: u64;

        t  =       x ^ (x << 36);
        x ^= K4 & (t ^ (x >> 36));
        t  = K2 & (x ^ (x << 18));
        x ^=       t ^ (t >> 18);
        t  = K1 & (x ^ (x <<  9));
        x ^=       t ^ (t >>  9);

        Mask(x)
    }

    /// Rotate the board 90 degrees clockwise.
    ///
    /// This is done by flipping the board vertically, then taking the
    /// transpose. This is a fun linear algebra trick to avoid using
    /// trigonometric functions to do rotations in computer graphics.
    ///
    /// Here's a ASCII map to help visualize what happens.
    ///
    /// ```txt
    /// 1 2 3                      7 8 9                 7 4 1
    /// 4 5 6 -> flip vertical ->  4 5 6 -> transpose -> 8 5 2
    /// 7 8 9                      1 2 3                 9 6 3
    /// ```
    #[inline]
    pub const fn rotate(self) -> Mask {
        self.flip_vertical().transpose()
    }
}

// Puzzle layout specific Masks.
impl Mask {
    /// The default puzzle frame.
    ///
    ///Since the puzzle is 7x7-ish, we block off the right and bottom.
    #[rustfmt::skip]
    pub const FRAME: Mask = Mask(0)
        .set(0, 6).set(0, 7)
        .set(1, 6).set(1, 7)
        .set(2, 7)
        .set(3, 7)
        .set(4, 7)
        .set(5, 7)
        .set(6, 3).set(6, 4).set(6, 5).set(6, 6).set(6, 7)
        .set(7, 0).set(7, 1).set(7, 2).set(7, 3)
        .set(7, 4).set(7, 5).set(7, 6).set(7, 7);

    /// Create a [`Mask`] with a bit set for the specified 0-indexed month.
    ///
    /// # Panics
    ///
    /// Only months between 0 and 11 are valid
    #[inline]
    pub fn for_month(month: u32) -> Mask {
        debug_assert!(month < 12); // Using `<` because it's 0-indexed.

        let index = if month < 6 { month } else { month - 6 + 8 };
        Mask(1 << index)
    }

    /// Create a [`Mask`] with a bit set for the specified 0-indexed day.
    ///   
    /// # Panics
    ///
    /// Only days between 0 and 30 are valid.
    #[inline]
    pub fn for_day(day: u32) -> Mask {
        debug_assert!(day < 31); // Using `<` because it's 0-indexed.

        let column = (day % 7) as usize;
        let row = (2 + day / 7) as usize; // 2 for the month rows
        Mask(0).set(row, column)
    }
}

impl BitAnd<Mask> for Mask {
    type Output = Mask;

    #[inline]
    fn bitand(self, rhs: Mask) -> Mask {
        Mask(self.0 & rhs.0)
    }
}

impl BitOr<Mask> for Mask {
    type Output = Mask;

    #[inline]
    fn bitor(self, rhs: Mask) -> Mask {
        Mask(self.0 | rhs.0)
    }
}

// We don't need to print Masks in release builds, but this is useful for
// debugging and testing.
#[cfg(not(feature = "release"))]
mod not_release {
    use super::Mask;

    #[cfg(not(feature = "release"))]
    impl std::fmt::Debug for Mask {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            // https://www.youtube.com/watch?v=Svd9qMlV9wU
            write!(f, "Mask({:064b})", self.0)
        }
    }

    #[cfg(not(feature = "release"))]
    impl std::fmt::Display for Mask {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            for r in 0..8 {
                for c in 0..8 {
                    let c = if self.get(r, c) { '•' } else { '-' };
                    write!(f, "{}", c)?;
                }
                writeln!(f)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // There aren't tests for the std::ops implementations because they're so
    // straight-forward.

    #[test]
    fn get() {
        assert!(Mask(1).get(0, 0));
        assert!(!Mask(1).get(0, 1));
    }

    #[test]
    fn set() {
        assert_eq!(Mask(0).set(0, 0).set(0, 1).set(0, 2).set(0, 3), Mask(0xF));
        assert_eq!(Mask(0).set(1, 0), Mask(0x100));
    }

    #[test]
    fn translate() {
        let mask = Mask(0).set(1, 0).set(1, 5);
        let after = Mask(0).set(3, 2).set(3, 7);
        assert_eq!(mask.translate(2, 2), after);
    }

    #[test]
    fn flip_vertical() {
        // This pattern is from the chess programming wiki link.
        let mask = Mask(0xF0F0F0F00F0F0F0F);
        let after = Mask(0x0F0F0F0FF0F0F0F0);
        assert_eq!(mask.flip_vertical(), after);
    }

    #[test]
    fn transpose() {
        // This pattern is from the chess programming wiki link.
        // 0xC == 1100, so transposed it's 0011 = 0x3.
        let mask = Mask(0xCCCC0000CCCC0000);
        let after = Mask(0x0000333300003333);
        assert_eq!(mask.transpose(), after);
    }

    #[test]
    fn rotate() {
        // This pattern is from the chess programming wiki link.
        let mask = Mask(0xAA00AA00AA00AA00);
        let after = Mask(0x00AA00AA00AA00AA);
        assert_eq!(mask.rotate(), after);
    }

    // puzzle specific impl section

    #[test]
    fn for_month() {
        assert_eq!(Mask::for_month(0), Mask(1));
        assert_eq!(Mask::for_month(5), Mask(0x020), "pick the right column");
        assert_eq!(Mask::for_month(6), Mask(0x100), "didn't wrap correctly");
    }

    #[test]
    fn for_day() {
        assert_eq!(
            Mask::for_day(0),
            Mask(0).set(2, 0),
            "didn't skip month rows"
        );
        assert_eq!(
            Mask::for_day(30),
            Mask(0).set(6, 2),
            "didn't wrap correctly"
        );
    }

    #[test]
    fn date_of_writing() {
        // Today's not working, so I'm making it a test
        let date = Mask::for_day(18) | Mask::for_month(5);
        let expected = Mask(0).set(0, 5).set(4, 4);
        assert_eq!(date, expected);
    }
}
