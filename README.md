# cron_exp_parser

A Rust library that parses standard five-field cron expressions
(`minute hour day-of-month month day-of-week`) and computes when they fire next.

```rust
use cron_exp_parser::cron::Expression;

let expr: Expression = "*/5 * * * *".parse()?;

println!("{}", expr.next().unwrap());        // next match after now (UTC)
for t in expr.upcoming().take(5) {           // the next five matches
    println!("{t}");
}
```

For the reasoning behind the implementation, see [DESIGN.md](DESIGN.md).

## Requirements

- [rustup](https://rustup.rs/)
- A **stable** toolchain, Rust 1.85 or newer (the crate uses edition 2024).
  `rust-toolchain.toml` pins the project to `stable`, so rustup selects it
  automatically the first time you run `cargo` in this directory.

## Build

```sh
cargo build            # debug build
cargo build --release  # optimized build
```

## Run the tests

```sh
cargo test
```

The tests are in `src/cron/tests/`, with one file per source module
(`time`, `token`, `expression`, `schedule`). To run only part of the suite, pass a
filter that matches the test path:

```sh
cargo test cron::tests::token             # all token tests
cargo test cron::tests::schedule::upcoming  # just the Upcoming iterator tests
cargo test leap_day                       # any test whose name contains "leap_day"
```

## Lint

```sh
cargo clippy --all-targets
```

## Trying it out locally

The repository is library-only. `src/main.rs` is in `.gitignore`, so you can keep
a personal scratch binary there without committing it. Cargo picks it up
automatically, and you run it with `cargo run`:

```rust
// src/main.rs
use cron_exp_parser::cron;

fn main() {
    let expr: cron::Expression = "0 9 * * 1-5".parse().unwrap();
    println!("{}", expr.next().unwrap());
}
```
