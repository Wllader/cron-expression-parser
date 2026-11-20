use cron_exp_parser::cron;

fn main() {
    let expr: cron::Expression = "0 0 31 2 0-6".parse().unwrap();
    let a = expr.json_val_sets();
    println!("{a}");
    println!("{:#?}", expr);

    let a = expr.minute().mask();
    println!("{:b} : {}", a, format!("{:b}", a).len())


}
