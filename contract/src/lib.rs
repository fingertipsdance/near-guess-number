use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Promise};
//use rand::Rng;
use std::collections::HashMap;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub guess_history: HashMap<AccountId, Vec<GuessRecord>>,
}

const interval_minimum: u32 = 1; // interval minimum

#[derive(
    BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug, Clone, Copy,
)]
#[serde(crate = "near_sdk::serde")]
pub struct GuessRecord {
    created: u64,  // create time
    number: u32,   // bet number
    maximum: u32,  // interval maximum
    win_num: u32,  // win number
    deposit: U128, // bet amount
    reward: U128,  // reward amount
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn guess_number(&mut self, number: u32, maximum: u32) {
        let owner_id = env::predecessor_account_id();
        //let mut rng = rand::thread_rng();
        //let win_num = rng.gen_range(1..=maximum);
        let win_num = self.random_win_num(interval_minimum, maximum);
        // deposit amount
        let deposit = env::attached_deposit();
        // reward amount
        let reward = deposit * maximum as u128;
        assert!(
            reward <= env::account_balance(),
            "transaction cancel, the contract balance({}) is insufficient for payment({})",
            env::account_balance(),
            reward
        );
        // win transfer reward
        if number == win_num {
            Promise::new(owner_id.clone()).transfer(reward);
        }

        // history record
        let record = GuessRecord {
            number,
            maximum,
            win_num,
            created: env::block_timestamp(),
            deposit: U128(deposit),
            reward: U128(reward),
        };

        if let Some(records) = self.guess_history.get_mut(&owner_id) {
            records.push(record);
        } else {
            self.guess_history.insert(owner_id, vec![record]);
        }
    }

    // random a number in [minimum, maximum]
    pub fn random_win_num(&self, minimum: u32, maximum: u32) -> u32 {
        assert!(maximum > 0, "maximum need greater then zero");

        let seed = env::random_seed();
        let mut arr: [u8; 8] = Default::default();
        arr.copy_from_slice(&seed[..8]);
        let seed_num: u64 = u64::from_le_bytes(arr);
        (seed_num % maximum as u64 + minimum as u64) as u32
    }

    // guess hisory record
    pub fn guess_history(
        &self,
        account_id: AccountId,
        size: usize,
        offset: usize,
    ) -> Result<Vec<GuessRecord>, String> {
        if let Some(records) = self.guess_history.get(&account_id) {
            if records.len() == 0 {
                return Ok(vec![]);
            }
            // history record len
            let history_len = records.len();
            // query start
            let start = if history_len > (size + offset) {
                history_len - (size + offset)
            } else {
                0
            };
            // query end
            let end = if start + size < history_len - 1 {
                start + size
            } else {
                if history_len < offset {
                    0
                } else {
                    history_len - offset
                }
            };

            println!("start: {}, end: {}", start, end);

            let mut history_recent = records[start..end].to_vec();
            history_recent.reverse();

            Ok(history_recent.to_vec())
        } else {
            Ok(vec![])
            //Err("nodata".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::{
        json_types::ValidAccountId,
        test_utils::{accounts, VMContextBuilder},
        testing_env, MockedBlockchain,
    };

    // signer account index
    const SIGNER_ACCOUNT_INDEX: usize = 1;

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.current_account_id(accounts(0));
        builder.signer_account_id(predecessor_account_id.clone());
        builder.predecessor_account_id(predecessor_account_id);

        builder
    }
    // init default contract
    fn init_contract_default() -> Contract {
        let context = get_context(accounts(SIGNER_ACCOUNT_INDEX));
        testing_env!(context.build());
        Contract::default()
    }

    #[test]
    fn test_win_num() {
        let contract = init_contract_default();

        let cases = vec![1, 2, 28, 99, 100];
        for maximum in cases {
            assert!((1..=maximum).contains(&contract.random_win_num(interval_minimum, maximum)))
        }
    }

    #[test]
    fn test_guess_history_save() {
        let mut contract = init_contract_default();

        contract.guess_number(1, 1);
        assert_eq!(1, contract.guess_history.len())
    }

    #[test]
    fn test_guess_history_record() {
        let mut contract = init_contract_default();

        for i in 1..4 {
            contract.guess_number(i, i * i);
        }

        if let Some(records) = contract
            .guess_history
            .get(accounts(SIGNER_ACCOUNT_INDEX).as_ref())
        {
            assert_eq!(3, records.len());
            assert_eq!(1, records[0].number);
            assert_eq!(1, records[0].maximum);
            assert_eq!(2, records[1].number);
            assert_eq!(4, records[1].maximum);
            assert_eq!(3, records[2].number);
            assert_eq!(9, records[2].maximum);
        } else {
            panic!("not found history record");
        }
    }

    #[test]
    fn test_guess_history_query() {
        let mut contract = init_contract_default();

        for i in 1..=3 {
            contract.guess_number(i, i * i);
        }

        if let Ok(history) =
            contract.guess_history(accounts(SIGNER_ACCOUNT_INDEX).to_string(), 2, 1)
        {
            println!("{:?}", history);
            assert_eq!(2, history.len());
            assert_eq!(2, history[0].number);
            assert_eq!(1, history[1].number);
        }
        if let Ok(history) =
            contract.guess_history(accounts(SIGNER_ACCOUNT_INDEX).to_string(), 5, 3)
        {
            println!("{:?}", history);
            assert_eq!(0, history.len());
        }
    }

    #[test]
    fn test_guess_number_win() {
        let mut contract = init_contract_default();

        let cases = vec![(1, 1), (1, 2), (1, 28), (1, 99), (1, 100)];
        for case in cases.iter() {
            contract.guess_number(case.0, case.1);
        }

        if let Some(records) = contract
            .guess_history
            .get(accounts(SIGNER_ACCOUNT_INDEX).as_ref())
        {
            let mut iterator = records.iter();
            for case in cases.iter() {
                assert!((case.0..=case.1).contains(&iterator.next().unwrap().win_num));
            }
            assert_eq!(iterator.next(), None);
        } else {
            panic!("not found history record");
        }
    }
}
