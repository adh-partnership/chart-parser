use std::cmp::Ordering;

// Find if provided date is a an interval of 28 days from 2023-01-26
pub fn is_cycle_date(date: &str) -> bool {
    let known_cycle_date = "2023-01-26";
    let date = date.parse::<chrono::NaiveDate>().unwrap();
    let known_cycle_date = known_cycle_date.parse::<chrono::NaiveDate>().unwrap();
    let diff = date.signed_duration_since(known_cycle_date).num_days();
    diff % 28 == 0
}

pub fn first_cycle_of_year(year: i64) -> String {
    let mut new_date = chrono::NaiveDate::from_ymd_opt(i32::try_from(year).unwrap(), 1, 1).unwrap();
    while !is_cycle_date(new_date.to_string().as_str()) {
        new_date += chrono::Duration::days(1);
    }
    new_date.to_string()
}

pub fn number_of_cycles_in_year(year: i64) -> i64 {
    let first_cycle = first_cycle_of_year(year);
    let next_year_first_cycle = first_cycle_of_year(year + 1);

    return cycles_between_dates(first_cycle.as_str(), next_year_first_cycle.as_str());
}

pub fn cycles_between_dates(date1: &str, date2: &str) -> i64 {
    let date1 = date1.parse::<chrono::NaiveDate>().unwrap();
    let date2 = date2.parse::<chrono::NaiveDate>().unwrap();
    let diff = date2.signed_duration_since(date1).num_days();
    diff / 28 + 1
}

fn split_year(year: &i64) -> i32 {
    let year = year.to_string();
    let year = year.split_at(2);
    year.1.parse::<i32>().unwrap()
}

pub fn calculate_cycle(date: &str) -> Option<String> {
    // Find out if date is before get_first_cytcle_of_year
    // Extract year from date
    let year = date.split('-').collect::<Vec<&str>>()[0].parse::<i64>();
    if year.is_err() {
        return None;
    }
    let year = year.unwrap();

    match date.cmp(first_cycle_of_year(year).as_str()) {
        Ordering::Less => {
            let year = year - 1;
            Some(format!(
                "{:02}{:02}",
                split_year(&year),
                number_of_cycles_in_year(year)
            ))
        }
        Ordering::Equal => Some(format!("{:02}{:02}", split_year(&year), 1)),
        Ordering::Greater => Some(format!(
            "{:02}{:02}",
            split_year(&year),
            cycles_between_dates(first_cycle_of_year(year).as_str(), date)
        )),
    }
}

pub fn calculate_cycle_dates(cycle: &str) -> (String, String) {
    let year = format!("20{}", &cycle[0..2]);
    let cycle_num = &cycle[2..4];

    let mut new_date = chrono::NaiveDate::from_ymd_opt(year.parse::<i32>().unwrap(), 1, 1).unwrap();
    while !is_cycle_date(new_date.to_string().as_str()) {
        new_date = new_date.succ_opt().unwrap();
    }

    let cycle_num = cycle_num.parse::<i64>().unwrap();
    new_date += chrono::Duration::days((cycle_num - 1) * 28);
    let cycle_end = new_date + chrono::Duration::days(27);

    (new_date.to_string(), cycle_end.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests() {
        assert_eq!(is_cycle_date("2023-12-28"), true);
        assert_ne!(is_cycle_date("2023-12-27"), true);

        assert_eq!(first_cycle_of_year(2023), "2023-01-26");

        assert_eq!(number_of_cycles_in_year(2023), 13);
        assert_eq!(number_of_cycles_in_year(2020), 14);

        assert_eq!(cycles_between_dates("2023-01-26", "2023-02-22"), 1);
        assert_eq!(cycles_between_dates("2023-01-26", "2023-02-23"), 2); // technically, 2023-02-23 is the start of cycle 2

        let year = 2023;
        assert_eq!(split_year(&year), 23);

        assert_eq!(calculate_cycle("2023-01-26").unwrap(), "2301");
        assert_eq!(calculate_cycle("2023-01-01").unwrap(), "2213");
        assert_eq!(calculate_cycle("invaliddate"), None);

        let cycle = "2301".to_string();
        assert_eq!(
            calculate_cycle_dates(&cycle),
            ("2023-01-26".to_string(), "2023-02-22".to_string())
        );
    }
}
