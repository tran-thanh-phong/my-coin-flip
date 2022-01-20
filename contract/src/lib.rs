/*
 * This is an example of a Rust smart contract with two simple, symmetric functions:
 *
 * 1. set_greeting: accepts a greeting, such as "howdy", and records it for the user (account_id)
 *    who sent the request
 * 2. get_greeting: accepts an account_id and returns the greeting saved for it, defaulting to
 *    "Hello"
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://github.com/near/near-sdk-rs
 *
 */

// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    env, near_bindgen, setup_alloc, AccountId, Balance,
    collections::{ UnorderedMap },
    json_types:: { U128 }
};

setup_alloc!();

const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
const PROB: u8 = 128;

// Structs in Rust are similar to other languages, and may include impl keyword as shown below
// Note: the names of the structs are not important when calling the smart contract, but the function names are
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct SlotMachine {
    owner_id: AccountId,
    credits: UnorderedMap<AccountId, Balance>
}

impl Default for SlotMachine {
    fn default() -> Self {
        panic!("Should be initialized before usage")
    }
}

#[near_bindgen]
impl SlotMachine {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(env::is_valid_account_id(&owner_id.as_bytes()), "Invalid owner account!");
        assert!(!env::state_exists(), "Already initialized!");

        env::log(format!("Creating a SlotMachine with owner id '{}'", &owner_id).as_bytes());

        Self {
            owner_id,
            credits: UnorderedMap::new(b"credits".to_vec()),
        }
    }

    #[payable]
    pub fn deposit(&mut self) {
        let account_id = env::signer_account_id();
        let deposit_amount = env::attached_deposit();

        let mut credits = self.credits.get(&account_id).unwrap_or(0);
        credits += deposit_amount;

        self.credits.insert(&account_id, &credits);
    }

    pub fn play(&mut self) -> u8{
        let account_id = env::signer_account_id();
        let mut credits = self.credits.get(&account_id).unwrap_or(0);

        assert!(credits >= ONE_NEAR, "No credits to play!!!");

        credits -= ONE_NEAR;
        let random_number = *env::random_seed().get(0).unwrap();
        if random_number < PROB {
            credits += 10 * ONE_NEAR;
        }

        self.credits.insert(&account_id, &credits);
        
        random_number
    }

    pub fn get_credits(&self, account_id: AccountId) -> U128 {
        println!("get_credits");
        self.credits.get(&account_id).unwrap_or(0).into()
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 *
 * To run from contract directory:
 * cargo test -- --nocapture
 *
 * From project root, to run in combination with frontend tests:
 * yarn test
 *
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    const DEPOSIT_AMOUNT: u128 = 10 * ONE_NEAR;

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: DEPOSIT_AMOUNT,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn deposit() {
        let context = get_context(vec![], false);
        testing_env!(context);
        
        let mut contract = SlotMachine::new(String::from("carol_near"));

        contract.deposit();

        assert_eq!(
            U128::from(DEPOSIT_AMOUNT),
            contract.get_credits(String::from("bob_near"))
        );

        assert_eq!(
            U128::from(0),
            contract.get_credits(String::from("carol_near"))
        );
    }

    #[test]
    fn play() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = SlotMachine::new(String::from("carol_near"));
        
        // Deposit 10 NEAR to 'bob_near'
        contract.deposit();

        let number = contract.play();
        let mut credits = DEPOSIT_AMOUNT;
        
        if number < 128 {
            credits += 10 * ONE_NEAR;
        }

        credits -= ONE_NEAR;

        assert_eq!(
            U128::from(credits),
            contract.get_credits(String::from("bob_near"))
        );
    }

    #[test]
    fn get_initial_credits() {
        let context = get_context(vec![], true);
        testing_env!(context);
        let contract = SlotMachine::new(String::from("carol_near"));
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(
            U128::from(0),
            contract.get_credits(String::from("bob_near"))
        );
    }
}
