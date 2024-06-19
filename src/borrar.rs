// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]

extern crate alloc;

// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{alloy_primitives::U256, call, msg, prelude::*};

// Define some persistent storage using the Solidity ABI.
// `Contract` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct RPS { // The entrypoint necessarily has to have the same name as the contract since it is linked to it
        mapping(address => uint256) player_balances; // A mapping can't be erased, you can erase a primitive or a vector of primitives
    }
}

#[external]
impl RPS {
    #[payable] // This is a payable function, it can receive ETH as a call value
    pub fn fund_contract(&mut self) -> Result<(), Vec<u8>> {
        Ok(())
    }

    #[payable]
    pub fn deposit(&mut self) -> Result<(), Vec<u8>> {
        // Insert the sender's address into the player_balances mapping and add the msg value to their balance
        self.player_balances.insert(
            msg::sender(),
            self.player_balances.get(msg::sender()) + msg::value(),
        );
        Ok(())
    }

    pub fn withdraw(&mut self) -> Result<(), Vec<u8>> {
        // Get the player's balance
        let player_balance = self.player_balances.get(msg::sender());
        // Check if the balance is zero, if so return an error
        if player_balance.is_zero() {
            return Err("No balance to withdraw".into());
        }

        // Set the player's balance to zero
        self.player_balances.insert(msg::sender(), U256::ZERO);
        // Transfer the player's balance to their address
        call::transfer_eth(msg::sender(), player_balance)?;

        Ok(())
    }

    // Notice that since we are using rust we do not need to do string comparison using keccak256 and we can just use the str::eq method from the rust standard library
    pub fn play_game(
        &mut self,
        player_one_choice: String,
        player_two_choice: String,
        game_stake: U256,
    ) -> Result<U256, Vec<u8>> {
        // Get the sender's balance
        let balance = self.player_balances.get(msg::sender());
        // Check if the balance is less than the game stake, if so return an error
        if balance < game_stake {
            return Err("Not enough funds to place bet".into());
        }

        // Compare the player's choices and determine the result
        let rslt = if player_one_choice.eq("rock") && player_two_choice.eq("rock")
            || player_one_choice.eq("paper") && player_two_choice.eq("paper")
            || player_one_choice.eq("scissors") && player_two_choice.eq("scissors")
        {
            0 // Draw
        } else if player_one_choice.eq("scissors") && player_two_choice.eq("paper")
            || player_one_choice.eq("rock") && player_two_choice.eq("scissors")
            || player_one_choice.eq("paper") && player_two_choice.eq("rock")
        {
            // Player 1 wins, add the game stake to their balance
            self.player_balances
                .insert(msg::sender(), balance + game_stake);
            1
        } else if player_one_choice.eq("paper") && player_two_choice.eq("scissors")
            || player_one_choice.eq("scissors") && player_two_choice.eq("rock")
            || player_one_choice.eq("rock") && player_two_choice.eq("paper")
        {
            // Player 2 wins, subtract the game stake from player 1's balance
            self.player_balances
                .insert(msg::sender(), balance - game_stake);
            2
        } else {
            3 // Invalid choices
        };

        Ok(U256::from(rslt))
    }
}
