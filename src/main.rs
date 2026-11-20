use cron_exp_parser::cron;

fn main() {
    let expr: cron::Expression = "0 0 29 2 *".parse().unwrap();
    let a = expr.next().unwrap();

    // println!("{a}");

    let u = expr.upcoming().take(3);
    for t in u {
        println!("{t}");
    }
}
