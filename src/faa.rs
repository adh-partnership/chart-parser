//! Aviation-related code.

use chrono::NaiveDate;
use itertools::Itertools;
use log::error;
use std::sync::LazyLock;

pub const DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Debug, Clone)]
pub struct Airac {
    pub code: u16,
    pub start: NaiveDate,
}

impl Airac {
    fn new(code: u16, start: &str) -> Self {
        Self {
            code,
            start: NaiveDate::parse_from_str(start, DATE_FORMAT).unwrap(),
        }
    }
}

// TODO I guess I'll have to calculate end dates for these
// or put the end dates into the struct itself?

/// Static collection of AIRAC cycle info.
///
/// Sourced from [Wikipedia].
///
/// [Wikipedia]: https://en.wikipedia.org/wiki/Aeronautical_Information_Publication
static AIRAC_DATES: LazyLock<Vec<Airac>> = LazyLock::new(|| {
    vec![
        // 2024
        Airac::new(2401, "2024-01-25"),
        Airac::new(2402, "2024-02-22"),
        Airac::new(2403, "2024-03-21"),
        Airac::new(2404, "2024-04-18"),
        Airac::new(2405, "2024-05-16"),
        Airac::new(2406, "2024-06-13"),
        Airac::new(2407, "2024-07-11"),
        Airac::new(2408, "2024-08-08"),
        Airac::new(2409, "2024-09-05"),
        Airac::new(2410, "2024-10-03"),
        Airac::new(2411, "2024-10-31"),
        Airac::new(2412, "2024-11-28"),
        Airac::new(2413, "2024-12-26"),
        // 2025
        Airac::new(2501, "2025-01-23"),
        Airac::new(2502, "2025-02-20"),
        Airac::new(2503, "2025-03-20"),
        Airac::new(2504, "2025-04-17"),
        Airac::new(2505, "2025-05-15"),
        Airac::new(2506, "2025-06-12"),
        Airac::new(2507, "2025-07-12"),
        Airac::new(2508, "2025-08-07"),
        Airac::new(2509, "2025-09-04"),
        Airac::new(2510, "2025-10-02"),
        Airac::new(2511, "2025-10-30"),
        Airac::new(2512, "2025-11-27"),
        Airac::new(2513, "2025-12-25"),
        // 2026
        Airac::new(2601, "2026-01-22"),
        Airac::new(2602, "2026-02-19"),
        Airac::new(2603, "2026-03-19"),
        Airac::new(2604, "2026-04-16"),
        Airac::new(2605, "2026-05-14"),
        Airac::new(2606, "2026-06-11"),
        Airac::new(2607, "2026-07-09"),
        Airac::new(2608, "2026-08-06"),
        Airac::new(2609, "2026-09-03"),
        Airac::new(2610, "2026-10-01"),
        Airac::new(2611, "2026-10-29"),
        Airac::new(2612, "2026-11-26"),
        Airac::new(2613, "2026-12-24"),
    ]
});

/// Return `true` if the passed date is exactly
/// an AIRAC cycle update date.
pub fn is_cycle_date(date: &str) -> bool {
    AIRAC_DATES
        .iter()
        .any(|airac| airac.start.format(DATE_FORMAT).to_string() == date)
}

/// Get the cycle code for the supplied date.
///
/// `None` is returned if the date falls outside the bounds
/// of the static data.
pub fn get_cycle_for(date: &str) -> Option<Airac> {
    // parse the String into a date for use in comparison
    let dt = match NaiveDate::parse_from_str(date, DATE_FORMAT) {
        Ok(dt) => dt,
        Err(e) => {
            error!("Error parsing date string \"{date}\": {e}");
            return None;
        }
    };
    /*
        The code that should be returned, if any, is the code for the AIRAC
        date that the supplied date is after, but only just. Using tuples
        (i.e. iter windows), the supplied date should be after the first
        tuple item but before the second.
    */
    for (a, b) in AIRAC_DATES.iter().tuples() {
        if a.start <= dt && dt <= b.start {
            return Some(a.to_owned());
        }
    }
    None
}
