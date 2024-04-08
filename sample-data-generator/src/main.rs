const N_CLIENTS: u16 = 5_000;
const N_DEPOSITS_PER_CLIENT: u32 = 8_000;
const N_WITHDRAWALS_PER_CLIENT: u32 = 2_000;
const N_DISPUTES_PER_CLIENT: u32 = 300;

fn main() {
    let mut wtr = csv::Writer::from_path("transactions.csv").unwrap();

    wtr.write_record(["type", "client", "tx", "amount"])
        .unwrap();

    let mut tx_id_count = 0;

    let mut disputes_to_create: Vec<(u16, u32)> = Vec::new();

    for client_id in 0..N_CLIENTS {
        for d in 0..N_DEPOSITS_PER_CLIENT {
            wtr.serialize(("deposit", client_id, tx_id_count, 100.0))
                .unwrap();
            if d < N_DISPUTES_PER_CLIENT {
                disputes_to_create.push((client_id, tx_id_count));
            }
            tx_id_count += 1;
        }
    }

    for client_id in 0..N_CLIENTS {
        for _ in 0..N_WITHDRAWALS_PER_CLIENT {
            wtr.serialize(("withdrawal", client_id, tx_id_count, 100.0))
                .unwrap();
            tx_id_count += 1;
        }
    }

    for (client_id, tx_id) in disputes_to_create.as_slice() {
        wtr.serialize(("dispute", client_id, tx_id, "")).unwrap();
    }

    for (client_id, tx_id) in disputes_to_create.as_slice() {
        wtr.serialize(("resolve", client_id, tx_id, "")).unwrap();
    }

    for (client_id, tx_id) in disputes_to_create.as_slice() {
        wtr.serialize(("dispute", client_id, tx_id, "")).unwrap();
    }

    for (client_id, tx_id) in disputes_to_create.as_slice() {
        wtr.serialize(("chargeback", client_id, tx_id, "")).unwrap();
    }

    wtr.flush().unwrap();
}
