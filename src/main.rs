use std::process;

fn main() {
    if let Err(err) = payment_engine::run() {
        println!("{}", err);
        process::exit(1);
    }
}