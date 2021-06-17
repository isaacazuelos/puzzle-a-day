//! A solver for DragonFjord's A-Puzzle-A-Day.

mod game;
mod mask;

use std::process::exit;

// Using a full-fat date and time library is a bit overkill, but I think it's
// fun to have it correctly handle leap years and such.
use chrono::{Datelike, Local, NaiveDate};

use crate::game::Game;

const LONG_HELP: &str =
    "Solve for a specified date. The date must be formatted similar to \
     `2020-03-13`. You'd better believe we make sure that date exists in a \
     proleptic Gregorian calendar. Why do we even care about the year, I hear \
     you ask? Why, so we can check of course!";

fn main() {
    let app = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .args(&[clap::Arg::with_name("date")
            .help("solve for a specified date")
            .long_help(LONG_HELP)
            .short("d")
            .long("date")
            .takes_value(true)
            .value_name("DATE")]);

    let matches = app.get_matches();

    let date: NaiveDate = if let Some(date) = matches.value_of("date") {
        parse_date(date)
    } else {
        Local::now().naive_local().date()
    };

    let mut game = Game::for_date(date.month0(), date.day0());

    game.solve();

    println!("{}", game);
}

/// Parse a date in the correct `YYYY-MM-DD` format. There's not much the
/// program can do with an invalid date, so we just exits cleanly on invalid
/// dates.
fn parse_date(input: &str) -> NaiveDate {
    match chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        Ok(date) => date,
        Err(msg) => {
            eprintln!("cannot parse `{}` as a date because {}", input, msg);
            exit(1);
        }
    }
}
