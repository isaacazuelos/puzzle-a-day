//! Game state is stored as a collection of bit masks which track which spots
//! are filled by pieces.

// It would be nice to go and add some `debug_assert` checks into `Mask::set`
// and `Mask::get`, but we can't yet `panic!` on a failed assertion while
// they're `const`. I wanted [`Mask::FRAME`] to be `const` so I went that route.

use std::ops::{BitAnd, BitOr, Not, Sub};

/// A mask is an 8x8 bit board.
///
/// Bit 0 is the top left, progressing in English reading order.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Mask(u64);

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
        .set(6, 3).set(6,4).set(6, 5).set(6, 6).set(6, 7)
        .set(7, 0).set(7, 1).set(7, 2).set(7, 3)
        .set(7, 4).set(7, 5).set(7, 6).set(7, 7);

    /// Get a bit at a specific row and column.
    pub const fn get(&self, row: usize, column: usize) -> bool {
        let bit = 1 << (row * 8 + column);

        (self.0 & bit) != 0
    }

    /// Set a bit (i.e. the bit is 1) at a specific row and column.
    pub const fn set(&self, row: usize, column: usize) -> Mask {
        let bit = 1 << (row * 8 + column);
        Mask(self.0 | bit)
    }

    /// Create a [`Mask`] with a bit set for the specified 0-indexed month.
    pub fn for_month(month: u32) -> Mask {
        debug_assert!(month < 12); // Using `<` because it's 0-indexed.

        let index = if month < 6 { month } else { month - 6 + 8 };
        Mask(1 << index)
    }

    /// Create a [`Mask`] with a bit set for the specified 0-indexed day.
    ///
    /// Only days between 0 and 30 are valid.
    pub fn for_day(day: u32) -> Mask {
        debug_assert!(day < 31); // Using `<` because it's 0-indexed.

        let column = (day % 7) as usize;
        let row = (2 + day / 7) as usize; // 2 for the month rows
        Mask(0).set(row, column)
    }

    /// Clear the bit (i.e. the bit is 0) at a specific row and column.
    #[allow(dead_code)]
    pub fn clear(&self, row: usize, column: usize) -> Mask {
        debug_assert!(row < 8);
        debug_assert!(column < 8);

        let bit = 1 << (row * 8 + column);
        Mask(self.0 & !bit)
    }
}

impl BitAnd<Mask> for Mask {
    type Output = Mask;

    fn bitand(self, rhs: Mask) -> Mask {
        Mask(self.0 & rhs.0)
    }
}

impl BitOr<Mask> for Mask {
    type Output = Mask;

    fn bitor(self, rhs: Mask) -> Mask {
        Mask(self.0 | rhs.0)
    }
}

impl Not for Mask {
    type Output = Mask;

    fn not(self) -> Self::Output {
        Mask(!self.0)
    }
}

impl Sub for Mask {
    type Output = Mask;

    fn sub(self, rhs: Self) -> Self::Output {
        Mask(self.0 & !rhs.0)
    }
}

impl std::fmt::Debug for Mask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // https://www.youtube.com/watch?v=Svd9qMlV9wU
        write!(f, "Mask({:064b})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // There aren't tests for the std::ops impls because they're so
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
    fn clear() {
        assert_eq!(Mask(0).set(3, 3).clear(3, 3), Mask(0));
    }

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
}
