# Design choices

## Range-checked field newtypes

Basic newtypes created by a macro to reduce boilerplate.

The field is private, so values can only be created through two range-checked constructors:
- `TryFrom<u8>` - the fallible one, returning `Err(OutOfRange)`, for input we don't control (parsing),
- `new` - a `const fn` that panics on an out-of-range value, for values known to be valid (constants, tests). In a `const` context, an invalid value becomes a compile error.

## Token representation and normalization

In cron expressions, we distinguish multiple tokens that can be used: Literals, Wildcards, Ranges, Steps, RangeSteps and Lists.

We can merge some of them - namely Ranges, Steps and RangeSteps - into a single structure, which I decided to do for my `Token` enum. On paper this is a data tradeoff, since with every range, I need to store a default value for its step (`1`), but it simplifies some decision logic. In practice it costs nothing: `Range<Minute>` is 3 bytes, while `Token` is 24 bytes either way, because its size is dictated by the `Vec` inside the `List` variant.

If I didn't care about memory footprint at all, all cron tokens could be represented as Lists - which are useful, but are also easy to get on demand, instead of storing them all the time.

This overlap of possible token representations implies a need for a normal form, in which we will store tokens that represent the same semantics but came in a different form - e.g. minutes written as `*` vs `0-59`.

I implemented some normalization - the kind that shows up the most - so full ranges normalize to wildcards, ranges over a single element (`6-6`) normalize to a literal (`6`), and lists containing a wildcard (`1,*,5`, or `1,0-59,5` after its full range normalizes) collapse into a wildcard.

Other such cases could be found - single-element lists could be normalized to literals too, or parts of lists that are consecutive could be normalized to ranges (and those could then be queried for overlaps, merged, and possibly the whole list could normalize to a single range or even a wildcard). Those I didn't bother with, as that would go beyond what typical cron parsers do.

## Bitmasks for field sets

For a fast search, we need an alternative representation that is easy to query. One alternative was an array of booleans, but I like my memory footprint slim. A `u64` encoding fits every `TimeElement` as a presence flag, and is easy to both write and read with bit shifts.

The bigger win is speed, though. The question the search keeps asking is "what is the next allowed value from here?" With a bool array, that is a loop over up to 60 entries. With a mask, it is one shift and one `trailing_zeros` (`Mask::next_in`), which compiles down to a single CPU instruction on modern hardware. Membership (`contains`) and overlap checks (`intersects`, used when rejecting impossible dates) are single bitwise operations too.

Masks are generic over the `TimeElement` they are tracking, for two reasons:
- nicer API
- compile-time prohibited interaction of masks of two different kinds

## Search algorithm for the next match

For a fast lookup, we always try to skip the longest period first. `next_since` is an odometer-style search: it walks the fields from the coarsest to the finest (month, day, hour, minute), and for each one:

1. finds the next allowed value that is not smaller than the current one - a single shift and `trailing_zeros` on the field's mask (`Mask::next_in`),
2. if there is none, carries over into the next larger unit (no month left - next year, no hour left - next day, ...) and resets all smaller units,
3. if the allowed value lies ahead, jumps to it and resets all smaller units,
4. if the current value already matches, moves on to the finer field.

Since the search goes from coarse to fine, the first time all fields match is the earliest match.

Days are the exception - they are stepped one at a time. Day-of-month and day-of-week are judged together (see the day rule below), so they can't be merged into a single mask to jump over.

Iterating over minutes with roll-over would work, but an 8-year search could take over 4 million steps. With jumps, a typical search takes a handful of steps, and even the worst case (stepping through days) stays in the low thousands.

My approach is often to build a nice-to-use API from the ground up, which makes implementing algorithms easier down the line. The `next_since` function is such an example - the `Cursor` trait provides the calendar jumps, `Schedule` the masks, and the loop itself reads almost like the description above.

The 8-year search limit is not arbitrary - it is the smallest safe one. The longest gap between two occurrences of a valid date is February 29 across a non-leap century year: 2096 to 2104, exactly 8 years. The limit could be made configurable (an env variable or a config file, if actual deployment should happen), but anything lower than 8 would break expressions like `0 0 29 2 *`.

## Day-of-month vs. day-of-week rule

In cron, usually when neither of the day fields is a wildcard, we should match on either one, while if one is a wildcard, we wait for both to match at the same time. "Usually" because this is the behaviour of Vixie cron, which most modern implementations follow, rather than a formal standard.

The either-one case surprises people, so an example. Starting from December 1st, 2026:

| Expression | Fires on | Why |
|---|---|---|
| `0 0 13 * *` | Sun 13 Dec, Wed 13 Jan, ... | only day-of-month is restricted |
| `0 0 * * 5` | Fri 4 Dec, Fri 11 Dec, Fri 18 Dec, ... | only day-of-week is restricted |
| `0 0 13 * 5` | Fri 4 Dec, Fri 11 Dec, Sun 13 Dec, Fri 18 Dec, ... | both restricted - the 13th **or** any Friday, not "Friday the 13th" |

This behaviour is implemented in `Schedule::day_matches`.

## Rejecting impossible dates at parse time

Some expressions are syntactically valid but can never fire, e.g. `0 0 30 2 *` (February 30th). Those are rejected right away with `cron::Error::ImpossibleDate`, instead of letting every `next_since` call search the full 8 years and return `None`.

The check (`Expression::has_possible_date`) works on the masks: it finds the longest month the month field allows (31, 30 or 29 days - February counted with its leap day) and checks that the smallest allowed day-of-month fits into it.

It only runs when day-of-week is a wildcard. When day-of-week is restricted and day-of-month is not, any day of month is allowed; when both are restricted, the either-one rule applies and some matching weekday always comes. So `0 0 30 2 1` is valid - it fires on every Monday in February.

Parsing is handled through `FromStr` implemented on both `Token` and `Expression`.

The former takes care of the normalization described earlier and produces errors on invalid input.

The latter binds the parsed tokens together in an expression structure, together with a `Schedule` struct composed of masks for each field - if we chose the bool-array alternative, keeping the masks from the get-go would be more memory demanding, but storing a few `u64`s is quite alright and we don't need to manage a lazy-loading mechanism.

The `Expression` part is also where possible errors are short-circuited and delegated to the call site (typically `.parse()`).

## Error handling

As described above, the errors that can occur while parsing expressions are all encapsulated by the `cron::Error` enum. It implements `Display` and `std::error::Error`, so it works with `?` in functions returning `Box<dyn Error>`, and with crates like `anyhow`.

Searching for the next match returns `Option` instead. A `None` can come from two places: the 8-year limit running out, or a date past the maximum representable year in `NaiveDateTime`. Since impossible dates are rejected at parse time and 8 years is always enough for a valid date, the first can't happen for an accepted expression. That leaves only the end of representable time, so in practice `next_since` always returns `Some`.

## Public API

I aimed for an API that feels like the standard library, so that using it needs no reading of documentation:

```rust
let expr: Expression = "0 9-17/2 * * 1-5".parse()?;
let next = expr.next_since(since);
let week: Vec<_> = expr.upcoming_since(since).take_while(|t| *t < since + TimeDelta::weeks(1)).collect();
```

Parsing goes through `FromStr` rather than a free `parse` function. That is the idiomatic way to turn a string into a value in Rust: it works with `str::parse` and type inference, with `?` (thanks to `cron::Error` implementing `std::error::Error`), and with any code that is generic over `FromStr`. `Display` goes the other way and prints the normalized form, so an expression round-trips through a string.

Times are `chrono::DateTime<Utc>` instead of raw Unix timestamps. The type itself says that the value is in UTC and has no unit to get wrong (seconds vs. milliseconds), and it formats nicely when printed. A caller who works with timestamps is one call away in each direction - `DateTime::from_timestamp(ts, 0)` and `.timestamp()`.

Every operation comes in two flavours:
- `next` and `upcoming` start from the current time, for the common case,
- `next_since` and `upcoming_since` start from a given moment, which makes them deterministic - all the tests use these.

`Upcoming` is a lazy iterator over consecutive matches. It borrows the expression instead of cloning it, computes each match only when asked, and composes with the usual iterator adapters (`take`, `take_while`, `filter`, ...). It also implements `FusedIterator`, and `Clone`, so an iterator can be forked at any point.
