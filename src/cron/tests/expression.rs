use super::*;

fn expr(s: &str) -> Result<Expression, cron::Error> {
    s.parse()
}

#[test]
fn parses_all_fields() {
    let e = expr("*/15 0 1,15 1-6 1-5").unwrap();
    assert_eq!(e.minute().to_vec(), [0, 15, 30, 45]);
    assert_eq!(e.hour().to_vec(), [0]);
    assert_eq!(e.day_of_month().to_vec(), [1, 15]);
    assert_eq!(e.month().to_vec(), [1, 2, 3, 4, 5, 6]);
    assert_eq!(e.day_of_week().to_vec(), [1, 2, 3, 4, 5]);
}

#[test]
fn all_wildcards() {
    let e = expr("* * * * *").unwrap();
    assert!(e.minute().is_wildcard());
    assert!(e.hour().is_wildcard());
    assert!(e.day_of_month().is_wildcard());
    assert!(e.month().is_wildcard());
    assert!(e.day_of_week().is_wildcard());
}

#[test]
fn extra_whitespace_is_allowed() {
    assert!(expr("  *  *\t* *   * ").is_ok());
}

#[test]
fn wrong_field_count() {
    for s in ["", "   ", "*", "* * * *", "* * * * * *"] {
        assert!(
            matches!(expr(s), Err(cron::Error::WrongFieldCount)),
            "expected WrongFieldCount for {s:?}"
        );
    }
}

#[test]
fn bad_field_values_are_reported() {
    assert!(matches!(expr("60 * * * *"), Err(cron::Error::OutOfRange)));
    assert!(matches!(expr("* 24 * * *"), Err(cron::Error::OutOfRange)));
    assert!(matches!(expr("* * 0 * *"), Err(cron::Error::OutOfRange)));
    assert!(matches!(expr("* * * 13 *"), Err(cron::Error::OutOfRange)));
    assert!(matches!(expr("* * * * 7"), Err(cron::Error::OutOfRange)));
    assert!(matches!(expr("*/0 * * * *"), Err(cron::Error::InvalidStep)));
    assert!(matches!(expr("a * * * *"), Err(cron::Error::InvalidNumber(_))));
}

#[test]
fn display_round_trips() {
    for s in ["* * * * *", "*/5 0 1,15 * 1-5", "0 12 * 1-6 0", "30 4 1,15 * *"] {
        assert_eq!(expr(s).unwrap().to_string(), s);
    }
}

#[test]
fn impossible_dates_are_rejected_when_weekday_is_wildcard() {
    assert!(matches!(expr("0 0 31 2 *"), Err(cron::Error::ImpossibleDate)));
    assert!(matches!(expr("0 0 30 2 *"), Err(cron::Error::ImpossibleDate)));
    assert!(matches!(expr("0 0 31 4 *"), Err(cron::Error::ImpossibleDate)));
    assert!(matches!(expr("0 0 31 4,6,9,11 *"), Err(cron::Error::ImpossibleDate)));
    assert!(matches!(expr("0 0 30,31 2 *"), Err(cron::Error::ImpossibleDate)));
}

#[test]
fn possible_dates_are_accepted() {
    assert!(expr("0 0 29 2 *").is_ok());
    assert!(expr("0 0 30 4 *").is_ok());
    assert!(expr("0 0 31 1 *").is_ok());
    assert!(expr("0 0 31 1,4 *").is_ok());
    assert!(expr("0 0 15,31 2 *").is_ok());
    assert!(expr("0 0 * * *").is_ok());
}

#[test]
fn restricted_weekday_allows_impossible_day_of_month() {
    assert!(expr("0 0 31 2 1").is_ok());
}

/// Straightforward reference implementation that `Expression::has_possible_date` must agree with.
fn has_possible_date_slow(e: &Expression) -> bool {
    let months = e.month().to_vec();
    let days = e.day_of_month().to_vec();
    months.iter().any(|&m| days.iter().any(|&d| d <= Month::raw_max_days(m)))
}

/// A restricted day-of-week skips the impossible-date check in `from_str`,
/// so any day-of-month/month combination can be built for comparison.
fn with_dates(days: &str, months: &str) -> Expression {
    expr(&format!("0 0 {days} {months} 1")).unwrap()
}

fn assert_fast_matches_slow(days: &str, months: &str) {
    let e = with_dates(days, months);
    assert_eq!(
        e.has_possible_date(),
        has_possible_date_slow(&e),
        "mismatch for day_of_month={days:?}, month={months:?}"
    );
}

#[test]
fn has_possible_date_matches_reference_for_every_single_pair() {
    for m in 1..=12 {
        for d in 1..=31 {
            assert_fast_matches_slow(&d.to_string(), &m.to_string());
        }
    }
}

#[test]
fn has_possible_date_matches_reference_for_sets() {
    let days = ["*", "31", "30,31", "15,31", "29-31", "1-31/10", "*/15", "28,29"];
    let months = ["*", "2", "4,6", "4,6,9,11", "1,4", "2,4", "1-6", "*/3", "11-12"];
    for d in days {
        for m in months {
            assert_fast_matches_slow(d, m);
        }
    }
}

#[test]
fn json_val_sets_lists_every_field() {
    let json = expr("5 1 1 1 1").unwrap().json_val_sets();
    assert_eq!(
        json,
        "{\n  \"minute\": [5],\n  \"hour\": [1],\n  \"dayOfMonth\": [1],\n  \"month\": [1],\n  \"dayOfWeek\": [1]\n}\n"
    );
}
