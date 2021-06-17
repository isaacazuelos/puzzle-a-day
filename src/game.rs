//! Representation of game state.

use crate::mask::Mask;

/// Game state is represented as a collection of [`Mask`]s.
///
/// An invariant is that no bit is set in more than one of the masks.
pub struct Game {
    frame: Mask,
    date: Mask,
}

impl Game {
    const FRAME_DISPLAY: char = '#';
    const DATE_DISPLAY: char = '•';
    const BLANK_DISPLAY: char = '-';

    pub fn for_date(month: u32, day: u32) -> Game {
        let date = Mask::for_day(day) & Mask::for_month(month);

        Game {
            frame: Mask::FRAME,
            date,
        }
    }

    pub fn solve(&mut self) {
        todo!("solver isn't written yet, but here's the board:\n{}", self);
    }
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for r in 0..8 {
            for c in 0..8 {
                let display_character = if self.frame.get(r, c) {
                    Game::FRAME_DISPLAY
                } else if self.date.get(r, c) {
                    Game::DATE_DISPLAY
                } else {
                    Game::BLANK_DISPLAY
                };

                write!(f, "{}", display_character)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
