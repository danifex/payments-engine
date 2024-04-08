use crate::transaction::Transaction;
use crate::util::{fixed_point_4_decimal_to_float_str, signed_fixed_point_4_decimal_to_float_str};
use anyhow::Result;
use std::collections::HashMap;

pub struct Engine {
    accounts: HashMap<u16, Account>,
    deposits: HashMap<(u16, u32), Deposit>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            deposits: HashMap::new(),
        }
    }

    pub fn process_transaction(&mut self, transaction: Transaction) {
        match transaction {
            Transaction::Deposit {
                client_id,
                tx_id,
                amount,
            } => {
                let account = self.accounts.get_mut(&client_id);

                if let Some(account) = account {
                    account.deposit(amount)
                } else {
                    let mut account = Account::new();
                    account.deposit(amount);
                    self.accounts.insert(client_id, account);
                }

                self.deposits.insert(
                    (client_id, tx_id),
                    Deposit {
                        amount,
                        state: DepositState::Valid,
                    },
                );
            }
            Transaction::Withdrawal {
                client_id, amount, ..
            } => {
                let account = self.accounts.get_mut(&client_id);

                if let Some(account) = account {
                    account.withdraw(amount)
                } else {
                    eprintln!("An withdrawal failed because the target account couldn't be found")
                }
            }
            Transaction::Dispute { client_id, tx_id } => {
                let account = self.accounts.get_mut(&client_id);

                if let Some(account) = account {
                    if let Some(deposit) = self.deposits.get_mut(&(client_id, tx_id)) {
                        account.start_dispute(tx_id, deposit)
                    } else {
                        eprintln!(
                            "A dispute start failed because the referenced deposit couldn't be found \
            - tx_id: {tx_id}"
                        );
                    }
                } else {
                    eprintln!("A dispute start failed because the target account couldn't be found")
                }
            }
            Transaction::Resolve { client_id, tx_id } => {
                let account = self.accounts.get_mut(&client_id);

                if let Some(account) = account {
                    if let Some(deposit) = self.deposits.get_mut(&(client_id, tx_id)) {
                        account.resolve_dispute(tx_id, deposit)
                    } else {
                        eprintln!(
                            "A dispute resolve failed because the referenced deposit couldn't be found \
            - tx_id: {tx_id}"
                        );
                    }
                } else {
                    eprintln!(
                        "A dispute resolve failed because the target account couldn't be found"
                    )
                }
            }
            Transaction::Chargeback { client_id, tx_id } => {
                let account = self.accounts.get_mut(&client_id);

                if let Some(account) = account {
                    if let Some(deposit) = self.deposits.get_mut(&(client_id, tx_id)) {
                        account.chargeback(tx_id, deposit)
                    } else {
                        eprintln!(
                            "A chargeback failed because the referenced deposit couldn't be found \
            - tx_id: {tx_id}"
                        );
                    }
                } else {
                    eprintln!("A chargeback failed because the target account couldn't be found")
                }
            }
        }
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
}

impl Account {
    fn new() -> Self {
        Self {
            available_amount: 0,
            held_amount: 0,
            locked: false,
        }
    }

    fn deposit(&mut self, amount: u64) {
        if self.locked {
            eprintln!("A deposit failed because the target account is locked");
            return;
        }

        self.available_amount += amount as i64;
    }

    fn withdraw(&mut self, amount: u64) {
        if self.locked {
            eprintln!("An withdrawal failed because the target account is locked");
            return;
        }

        if self.available_amount >= amount as i64 {
            self.available_amount -= amount as i64
        } else {
            eprintln!("An withdrawal failed because there wasn't enough balance");
        }
    }

    fn start_dispute(&mut self, tx_id: u32, deposit: &mut Deposit) {
        if deposit.state == DepositState::Valid {
            deposit.state = DepositState::InDispute;
            self.available_amount -= deposit.amount as i64;
            self.held_amount += deposit.amount;
        } else {
            eprintln!(
                "A dispute start failed because the referenced deposit was already \
                chargedback or is currently in an active dispute - tx_id: {tx_id} \
                - deposit state: {:?}",
                deposit.state
            );
        }
    }

    fn resolve_dispute(&mut self, tx_id: u32, deposit: &mut Deposit) {
        if deposit.state == DepositState::InDispute {
            deposit.state = DepositState::Valid;
            self.available_amount += deposit.amount as i64;
            self.held_amount -= deposit.amount;
        } else {
            eprintln!(
                "A dispute resolve failed because the referenced deposit wasn't in an \
                active dispute - tx_id: {tx_id} - deposit state: {:?}",
                deposit.state
            );
        }
    }

    fn chargeback(&mut self, tx_id: u32, deposit: &mut Deposit) {
        if deposit.state == DepositState::InDispute {
            deposit.state = DepositState::ChargedBack;
            self.held_amount -= deposit.amount;
            self.locked = true;
        } else {
            eprintln!(
                "A chargeback failed because the referenced deposit wasn't in an active \
                dispute - tx_id: {tx_id} - deposit state: {:?}",
                deposit.state
            );
        }
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
