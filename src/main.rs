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

    let transactions_csv_path = &args[1];

    let mut csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(transactions_csv_path)
        .expect("Failed to create input csv reader");

    let mut engine = Engine::new();

    for result in csv_reader.deserialize::<RawTransaction>() {
        match result.map(TryInto::try_into) {
            Ok(Ok(t)) => engine.process_transaction(t),
            Ok(Err(e)) => {
                eprintln!("Invalid row in provided csv: {e}");
            }
            Err(e) => {
                eprintln!("Invalid row in provided csv: {e}");
            }
        }
    }

    engine
        .print_state_csv()
        .expect("Failed to print output csv");
}
