use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::{env, log, near_bindgen, require, AccountId, NearSchema, NearToken, Promise};

pub const STORAGE_COST: NearToken = NearToken::from_millinear(1);

use crate::Contract;
use crate::ContractExt;

#[derive(NearSchema, BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
#[abi(json, borsh)]
pub struct Donation {
    pub account_id: AccountId,
    pub total_amount: NearToken,
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn donate(&mut self) -> String {
        // Get who is calling the method and how much NEAR they attached
        let donor: AccountId = env::predecessor_account_id();
        let donation_amount = env::attached_deposit();

        require!(
            donation_amount > STORAGE_COST,
            format!(
                "Attach at least {} yoctoNEAR to cover for the storage cost",
                STORAGE_COST
            )
        );

        let mut donated_so_far: NearToken = self
            .donations
            .get(&donor)
            .unwrap_or(NearToken::from_near(0));

        let to_transfer = if donated_so_far.is_zero() {
            // This is the user's first donation, lets register it, which increases storage
            // Subtract the storage cost to the amount to transfer
            donation_amount.saturating_sub(STORAGE_COST).to_owned()
        } else {
            donation_amount
        };

        // Persist in storage the amount donated so far
        donated_so_far = donated_so_far.saturating_add(donation_amount);

        self.donations.insert(&donor, &donated_so_far);

        log!(
            "Thank you {} for donating {}! You donated a total of {}",
            donor.clone(),
            donation_amount,
            donated_so_far
        );

        // Send the NEAR to the beneficiary
        Promise::new(self.beneficiary.clone()).transfer(to_transfer);

        // Return the total amount donated so far
        donated_so_far.to_string()
    }

    // Public Method - get donation by account ID
    pub fn get_donation_for_account(&self, account_id: AccountId) -> Donation {
        Donation {
            account_id: account_id.clone(),
            total_amount: self
                .donations
                .get(&account_id)
                .unwrap_or(NearToken::from_near(0)),
        }
    }

    // Public Method - get total number of donors
    pub fn number_of_donors(&self) -> u64 {
        self.donations.len()
    }

    // Public Method - paginate through all donations on the contract
    pub fn get_donations(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<Donation> {
        // where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = from_index.unwrap_or(0);

        self.donations
            .into_iter()
            .skip(start as usize)
            .take(limit.unwrap_or(10) as usize)
            .map(|(account_id, total_amount)| Donation {
                account_id,
                total_amount,
            })
            .collect()
    }
}
