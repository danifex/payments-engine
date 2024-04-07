use csv::ReaderBuilder;
use std::process::Command;

#[test]
fn test_sample_transactions() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--",
            "tests/test_sample_data/sample_transactions.csv",
        ])
        .output()
        .expect("Failed to execute cargo run");

    assert!(output.status.success(), "Cargo run failed");

    assert_eq!(
        find_client_row(&output.stdout, "1"),
        "1,200.0000,0.0000,200.0000,false"
    );
    assert_eq!(
        find_client_row(&output.stdout, "2"),
        "2,25.0000,0.0000,25.0000,false"
    );
    assert_eq!(
        find_client_row(&output.stdout, "3"),
        "3,100.0000,100.0000,200.0000,false"
    );
    assert_eq!(
        find_client_row(&output.stdout, "4"),
        "4,200.0000,0.0000,200.0000,false"
    );
    assert_eq!(
        find_client_row(&output.stdout, "5"),
        "5,0.0000,0.0000,0.0000,true"
    );
    assert_eq!(
        find_client_row(&output.stdout, "6"),
        "6,-50.0000,100.0000,50.0000,false"
    );
    assert_eq!(
        find_client_row(&output.stdout, "7"),
        "7,100.0000,0.0000,100.0000,false"
    );
    assert_eq!(
        find_client_row(&output.stdout, "8"),
        "8,-1000000000.0000,0.0000,-1000000000.0000,true"
    );
}

fn find_client_row(csv_data: &[u8], client_id: &str) -> String {
    let mut reader = ReaderBuilder::new().from_reader(csv_data);
    for result in reader.records() {
        let record = result.unwrap();
        if record.get(0) == Some(client_id) {
            return record.iter().collect::<Vec<&str>>().join(",");
        }
    }
    String::new()
}
