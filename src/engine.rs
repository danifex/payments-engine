use crate::transaction::{Transaction, TransactionType};
use crate::util::{fixed_point_4_decimal_to_float_str, signed_fixed_point_4_decimal_to_float_str};
use std::collections::HashMap;

pub struct Engine {
    accounts: HashMap<u16, Account>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn process_transaction(&mut self, transaction: Transaction) {
        match transaction.transaction_type {
            TransactionType::Deposit => self.process_deposit_transaction(transaction),
            TransactionType::Withdrawal => self.process_withdrawal_transaction(transaction),
            TransactionType::Dispute => self.process_dispute_transaction(transaction),
            TransactionType::Resolve => self.process_resolve_transaction(transaction),
            TransactionType::Chargeback => self.process_chargeback_transaction(transaction),
        }
    }

    fn process_deposit_transaction(&mut self, transaction: Transaction) {
        debug_assert_eq!(transaction.transaction_type, TransactionType::Deposit);

        self.accounts
            .entry(transaction.client_id)
            .and_modify(|a| a.deposit(transaction.tx_id, transaction.amount.unwrap()))
            .or_insert({
                let mut account = Account::new();
                account.deposit(transaction.tx_id, transaction.amount.unwrap());
                account
            });
    }

    fn process_withdrawal_transaction(&mut self, transaction: Transaction) {
        debug_assert_eq!(transaction.transaction_type, TransactionType::Withdrawal);

        self.accounts
            .entry(transaction.client_id)
            .and_modify(|a| a.withdraw(transaction.amount.unwrap()));
    }

    fn process_dispute_transaction(&mut self, transaction: Transaction) {
        debug_assert_eq!(transaction.transaction_type, TransactionType::Dispute);

        self.accounts
            .entry(transaction.client_id)
            .and_modify(|a| a.start_dispute(transaction.tx_id));
    }

    fn process_resolve_transaction(&mut self, transaction: Transaction) {
        debug_assert_eq!(transaction.transaction_type, TransactionType::Resolve);

        self.accounts
            .entry(transaction.client_id)
            .and_modify(|a| a.resolve_dispute(transaction.tx_id));
    }

    fn process_chargeback_transaction(&mut self, transaction: Transaction) {
        debug_assert_eq!(transaction.transaction_type, TransactionType::Chargeback);

        self.accounts
            .entry(transaction.client_id)
            .and_modify(|a| a.chargeback(transaction.tx_id));
    }

    pub fn print_state_csv(&self) {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());

        wtr.write_record(["client", "available", "held", "total", "locked"])
            .unwrap();

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
            ))
            .unwrap();
        }

        wtr.flush().unwrap();
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

    fn deposit(&mut self, tx_id: u32, amount: u64) {
        if self.locked {
            return;
        }

        self.deposits.insert(
            tx_id,
            Deposit {
                amount,
                state: DepositState::Valid,
            },
        );

        self.available_amount += amount as i64;
    }

    fn withdraw(&mut self, amount: u64) {
        if self.locked {
            return;
        }

        if self.available_amount >= amount as i64 {
            self.available_amount -= amount as i64
        }
    }

    fn start_dispute(&mut self, tx_id: u32) {
        if self.locked {
            return;
        }

        let deposit = self.deposits.get_mut(&tx_id);

        if let Some(deposit) = deposit {
            if deposit.state == DepositState::Valid {
                deposit.state = DepositState::InDispute;
                self.available_amount -= deposit.amount as i64;
                self.held_amount += deposit.amount;
            }
        }
    }

    fn resolve_dispute(&mut self, tx_id: u32) {
        if self.locked {
            return;
        }

        let deposit = self.deposits.get_mut(&tx_id);

        if let Some(deposit) = deposit {
            if deposit.state == DepositState::InDispute {
                deposit.state = DepositState::Valid;
                self.available_amount += deposit.amount as i64;
                self.held_amount -= deposit.amount;
            }
        }
    }

    fn chargeback(&mut self, tx_id: u32) {
        if self.locked {
            return;
        }

        let deposit = self.deposits.get_mut(&tx_id);

        if let Some(deposit) = deposit {
            if deposit.state == DepositState::InDispute {
                deposit.state = DepositState::ChargedBack;
                self.held_amount -= deposit.amount;
                self.locked = true;
            }
        }
    }
}

struct Deposit {
    amount: u64,
    state: DepositState,
}

#[derive(PartialEq)]
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
        account.deposit(1, 20);
        account.deposit(2, 40);
        assert_eq!(account.available_amount, 60);
        assert_eq!(account.held_amount, 0);

        // Check disputing tx 1
        account.start_dispute(1);
        assert_eq!(account.available_amount, 40);
        assert_eq!(account.held_amount, 20);

        // Check resolving tx 1
        account.resolve_dispute(1);
        assert_eq!(account.available_amount, 60);
        assert_eq!(account.held_amount, 0);

        // Check dispute can be started again + double dispute is fine
        account.start_dispute(1);
        account.start_dispute(1);
        assert_eq!(account.available_amount, 40);
        assert_eq!(account.held_amount, 20);

        // Check having multiple in-progress disputes
        account.start_dispute(2);
        assert_eq!(account.available_amount, 0);
        assert_eq!(account.held_amount, 60);

        // Resolve all disputes
        account.resolve_dispute(1);
        account.resolve_dispute(2);
        assert_eq!(account.available_amount, 60);
        assert_eq!(account.held_amount, 0);

        // Chargeback non disputed tx does nothing
        account.chargeback(1);
        assert_eq!(account.available_amount, 60);
        assert_eq!(account.held_amount, 0);

        // Check chargeback
        account.start_dispute(1);
        account.chargeback(1);
        assert_eq!(account.available_amount, 40);
        assert_eq!(account.held_amount, 0);
        assert!(account.locked);
    }

    #[test]
    fn test_account_chargeback_after_withdrawal_flow() {
        let mut account = Account::new();
        account.deposit(1, 100);
        account.deposit(2, 50);
        assert_eq!(account.available_amount, 150);
        assert_eq!(account.held_amount, 0);

        account.withdraw(100);
        assert_eq!(account.available_amount, 50);
        assert_eq!(account.held_amount, 0);

        account.start_dispute(1);
        assert_eq!(account.available_amount, -50);
        assert_eq!(account.held_amount, 100);

        account.deposit(3, 25);
        assert_eq!(account.available_amount, -25);
        assert_eq!(account.held_amount, 100);

        account.start_dispute(3);
        assert_eq!(account.available_amount, -50);
        assert_eq!(account.held_amount, 125);

        account.resolve_dispute(3);
        assert_eq!(account.available_amount, -25);
        assert_eq!(account.held_amount, 100);

        account.chargeback(1);
        assert_eq!(account.available_amount, -25);
        assert_eq!(account.held_amount, 0);
        assert!(account.locked);
    }
}