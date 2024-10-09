use crate::util::float_str_to_fixed_point_4_decimal;
use anyhow::{anyhow, ensure, Result};
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

impl TryFrom<RawTransaction> for Transaction {
    type Error = anyhow::Error;

    fn try_from(value: RawTransaction) -> Result<Self> {
        match value.transaction_type {
            RawTransactionType::Deposit => Ok(Transaction::Deposit {
                client_id: value.client,
                tx_id: value.tx,
                amount: value
                    .amount
                    .ok_or(anyhow!("Deposit found without amount"))?,
            }),
            RawTransactionType::Withdrawal => Ok(Transaction::Withdrawal {
                client_id: value.client,
                tx_id: value.tx,
                amount: value
                    .amount
                    .ok_or(anyhow!("Withdrawal found without amount"))?,
            }),
            RawTransactionType::Dispute => {
                ensure!(value.amount.is_none(), anyhow!("Dispute found with amount"));
                Ok(Transaction::Dispute {
                    client_id: value.client,
                    tx_id: value.tx,
                })
            }
            RawTransactionType::Resolve => {
                ensure!(value.amount.is_none(), anyhow!("Resolve found with amount"));
                Ok(Transaction::Resolve {
                    client_id: value.client,
                    tx_id: value.tx,
                })
            }
            RawTransactionType::Chargeback => {
                ensure!(
                    value.amount.is_none(),
                    anyhow!("Chargeback found with amount")
                );
                Ok(Transaction::Chargeback {
                    client_id: value.client,
                    tx_id: value.tx,
                })
            }
        }
    }
}

#[derive(Debug)]
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
    use crate::transaction::{RawTransaction, RawTransactionType, Transaction};
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
            let _transaction: Transaction = raw_transaction.try_into().unwrap();
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
            let _transaction: Transaction = raw_transaction.try_into().unwrap();
        }
    }

    #[test]
    fn test_deposit_without_amount() {
        let raw = RawTransaction {
            transaction_type: RawTransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: None,
        };
        let result = Transaction::try_from(raw);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Deposit found without amount"
        );
    }

    #[test]
    fn test_withdrawal_without_amount() {
        let raw = RawTransaction {
            transaction_type: RawTransactionType::Withdrawal,
            client: 1,
            tx: 1,
            amount: None,
        };
        let result = Transaction::try_from(raw);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Withdrawal found without amount"
        );
    }
}
