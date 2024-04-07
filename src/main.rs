use crate::engine::Engine;
use crate::transaction::RawTransaction;
use std::env;

mod engine;
mod transaction;
mod util;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Correct usage: `cargo run -- <transactions_csv_file>`");
    }

    let transactions_csv_path = args[1].as_str();

    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(transactions_csv_path)
        .unwrap();

    let mut engine = Engine::new();

    for res in csv_reader.deserialize() {
        let raw_transaction: RawTransaction = res.expect("Invalid row in provided csv");
        let transaction = raw_transaction
            .into_transaction()
            .expect("Invalid row in provided csv");
        engine.process_transaction(transaction);
    }

    engine.print_state_csv();
}
