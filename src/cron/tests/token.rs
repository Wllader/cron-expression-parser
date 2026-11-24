mod parsing {
    use super::super::*;

    #[test]
    fn literal() {
        assert_eq!(raw::<Minute>("5"), [5]);
        assert!(matches!(tok::<Minute>("5").unwrap(), Token::Literal(_)));
    }

    #[test]
    fn wildcard() {
        let t = tok::<Minute>("*").unwrap();
        assert!(t.is_wildcard());
        assert_eq!(t.mask().to_vec(), (0..=59).collect::<Vec<u8>>());
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
    fn list_values_are_sorted_and_deduplicated() {
        assert_eq!(raw::<Minute>("30,5,5"), [5, 30]);
        assert_eq!(raw::<Minute>("40-45,1,42"), [1, 40, 41, 42, 43, 44, 45]);
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

mod operations {
    use super::super::*;

    fn bits(bits: u64) -> Mask<Minute> {
        Mask::from_bits(bits)
    }

    /// `Minute` has no `PartialEq`, so results are compared through `get()`.
    fn raw_opt(v: Option<Minute>) -> Option<u8> {
        v.map(|m| m.get())
    }

    /// The members of a mask, smallest first. `Mask<T>` doesn't implement `PartialEq`,
    /// so masks are compared through their members.
    fn members<T: TimeElement>(mask: Mask<T>) -> Vec<u8> {
        (0..64).filter(|&v| mask.contains(v)).collect()
    }

    #[test]
    fn mask_sets_one_bit_per_value() {
        assert_eq!(members(tok::<Minute>("0,5").unwrap().mask()), [0, 5]);
        assert_eq!(members(tok::<Minute>("59").unwrap().mask()), [59]);
        assert_eq!(members(tok::<Minute>("*").unwrap().mask()), (0..=59).collect::<Vec<u8>>());
    }

    #[test]
    fn mask_of_overlapping_list_has_no_duplicates() {
        assert_eq!(members(tok::<Minute>("1-3,2-4").unwrap().mask()), [1, 2, 3, 4]);
    }

    #[test]
    fn next_in_finds_next_member() {
        let mask = tok::<Minute>("*/15").unwrap().mask();
        assert_eq!(raw_opt(mask.next_in(0)), Some(0));
        assert_eq!(raw_opt(mask.next_in(1)), Some(15));
        assert_eq!(raw_opt(mask.next_in(31)), Some(45));
        assert_eq!(raw_opt(mask.next_in(45)), Some(45));
        assert_eq!(raw_opt(mask.next_in(46)), None);
    }

    #[test]
    fn first_is_smallest_member() {
        assert_eq!(raw_opt(tok::<Minute>("7,37").unwrap().mask().first()), Some(7));
        assert_eq!(raw_opt(tok::<Minute>("0,30").unwrap().mask().first()), Some(0));
        assert_eq!(raw_opt(bits(0).first()), None);
    }

    #[test]
    fn contains_intersects_is_empty() {
        let mask = tok::<Minute>("1-3").unwrap().mask();
        assert!(mask.contains(2));
        assert!(!mask.contains(4));
        assert!(mask.intersects(bits(1 << 3)));
        assert!(!mask.intersects(bits(1 << 4)));
        assert!(!mask.is_empty());
        assert!(bits(0).is_empty());
    }

    #[test]
    fn month_length_masks() {
        let months = |m: &str| tok::<Month>(m).unwrap().mask();
        assert_eq!(members(Month::MASK_31), members(months("1,3,5,7,8,10,12")));
        assert_eq!(members(Month::MASK_30), members(months("4,6,9,11")));
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
