use crate::cron::{self, DayOfMonth, DayOfWeek, Expression, Hour, Minute, Month, TimeElement, Token};

fn tok<T: TimeElement>(s: &str) -> Result<Token<T>, cron::Error> {
    s.parse()
}

fn raw<T: TimeElement>(s: &str) -> Vec<u8> {
    tok::<T>(s).unwrap().to_raw_vec()
}

mod time_elements {
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
}

mod token_parsing {
    use super::*;

    #[test]
    fn literal() {
        assert_eq!(raw::<Minute>("5"), [5]);
        assert!(matches!(tok::<Minute>("5").unwrap(), Token::Literal(_)));
    }

    #[test]
    fn wildcard() {
        let t = tok::<Minute>("*").unwrap();
        assert!(t.is_wildcard());
        assert_eq!(t.to_raw_vec(), (0..=59).collect::<Vec<u8>>());
    }

    #[test]
    fn surrounding_whitespace_is_trimmed() {
        assert_eq!(raw::<Minute>("  5 "), [5]);
    }

    #[test]
    fn range() {
        assert_eq!(raw::<Minute>("1-5"), [1, 2, 3, 4, 5]);
        assert!(matches!(tok::<Minute>("1-5").unwrap(), Token::Range(_)));
    }

    #[test]
    fn step_over_wildcard() {
        assert_eq!(raw::<Minute>("*/15"), [0, 15, 30, 45]);
        assert_eq!(raw::<DayOfMonth>("*/10"), [1, 11, 21, 31]);
    }

    #[test]
    fn step_over_range() {
        assert_eq!(raw::<Minute>("1-10/4"), [1, 5, 9]);
    }

    #[test]
    fn single_value_range_normalizes_to_literal() {
        for (s, expected) in [("5-5", "5"), ("5-5/3", "5"), ("59/5", "59"), ("0-0/59", "0")] {
            let t = tok::<Minute>(s).unwrap();
            assert!(matches!(t, Token::Literal(_)), "expected Literal for {s:?}, got {t:?}");
            assert_eq!(t.to_string(), expected);
        }
    }

    #[test]
    fn single_value_range_inside_list_normalizes() {
        match tok::<Minute>("5-5,7").unwrap() {
            Token::List(items) => {
                assert_eq!(items.len(), 2);
                assert!(items.iter().all(|t| matches!(t, Token::Literal(_))));
            }
            other => panic!("expected list, got {other:?}"),
        }
    }

    #[test]
    fn invalid_step_is_still_rejected_on_single_value_range() {
        assert!(matches!(tok::<Minute>("5-5/0"), Err(cron::Error::InvalidStep)));
    }

    #[test]
    fn step_from_start_runs_to_max() {
        assert_eq!(raw::<Minute>("10/25"), [10, 35]);
    }

    #[test]
    fn list() {
        assert_eq!(raw::<Minute>("1,5,10-12"), [1, 5, 10, 11, 12]);
        assert!(matches!(tok::<Minute>("1,5").unwrap(), Token::List(_)));
    }

    #[test]
    fn full_range_normalizes_to_wildcard() {
        assert!(tok::<Minute>("0-59").unwrap().is_wildcard());
        assert!(tok::<DayOfWeek>("0-6").unwrap().is_wildcard());
        assert!(tok::<Minute>("0-59/1").unwrap().is_wildcard());
    }

    #[test]
    fn stepped_full_range_is_not_wildcard() {
        assert!(!tok::<Minute>("0-59/5").unwrap().is_wildcard());
        assert!(!tok::<Minute>("*/5").unwrap().is_wildcard());
    }

    #[test]
    fn list_containing_wildcard_normalizes() {
        assert!(tok::<Minute>("1,*").unwrap().is_wildcard());
        assert!(tok::<Minute>("1,0-59").unwrap().is_wildcard());
    }

    #[test]
    fn out_of_range_values() {
        assert!(matches!(tok::<Minute>("60"), Err(cron::Error::OutOfRange)));
        assert!(matches!(tok::<Minute>("300"), Err(cron::Error::OutOfRange)));
        assert!(matches!(tok::<Hour>("24"), Err(cron::Error::OutOfRange)));
        assert!(matches!(tok::<DayOfMonth>("0"), Err(cron::Error::OutOfRange)));
        assert!(matches!(tok::<Month>("13"), Err(cron::Error::OutOfRange)));
        assert!(matches!(tok::<DayOfWeek>("7"), Err(cron::Error::OutOfRange)));
        assert!(matches!(tok::<Minute>("0-60"), Err(cron::Error::OutOfRange)));
    }

    #[test]
    fn reversed_range_is_rejected() {
        assert!(matches!(tok::<Minute>("5-1"), Err(cron::Error::OutOfRange)));
    }

    #[test]
    fn invalid_numbers() {
        for s in ["abc", "", "   ", "+5", "-5", "1-", "1,,2", "1,", "5.5"] {
            assert!(
                matches!(tok::<Minute>(s), Err(cron::Error::InvalidNumber(_))),
                "expected InvalidNumber for {s:?}"
            );
        }
    }

    #[test]
    fn invalid_steps() {
        assert!(matches!(tok::<Minute>("*/0"), Err(cron::Error::InvalidStep)));
        assert!(matches!(tok::<Minute>("*/x"), Err(cron::Error::InvalidStep)));
        assert!(matches!(tok::<Minute>("*/"), Err(cron::Error::InvalidStep)));
    }
}

mod token_operations {
    use super::*;

    #[test]
    fn mask_sets_one_bit_per_value() {
        assert_eq!(tok::<Minute>("0,5").unwrap().mask(), 1 | (1 << 5));
        assert_eq!(tok::<Minute>("59").unwrap().mask(), 1 << 59);
        assert_eq!(tok::<Minute>("*").unwrap().mask(), (1u64 << 60) - 1);
    }

    #[test]
    fn mask_of_overlapping_list_has_no_duplicates() {
        assert_eq!(
            tok::<Minute>("1-3,2-4").unwrap().mask(),
            0b11110
        );
    }

    #[test]
    fn next_in_finds_next_member() {
        let mask = tok::<Minute>("*/15").unwrap().mask();
        assert_eq!(Token::<Minute>::next_in(mask, 0), Some(0));
        assert_eq!(Token::<Minute>::next_in(mask, 1), Some(15));
        assert_eq!(Token::<Minute>::next_in(mask, 31), Some(45));
        assert_eq!(Token::<Minute>::next_in(mask, 45), Some(45));
        assert_eq!(Token::<Minute>::next_in(mask, 46), None);
    }

    #[test]
    fn max() {
        assert_eq!(tok::<Minute>("5").unwrap().max(), 5);
        assert_eq!(tok::<Minute>("*").unwrap().max(), 59);
        assert_eq!(tok::<Minute>("1-10").unwrap().max(), 10);
        assert_eq!(tok::<Minute>("3,40,7").unwrap().max(), 40);
    }

    #[test]
    fn to_vec_matches_to_raw_vec() {
        for s in ["5", "*", "1-5", "*/20", "1,5,10-12"] {
            let t = tok::<Minute>(s).unwrap();
            let typed: Vec<u8> = t.to_vec().unwrap().into_iter().map(|m| m.get()).collect();
            assert_eq!(typed, t.to_raw_vec(), "mismatch for {s:?}");
        }
    }

    #[test]
    fn range_accessors() {
        match tok::<Minute>("2-20/6").unwrap() {
            Token::Range(r) => {
                assert_eq!((r.start(), r.end(), r.step()), (2, 20, 6));
                assert_eq!(r.values().collect::<Vec<_>>(), [2, 8, 14, 20]);
            }
            other => panic!("expected range, got {other:?}"),
        }
    }

    #[test]
    fn display_round_trips() {
        for s in ["5", "*", "1-5", "*/15", "1-10/4", "1,5,10-12", "10-30/5"] {
            assert_eq!(tok::<Minute>(s).unwrap().to_string(), s);
        }
    }

    #[test]
    fn display_of_range_from_start_is_expanded() {
        assert_eq!(tok::<Minute>("10/25").unwrap().to_string(), "10-59/25");
    }
}

mod expression {
    use super::*;

    fn expr(s: &str) -> Result<Expression, cron::Error> {
        s.parse()
    }

    #[test]
    fn parses_all_fields() {
        let e = expr("*/15 0 1,15 1-6 1-5").unwrap();
        assert_eq!(e.minute().to_raw_vec(), [0, 15, 30, 45]);
        assert_eq!(e.hour().to_raw_vec(), [0]);
        assert_eq!(e.day_of_month().to_raw_vec(), [1, 15]);
        assert_eq!(e.month().to_raw_vec(), [1, 2, 3, 4, 5, 6]);
        assert_eq!(e.day_of_week().to_raw_vec(), [1, 2, 3, 4, 5]);
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
        assert!(expr("0 0 29 2 *").is_ok()); // leap years
        assert!(expr("0 0 30 4 *").is_ok());
        assert!(expr("0 0 31 1 *").is_ok());
        assert!(expr("0 0 31 1,4 *").is_ok()); // one valid pair is enough
        assert!(expr("0 0 15,31 2 *").is_ok()); // Feb 15 exists
        assert!(expr("0 0 * * *").is_ok());
    }

    #[test]
    fn restricted_weekday_allows_impossible_day_of_month() {
        // cron's OR rule: fires on every Monday even though Feb 31 doesn't exist
        assert!(expr("0 0 31 2 1").is_ok());
    }

    /// Straightforward reference implementation that `Expression::has_possible_date` must agree with.
    fn has_possible_date_slow(e: &Expression) -> bool {
        let months = e.month().to_raw_vec();
        let days = e.day_of_month().to_raw_vec();
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
}
