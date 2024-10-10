use crate::transaction::Transaction;
use crate::util::{fixed_point_4_decimal_to_float_str, signed_fixed_point_4_decimal_to_float_str};
use anyhow::{anyhow, bail, ensure, Result};
use std::collections::{HashMap, HashSet};
use std::ops::Not;

pub struct Engine {
    accounts: HashMap<u16, Account>,
    transactions: HashSet<u32>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashSet::new(),
        }
    }

    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<()> {
        // Check for tx_id uniqueness
        match transaction {
            Transaction::Deposit { tx_id, .. } | Transaction::Withdrawal { tx_id, .. } => {
                ensure!(
                    self.transactions.insert(tx_id),
                    anyhow!("A transaction failed because it had a duplicate tx_id: {tx_id}")
                );
            }
            Transaction::Dispute { .. }
            | Transaction::Resolve { .. }
            | Transaction::Chargeback { .. } => {}
        }

        // Process transaction
        match transaction {
            Transaction::Deposit {
                client_id,
                tx_id,
                amount,
            } => {
                let account = self.accounts.entry(client_id).or_insert_with(Account::new);
                account.deposit(tx_id, amount)?;
            }
            Transaction::Withdrawal {
                client_id, amount, ..
            } => {
                if let Some(account) = self.accounts.get_mut(&client_id) {
                    account.withdraw(amount)?
                } else {
                    bail!("An withdrawal failed because the target account couldn't be found")
                }
            }
            Transaction::Dispute { client_id, tx_id } => {
                if let Some(account) = self.accounts.get_mut(&client_id) {
                    account.start_dispute(tx_id)?
                } else {
                    bail!("A dispute start failed because the target account couldn't be found")
                }
            }
            Transaction::Resolve { client_id, tx_id } => {
                if let Some(account) = self.accounts.get_mut(&client_id) {
                    account.resolve_dispute(tx_id)?
                } else {
                    bail!("A dispute resolve failed because the target account couldn't be found")
                }
            }
            Transaction::Chargeback { client_id, tx_id } => {
                if let Some(account) = self.accounts.get_mut(&client_id) {
                    account.chargeback(tx_id)?
                } else {
                    bail!("A chargeback failed because the target account couldn't be found")
                }
            }
        };
        Ok(())
    }

    pub fn print_state_csv(&self) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());

        wtr.write_record(["client", "available", "held", "total", "locked"])?;

        for (client_id, account) in self.accounts.iter() {
            let available_amount =
                signed_fixed_point_4_decimal_to_float_str(account.available_amount);
            let held_amount = fixed_point_4_decimal_to_float_str(account.held_amount);
            let total_amount = signed_fixed_point_4_decimal_to_float_str(
                account.available_amount + account.held_amount as i64,
            );

            wtr.serialize((
                client_id,
                available_amount,
                held_amount,
                total_amount,
                account.locked,
            ))?;
        }

        wtr.flush()?;

        Ok(())
    }
}

struct Account {
    available_amount: i64,
    held_amount: u64,
    locked: bool,
    deposits: HashMap<u32, Deposit>,
}

impl Account {
    fn new() -> Self {
        Self {
            available_amount: 0,
            held_amount: 0,
            locked: false,
            deposits: HashMap::new(),
        }
    }

    fn deposit(&mut self, tx_id: u32, amount: u64) -> Result<()> {
        ensure!(
            self.locked.not(),
            anyhow!("A deposit failed because the target account is locked")
        );

        self.deposits.insert(
            tx_id,
            Deposit {
                amount,
                state: DepositState::Valid,
            },
        );

        self.available_amount += amount as i64;
        Ok(())
    }

    fn withdraw(&mut self, amount: u64) -> Result<()> {
        ensure!(
            self.locked.not(),
            anyhow!("An withdrawal failed because the target account is locked")
        );

        if self.available_amount >= amount as i64 {
            self.available_amount -= amount as i64
        } else {
            bail!("An withdrawal failed because there wasn't enough balance");
        }
        Ok(())
    }

    fn start_dispute(&mut self, tx_id: u32) -> Result<()> {
        let deposit = self.deposits.get_mut(&tx_id);

        if let Some(deposit) = deposit {
            match deposit.state {
                DepositState::Valid => {
                    deposit.state = DepositState::InDispute;
                    self.available_amount -= deposit.amount as i64;
                    self.held_amount += deposit.amount;
                }
                DepositState::InDispute | DepositState::ChargedBack => {
                    bail!(
                        "A dispute start failed because the referenced deposit was already \
                chargedback or is currently in an active dispute - tx_id: {tx_id} \
                - deposit state: {:?}",
                        deposit.state
                    )
                }
            }
        } else {
            bail!(
                "A dispute start failed because the referenced deposit couldn't be found \
            - tx_id: {tx_id}"
            )
        }
        Ok(())
    }

    fn resolve_dispute(&mut self, tx_id: u32) -> Result<()> {
        let deposit = self.deposits.get_mut(&tx_id);

        if let Some(deposit) = deposit {
            match deposit.state {
                DepositState::InDispute => {
                    deposit.state = DepositState::Valid;
                    self.available_amount += deposit.amount as i64;
                    self.held_amount -= deposit.amount;
                }
                DepositState::ChargedBack | DepositState::Valid => {
                    bail!(
                        "A dispute resolve failed because the referenced deposit wasn't in an \
                active dispute - tx_id: {tx_id} - deposit state: {:?}",
                        deposit.state
                    )
                }
            }
        } else {
            bail!(
                "A dispute resolve failed because the referenced deposit couldn't be found \
            - tx_id: {tx_id}"
            )
        }
        Ok(())
    }

    fn chargeback(&mut self, tx_id: u32) -> Result<()> {
        let deposit = self.deposits.get_mut(&tx_id);

        if let Some(deposit) = deposit {
            match deposit.state {
                DepositState::InDispute => {
                    deposit.state = DepositState::ChargedBack;
                    self.held_amount -= deposit.amount;
                    self.locked = true;
                }
                DepositState::ChargedBack | DepositState::Valid => {
                    bail!(
                        "A chargeback failed because the referenced deposit wasn't in an active \
                dispute - tx_id: {tx_id} - deposit state: {:?}",
                        deposit.state
                    )
                }
            }
        } else {
            bail!(
                "A chargeback failed because the referenced deposit couldn't be found \
            - tx_id: {tx_id}"
            )
        }
        Ok(())
    }
}

struct Deposit {
    amount: u64,
    state: DepositState,
}

#[derive(PartialEq, Debug)]
enum DepositState {
    Valid,
    InDispute,
    ChargedBack,
}

#[cfg(test)]
mod tests {
    use crate::engine::Account;
    use std::ops::Not;

    #[test]
    fn test_account_flow() {
        let mut account = Account::new();
        assert_eq!(account.available_amount, 0);
        assert_eq!(account.held_amount, 0);
        assert!(account.locked.not());
        assert!(account.deposits.is_empty());

        // Make 2 deposits totalling 60
        account.deposit(1, 20).unwrap();
        account.deposit(2, 40).unwrap();
        assert_eq!(account.available_amount, 60);
        assert_eq!(account.held_amount, 0);

        // Check disputing tx 1
        account.start_dispute(1).unwrap();
        assert_eq!(account.available_amount, 40);
        assert_eq!(account.held_amount, 20);

        // Check resolving tx 1
        account.resolve_dispute(1).unwrap();
        assert_eq!(account.available_amount, 60);
        assert_eq!(account.held_amount, 0);

        // Check dispute can be started again + can't dispute same tx again
        account.start_dispute(1).unwrap();
        assert!(account.start_dispute(1).is_err());
        assert_eq!(account.available_amount, 40);
        assert_eq!(account.held_amount, 20);

        // Check having multiple in-progress disputes
        account.start_dispute(2).unwrap();
        assert_eq!(account.available_amount, 0);
        assert_eq!(account.held_amount, 60);

        // Resolve all disputes
        account.resolve_dispute(1).unwrap();
        account.resolve_dispute(2).unwrap();
        assert_eq!(account.available_amount, 60);
        assert_eq!(account.held_amount, 0);

        // Chargeback non disputed tx returns error
        assert!(account.chargeback(1).is_err());
        assert_eq!(account.available_amount, 60);
        assert_eq!(account.held_amount, 0);

        // Check chargeback
        account.start_dispute(1).unwrap();
        account.chargeback(1).unwrap();
        assert_eq!(account.available_amount, 40);
        assert_eq!(account.held_amount, 0);
        assert!(account.locked);
    }

    #[test]
    fn test_account_chargeback_after_withdrawal_flow() {
        let mut account = Account::new();
        account.deposit(1, 100).unwrap();
        account.deposit(2, 50).unwrap();
        assert_eq!(account.available_amount, 150);
        assert_eq!(account.held_amount, 0);

        account.withdraw(100).unwrap();
        assert_eq!(account.available_amount, 50);
        assert_eq!(account.held_amount, 0);

        account.start_dispute(1).unwrap();
        assert_eq!(account.available_amount, -50);
        assert_eq!(account.held_amount, 100);

        account.deposit(3, 25).unwrap();
        assert_eq!(account.available_amount, -25);
        assert_eq!(account.held_amount, 100);

        account.start_dispute(3).unwrap();
        assert_eq!(account.available_amount, -50);
        assert_eq!(account.held_amount, 125);

        account.resolve_dispute(3).unwrap();
        assert_eq!(account.available_amount, -25);
        assert_eq!(account.held_amount, 100);

        account.chargeback(1).unwrap();
        assert_eq!(account.available_amount, -25);
        assert_eq!(account.held_amount, 0);
        assert!(account.locked);
    }
}
