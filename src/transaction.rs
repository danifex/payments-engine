use crate::util::float_str_to_fixed_point_4_decimal;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RawTransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
pub struct RawTransaction {
    #[serde(rename = "type")]
    pub transaction_type: RawTransactionType,
    pub client: u16,
    pub tx: u32,
    #[serde(deserialize_with = "deserialize_fixed_point")]
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

impl RawTransaction {
    pub fn into_transaction(self) -> Result<Transaction> {
        match self.transaction_type {
            RawTransactionType::Deposit => Ok(Transaction::Deposit {
                client_id: self.client,
                tx_id: self.tx,
                amount: self.amount.ok_or(anyhow!("Deposit found without amount"))?,
            }),
            RawTransactionType::Withdrawal => Ok(Transaction::Withdrawal {
                client_id: self.client,
                tx_id: self.tx,
                amount: self
                    .amount
                    .ok_or(anyhow!("Withdrawal found without amount"))?,
            }),
            RawTransactionType::Dispute => Ok(Transaction::Dispute {
                client_id: self.client,
                tx_id: self.tx,
            }),
            RawTransactionType::Resolve => Ok(Transaction::Resolve {
                client_id: self.client,
                tx_id: self.tx,
            }),
            RawTransactionType::Chargeback => Ok(Transaction::Chargeback {
                client_id: self.client,
                tx_id: self.tx,
            }),
        }
    }
}

pub(crate) enum Transaction {
    Deposit {
        client_id: u16,
        tx_id: u32,
        amount: u64,
    },
    Withdrawal {
        client_id: u16,
        #[allow(dead_code)]
        tx_id: u32,
        amount: u64,
    },
    Dispute {
        client_id: u16,
        tx_id: u32,
    },
    Resolve {
        client_id: u16,
        tx_id: u32,
    },
    Chargeback {
        client_id: u16,
        tx_id: u32,
    },
}

#[cfg(test)]
mod tests {
    use crate::transaction::RawTransaction;
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
            let raw_transaction: RawTransaction = res.unwrap();
            let _transaction = raw_transaction.into_transaction().unwrap();
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
            let raw_transaction: RawTransaction = res.unwrap();
            let _transaction = raw_transaction.into_transaction().unwrap();
        }
    }
}
