use crate::util::float_str_to_fixed_point_4_decimal;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

// This could easily be an enum with the variants of TransactionType and enum values that store
//  only the necessary data (amount would only be a value of Deposit and Withdrawal). Going with
//  the approach below to take advantage of serde's deserialization. Serde's Enum tagging doesn't
//  work with csv parsing.
#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    #[serde(deserialize_with = "deserialize_fixed_point", rename = "amount")]
    pub amount: Option<u64>,
}

fn deserialize_fixed_point<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let f_string: Option<String> = Deserialize::deserialize(deserializer)?;
    f_string
        .map(|f| {
            float_str_to_fixed_point_4_decimal(&f).map_err(|e| {
                serde::de::Error::custom(format!(
                    "Failed to parse float into fixed point representation: {e}"
                ))
            })
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use crate::transaction::Transaction;
    use std::io::BufReader;

    #[test]
    fn test_transaction_deserialization() {
        let csv = "type, client, tx, amount
                        deposit, 1, 1, 1.0
                        withdrawal, 1, 4, 1.5
                        dispute, 1, 1,
                        resolve, 1, 1,
                        dispute, 2, 2,
                        chargeback, 2, 2,";

        let mut csv_reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(BufReader::new(csv.as_bytes()));

        for res in csv_reader.deserialize() {
            let _transaction: Transaction = res.unwrap();
        }
    }

    #[test]
    fn test_transaction_deserialization_white_spaces_are_ignored() {
        let csv = "type,client, tx,amount
                        deposit, 1,1, 1.0
                        deposit,2,2,2.0";

        let mut csv_reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(BufReader::new(csv.as_bytes()));

        for res in csv_reader.deserialize() {
            let _transaction: Transaction = res.unwrap();
        }
    }
}
