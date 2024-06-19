// Rock Paper Scissors Game Tutorial
//
// This tutorial demonstrates how to create a simple Rock Paper Scissors game using the Stylus SDK and Rust.
// The game allows two players to commit their choices (rock, paper, or scissors) and then determines the winner based on the classic rules of the game.
//
// Steps:
// 1. Define the `Choice` enum to represent the possible choices: None, Rock, Paper, or Scissors.
// 2. Implement the `From` trait for converting between `U256` and `Choice`.
// 3. Define the `RPS` struct using the `sol_storage!` macro to store the game state.
// 4. Implement the `new` function to initialize the game with a bet amount.
// 5. Implement the `lock` and `unlock` functions to control the game state.
// 6. Implement the `commit` function to allow players to commit their choices and place bets.
// 7. Implement the `distribute` function to determine the winner and distribute the winnings.
//
// Let's go through each step in detail:

#![cfg_attr(not(feature = "export-abi"), no_main)]

extern crate alloc;

use stylus_sdk::{
    alloy_primitives::U256,
    prelude::*,
    msg, call,
};

// Define the `Choice` enum to represent the possible choices in the game
// The choices are: None, Rock, Paper, or Scissors
#[derive(Copy, Clone, PartialEq)]
pub enum Choice {
    None,
    Rock,
    Paper,
    Scissors,
}

// Implement the `From` trait for converting from `U256` to `Choice`
// This allows us to convert a `U256` value to a `Choice` enum variant
impl From<U256> for Choice {
    fn from(value: U256) -> Self {
        if value == U256::from(0) {
            Choice::None
        } else if value == U256::from(1) {
            Choice::Rock
        } else if value == U256::from(2) {
            Choice::Paper
        } else if value == U256::from(3) {
            Choice::Scissors
        } else {
            panic!("Invalid choice"); // Panic if the value is not a valid choice
        }
    }
}

// Implement the `From` trait for converting from `Choice` to `U256`
// This allows us to convert a `Choice` enum variant to a `U256` value
impl From<Choice> for U256 {
    fn from(choice: Choice) -> Self {
        match choice {
            Choice::None => U256::from(0),
            Choice::Rock => U256::from(1),
            Choice::Paper => U256::from(2),
            Choice::Scissors => U256::from(3),
        }
    }
}

// Define the `RPS` struct using the `sol_storage!` macro
// This struct represents the storage layout of the contract
sol_storage! {
    #[entrypoint]
    pub struct RPS {
        mapping(address => uint256) player_balances; // Mapping to store player balances
        mapping(uint256 => uint256) player_choices; // Mapping to store player choices
        mapping(uint256 => address) player_addresses; // Mapping to store player addresses
        uint256 bet; // The bet amount for the game
        uint256 stage; // The current stage of the game
        bool locked; // Flag to indicate if the contract is locked
    }
}

// Implement the external functions for the `RPS` contract
#[external]
impl RPS {
    // The `new` function is used to initialize the contract
    // It takes the bet amount as a parameter and sets the initial state
    pub fn new(&mut self, bet: U256) -> Result<(), Vec<u8>> {
        self.bet.set(bet); // Set the bet amount
        self.stage.set(U256::from(0)); // Set the initial stage to FirstCommit
        self.locked.set(false); // Set the locked flag to false
        Ok(())
    }

    // The `lock` function is used to lock the contract
    // It sets the locked flag to true
    pub fn lock(&mut self) -> Result<(), Vec<u8>> {
        self.locked.set(true);
        Ok(())
    }

    // The `unlock` function is used to unlock the contract
    // It sets the locked flag to false
    pub fn unlock(&mut self) -> Result<(), Vec<u8>> {
        self.locked.set(false);
        Ok(())
    }

    // The `commit` function is used by players to commit their choices and place bets
    // It is marked as `#[payable]` to allow players to send Ether along with their commitments
    #[payable]
    pub fn commit(&mut self, choice: U256) -> Result<(), Vec<u8>> {
        if self.locked.get() {
            return Err("Contract is locked".into()); // Return an error if the contract is locked
        }

        let player_index = self.stage.get(); // Get the current player index based on the stage
        if player_index > U256::from(1) {
            return Err("Invalid stage for commit".into()); // Return an error if the stage is invalid for committing
        }

        if msg::value() < self.bet.get() {
            return Err("Insufficient funds committed".into()); // Return an error if the committed funds are insufficient
        }

        if msg::value() > self.bet.get() {
            // If the player sent more than the required bet amount, refund the excess amount
            call::transfer_eth(msg::sender(), msg::value() - self.bet.get())?;
        }

        self.player_choices.insert(player_index, choice); // Store the player's choice
        self.player_addresses.insert(player_index, msg::sender()); // Store the player's address

        self.stage.set(player_index + U256::from(1)); // Advance the stage to the next player or to the distribute stage

        Ok(())
    }

    // The `distribute` function is used to determine the winner and distribute the winnings
    pub fn distribute(&mut self) -> Result<(), Vec<u8>> {
        if self.stage.get() != U256::from(2) {
            return Err("Invalid stage for distribute".into()); // Return an error if the stage is not valid for distribution
        }

        // Get the choices made by the players
        let player0_choice = Choice::from(self.player_choices.get(U256::from(0)));
        let player1_choice = Choice::from(self.player_choices.get(U256::from(1)));

        // Determine the winner based on the choices made by the players
        let winner = match (player0_choice, player1_choice) {
            (Choice::Rock, Choice::Scissors) | (Choice::Paper, Choice::Rock) | (Choice::Scissors, Choice::Paper) => U256::from(0),
            (Choice::Rock, Choice::Paper) | (Choice::Paper, Choice::Scissors) | (Choice::Scissors, Choice::Rock) => U256::from(1),
            _ => return Err("Draw".into()), // Return an error if there is a draw
        };

        let winning_amount = self.bet.get() * U256::from(2); // Calculate the winning amount (2 times the bet)
        let winner_address = self.player_addresses.get(winner); // Get the address of the winner
        call::transfer_eth(winner_address, winning_amount)?; // Transfer the winnings to the winner

        self.stage.set(U256::from(0)); // Reset the stage to FirstCommit for a new game

        Ok(())
    }
}
