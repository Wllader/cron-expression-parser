use super::*;

#[test]
fn new_accepts_bounds() {
    assert_eq!(Minute::new(0).get(), 0);
    assert_eq!(Minute::new(59).get(), 59);
    assert_eq!(DayOfMonth::new(1).get(), 1);
    assert_eq!(DayOfMonth::new(31).get(), 31);
}

#[test]
#[should_panic(expected = "Value out of range")]
fn new_panics_out_of_range() {
    Minute::new(60);
}

#[test]
fn try_from_checks_range() {
    assert!(Minute::try_from(59).is_ok());
    assert!(matches!(Minute::try_from(60), Err(cron::Error::OutOfRange)));
    assert!(matches!(Hour::try_from(24), Err(cron::Error::OutOfRange)));
    assert!(matches!(DayOfMonth::try_from(0), Err(cron::Error::OutOfRange)));
    assert!(matches!(Month::try_from(0), Err(cron::Error::OutOfRange)));
    assert!(matches!(DayOfWeek::try_from(7), Err(cron::Error::OutOfRange)));
}

#[test]
fn trait_consts_match_definitions() {
    assert_eq!((<Minute as TimeElement>::MIN, <Minute as TimeElement>::MAX), (0, 59));
    assert_eq!((<Hour as TimeElement>::MIN, <Hour as TimeElement>::MAX), (0, 23));
    assert_eq!((<DayOfMonth as TimeElement>::MIN, <DayOfMonth as TimeElement>::MAX), (1, 31));
    assert_eq!((<Month as TimeElement>::MIN, <Month as TimeElement>::MAX), (1, 12));
    assert_eq!((<DayOfWeek as TimeElement>::MIN, <DayOfWeek as TimeElement>::MAX), (0, 6));
}

#[test]
fn month_lengths() {
    let expected = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (i, days) in expected.into_iter().enumerate() {
        assert_eq!(Month::raw_max_days(i as u8 + 1), days);
        assert_eq!(Month::new(i as u8 + 1).max_days(), days);
    }
}

#[test]
fn display_and_debug() {
    assert_eq!(Minute::new(7).to_string(), "7");
    assert_eq!(format!("{:?}", Minute::new(7)), "Minute(7)");
}
