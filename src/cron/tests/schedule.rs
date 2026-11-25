mod next_since {
    use super::super::*;
    use chrono::{DateTime, TimeDelta, TimeZone, Timelike, Utc};

    fn dt(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
    }

    fn next(e: &str, since: DateTime<Utc>) -> DateTime<Utc> {
        let expr: Expression = e.parse().unwrap();
        expr.next_since(since)
            .unwrap_or_else(|| panic!("no match for {e:?} after {since}"))
    }

    #[test]
    fn result_is_strictly_after_since() {
        assert_eq!(next("*/15 * * * *", dt(2025, 1, 1, 12, 0)), dt(2025, 1, 1, 12, 15));
        assert_eq!(next("*/15 * * * *", dt(2025, 1, 1, 12, 15)), dt(2025, 1, 1, 12, 30));
    }

    #[test]
    fn seconds_and_nanoseconds_are_dropped() {
        let since = dt(2025, 1, 1, 12, 0).with_second(30).unwrap().with_nanosecond(500).unwrap();
        assert_eq!(next("* * * * *", since), dt(2025, 1, 1, 12, 1));
    }

    #[test]
    fn minute_carries_into_next_hour() {
        assert_eq!(next("*/15 * * * *", dt(2025, 1, 1, 12, 46)), dt(2025, 1, 1, 13, 0));
        assert_eq!(next("7,37 * * * *", dt(2025, 1, 1, 12, 40)), dt(2025, 1, 1, 13, 7));
    }

    #[test]
    fn minute_carries_over_midnight() {
        assert_eq!(next("5 * * * *", dt(2025, 1, 1, 23, 10)), dt(2025, 1, 2, 0, 5));
    }

    #[test]
    fn hour_jump_resets_minute() {
        assert_eq!(next("30 9 * * *", dt(2025, 1, 1, 8, 45)), dt(2025, 1, 1, 9, 30));
        assert_eq!(next("30 9 * * *", dt(2025, 1, 1, 10, 0)), dt(2025, 1, 2, 9, 30));
    }

    #[test]
    fn year_rolls_over() {
        assert_eq!(next("0 0 1 1 *", dt(2025, 12, 31, 23, 59)), dt(2026, 1, 1, 0, 0));
        assert_eq!(next("0 0 1 3 *", dt(2025, 6, 1, 0, 0)), dt(2026, 3, 1, 0, 0));
    }

    #[test]
    fn leap_day_is_found() {
        assert_eq!(next("0 0 29 2 *", dt(2025, 1, 1, 0, 0)), dt(2028, 2, 29, 0, 0));
    }

    #[test]
    fn short_months_are_skipped() {
        assert_eq!(next("0 0 31 * *", dt(2025, 4, 1, 0, 0)), dt(2025, 5, 31, 0, 0));
    }

    #[test]
    fn weekday_zero_is_sunday() {
        assert_eq!(next("0 0 * * 0", dt(2025, 1, 1, 12, 0)), dt(2025, 1, 5, 0, 0));
    }

    #[test]
    fn restricted_weekday_with_wildcard_day_must_match_weekday() {
        assert_eq!(next("0 12 * * 1-5", dt(2025, 2, 1, 0, 0)), dt(2025, 2, 3, 12, 0));
    }

    #[test]
    fn both_day_fields_restricted_use_or() {
        assert_eq!(next("0 0 13 * 5", dt(2025, 1, 11, 0, 0)), dt(2025, 1, 13, 0, 0));
        assert_eq!(next("0 0 13 * 5", dt(2025, 1, 13, 1, 0)), dt(2025, 1, 17, 0, 0));
        assert_eq!(next("0 0 31 2 1", dt(2025, 2, 1, 0, 0)), dt(2025, 2, 3, 0, 0));
    }

    #[test]
    fn daily_at_midnight() {
        assert_eq!(next("0 0 * * *", dt(2025, 1, 1, 12, 0)), dt(2025, 1, 2, 0, 0));
        assert_eq!(next("0 0 * * *", dt(2025, 1, 1, 0, 0)), dt(2025, 1, 2, 0, 0));
    }

    #[test]
    fn weekly_on_monday_morning() {
        assert_eq!(next("30 8 * * 1", dt(2025, 1, 1, 12, 0)), dt(2025, 1, 6, 8, 30));
        assert_eq!(next("30 8 * * 1", dt(2025, 1, 6, 8, 0)), dt(2025, 1, 6, 8, 30));
        assert_eq!(next("30 8 * * 1", dt(2025, 1, 6, 8, 30)), dt(2025, 1, 13, 8, 30));
    }

    #[test]
    fn stepped_hour_range() {
        assert_eq!(next("0 9-17/2 * * *", dt(2025, 1, 1, 8, 0)), dt(2025, 1, 1, 9, 0));
        assert_eq!(next("0 9-17/2 * * *", dt(2025, 1, 1, 9, 1)), dt(2025, 1, 1, 11, 0));
        assert_eq!(next("0 9-17/2 * * *", dt(2025, 1, 1, 16, 0)), dt(2025, 1, 1, 17, 0));
        assert_eq!(next("0 9-17/2 * * *", dt(2025, 1, 1, 17, 1)), dt(2025, 1, 2, 9, 0));
    }

    #[test]
    fn day_carries_into_next_month() {
        assert_eq!(next("0 0 * * *", dt(2025, 1, 31, 23, 30)), dt(2025, 2, 1, 0, 0));
        assert_eq!(next("0 0 * * *", dt(2025, 2, 28, 12, 0)), dt(2025, 3, 1, 0, 0));
        assert_eq!(next("0 0 * * *", dt(2024, 2, 28, 12, 0)), dt(2024, 2, 29, 0, 0));
    }

    #[test]
    fn day_30_skips_february() {
        assert_eq!(next("0 0 30 * *", dt(2025, 1, 31, 0, 0)), dt(2025, 3, 30, 0, 0));
        assert_eq!(next("0 0 31 * *", dt(2025, 2, 1, 0, 0)), dt(2025, 3, 31, 0, 0));
    }

    #[test]
    fn leap_day_across_a_non_leap_century() {
        assert_eq!(next("0 0 29 2 *", dt(2096, 3, 1, 0, 0)), dt(2104, 2, 29, 0, 0));
    }

    #[test]
    fn end_of_representable_time_returns_none() {
        let e = |s: &str| s.parse::<Expression>().unwrap();
        let max = DateTime::<Utc>::MAX_UTC;
        assert_eq!(e("* * * * *").next_since(max), None);
        assert_eq!(e("* * * * *").next_since(max - TimeDelta::seconds(30)), None);
        assert_eq!(e("0 * * * *").next_since(max - TimeDelta::minutes(30)), None);
    }
}

mod upcoming {
    use super::super::*;
    use chrono::{DateTime, TimeZone, Utc};

    fn dt(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, mo, d, h, mi, 0).unwrap()
    }

    fn expr(s: &str) -> Expression {
        s.parse().unwrap()
    }

    #[test]
    fn first_item_equals_next_since() {
        let e = expr("0 0 13 * 5");
        let since = dt(2025, 1, 11, 0, 0);
        assert_eq!(e.upcoming_since(since).next(), e.next_since(since));
    }

    #[test]
    fn yields_consecutive_matches_across_an_hour() {
        let e = expr("*/15 * * * *");
        let got: Vec<_> = e.upcoming_since(dt(2025, 1, 1, 12, 0)).take(5).collect();
        assert_eq!(got, [
            dt(2025, 1, 1, 12, 15),
            dt(2025, 1, 1, 12, 30),
            dt(2025, 1, 1, 12, 45),
            dt(2025, 1, 1, 13, 0),
            dt(2025, 1, 1, 13, 15),
        ]);
    }

    #[test]
    fn values_are_strictly_increasing() {
        let e = expr("0 0 13 * 5");
        let got: Vec<_> = e.upcoming_since(dt(2025, 1, 1, 0, 0)).take(100).collect();
        assert_eq!(got.len(), 100);
        assert!(got.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn every_value_is_a_match() {
        let e = expr("30 9 * * 1-5");
        let mut prev = dt(2025, 1, 1, 0, 0);
        for t in e.upcoming_since(prev).take(20) {
            assert_eq!(Some(t), e.next_since(prev));
            prev = t;
        }
    }

    #[test]
    fn clone_continues_from_the_same_point() {
        let e = expr("*/15 * * * *");
        let mut it = e.upcoming_since(dt(2025, 1, 1, 12, 0));
        it.next();
        it.next();

        let forked = it.clone();
        assert_eq!(it.take(3).collect::<Vec<_>>(), forked.take(3).collect::<Vec<_>>());
    }

    #[test]
    fn upcoming_starts_after_now() {
        let before = Utc::now();
        let first = expr("* * * * *").upcoming().next().unwrap();
        assert!(first > before);
    }
}
