#![cfg_attr(not(feature = "export-abi"), no_main)]

extern crate alloc;

use stylus_sdk::{
    alloy_primitives::U256,
    prelude::*,
    storage::{StorageAddress, StorageU256, StorageU8},
    msg, call, block,
};
use alloy_sol_types::sol;
use alloy_primitives::Address;

#[derive(Copy, Clone, PartialEq)]
pub enum Choice {
    None,
    Rock,
    Paper,
    Scissors,
}

impl From<u8> for Choice {
    fn from(value: u8) -> Self {
        match value {
            0 => Choice::None,
            1 => Choice::Rock,
            2 => Choice::Paper,
            3 => Choice::Scissors,
            _ => panic!("Invalid choice"),
        }
    }
}

impl From<Choice> for u8 {
    fn from(choice: Choice) -> Self {
        match choice {
            Choice::None => 0,
            Choice::Rock => 1,
            Choice::Paper => 2,
            Choice::Scissors => 3,
        }
    }
}

sol_storage! {
    #[entrypoint]
    pub struct RPS {
        mapping(address => uint256) player_balances;
        mapping(uint256 => address) player_addresses;
        mapping(uint256 => uint256) player_commitments;
        mapping(uint256 => uint256) player_choices;
        uint256 bet;
        uint256 deposit;
        uint256 revealSpan;
        uint256 revealDeadline;
        uint256 stage;
        bool locked;
    }
}

#[external]
impl RPS {
    pub fn new(&mut self, bet: U256, deposit: U256, revealSpan: U256) -> Result<(), Vec<u8>> {
        self.bet.set(bet);
        self.deposit.set(deposit);
        self.revealSpan.set(revealSpan);
        self.stage.set(U256::from(0)); // FirstCommit
        self.locked.set(false);
        Ok(())
    }

    pub fn lock(&mut self) -> Result<(), Vec<u8>> {
        self.locked.set(true);
        Ok(())
    }

    pub fn unlock(&mut self, stage: U256, player1: (Address, U256, U256), player2: (Address, U256, U256)) -> Result<(), Vec<u8>> {
        self.locked.set(false);
        self.stage.set(stage);
        self.player_addresses.insert(U256::from(0), player1.0);
        self.player_commitments.insert(U256::from(0), player1.1);
        self.player_choices.insert(U256::from(0), player1.2);
        self.player_addresses.insert(U256::from(1), player2.0);
        self.player_commitments.insert(U256::from(1), player2.1);
        self.player_choices.insert(U256::from(1), player2.2);
        Ok(())
    }

    #[payable]
    pub fn commit(&mut self, commitment: U256) -> Result<(), Vec<u8>> {
        if self.locked.get() {
            return Err("Contract is locked".into());
        }

        let mut player_index = U256::from(0);
        if self.stage.get() == U256::from(0) { // FirstCommit
            player_index = U256::from(0);
        } else if self.stage.get() == U256::from(1) { // SecondCommit
            player_index = U256::from(1);
        } else {
            return Err("Invalid stage for commit".into());
        }

        let commit_amount = self.bet.get() + self.deposit.get();
        if msg::value() < commit_amount {
            return Err("Insufficient funds committed".into());
        }

        if msg::value() > commit_amount {
            call::transfer_eth(msg::sender(), msg::value() - commit_amount)?;
        }

        self.player_addresses.insert(player_index, msg::sender());
        self.player_commitments.insert(player_index, commitment);
        self.player_choices.insert(player_index, U256::from(0)); // Choice::None

        if self.stage.get() == U256::from(0) { // FirstCommit
            self.stage.set(U256::from(1)); // SecondCommit
        } else {
            self.stage.set(U256::from(2)); // FirstReveal
        }

        Ok(())
    }

    pub fn reveal(&mut self, choice: u8, blinding_factor: U256) -> Result<(), Vec<u8>> {
        if self.locked.get() {
            return Err("Contract is locked".into());
        }
    
        if self.stage.get() != U256::from(2) && self.stage.get() != U256::from(3) {
            return Err("Invalid stage for reveal".into());
        }
    
        let choice = Choice::from(choice);
        if choice != Choice::Rock && choice != Choice::Paper && choice != Choice::Scissors {
            return Err("Invalid choice".into());
        }
    
        let mut player_index = U256::from(0);
        if self.player_addresses.get(U256::from(0)) == msg::sender() {
            player_index = U256::from(0);
        } else if self.player_addresses.get(U256::from(1)) == msg::sender() {
            player_index = U256::from(1);
        } else {
            return Err("Unknown player".into());
        }
    
        let commit_choice = self.player_commitments.get(player_index);
    
        if alloy_primitives::keccak256(msg::sender().as_bytes()) != commit_choice {
            return Err("Invalid hash".into());
        }
    
        self.player_choices.insert(player_index, U256::from(choice as u8));
    
        if self.stage.get() == U256::from(2) { // FirstReveal
            self.revealDeadline.set(block::number() + self.revealSpan.get());
            self.stage.set(U256::from(3)); // SecondReveal
        } else {
            self.stage.set(U256::from(4)); // Distribute
        }
    
        Ok(())
    }

    // pub fn distribute(&mut self) -> Result<(), Vec<u8>> {
    //     if self.stage.get() != Stage::Distribute.into() && (self.stage.get() != Stage::SecondReveal.into() || block::number() <= self.revealDeadline.get()) {
    //         return Err("Invalid stage for distribute".into());
    //     }

    //     let mut player0_payout = U256::ZERO;
    //     let mut player1_payout = U256::ZERO;
    //     let winning_amount = self.deposit.get() + self.bet.get() * U256::from(2);

    //     let player0_choice = Choice::from(self.players.get(0).unwrap().choice.get().as_u32() as u8);
    //     let player1_choice = Choice::from(self.players.get(1).unwrap().choice.get().as_u32() as u8);

    //     if player0_choice == player1_choice {
    //         player0_payout = self.deposit.get() + self.bet.get();
    //         player1_payout = self.deposit.get() + self.bet.get();
    //     } else if player0_choice == Choice::None {
    //         player1_payout = winning_amount;
    //     } else if player1_choice == Choice::None {
    //         player0_payout = winning_amount;
    //     } else {
    //         match (player0_choice, player1_choice) {
    //             (Choice::Rock, Choice::Scissors) | (Choice::Paper, Choice::Rock) | (Choice::Scissors, Choice::Paper) => {
    //                 player0_payout = winning_amount;
    //                 player1_payout = self.deposit.get();
    //             }
    //             (Choice::Rock, Choice::Paper) | (Choice::Paper, Choice::Scissors) | (Choice::Scissors, Choice::Rock) => {
    //                 player0_payout = self.deposit.get();
    //                 player1_payout = winning_amount;
    //             }
    //             _ => return Err("Invalid choices".into()),
    //         }
    //     }

    //     if player0_payout > U256::ZERO {
    //         if call::transfer_eth(self.players.get(0).unwrap().playerAddress.get(), player0_payout).is_ok() {
    //             evm::log(Payout {
    //                 player: self.players.get(0).unwrap().playerAddress.get(),
    //                 amount: player0_payout,
    //             });
    //         }
    //     }

    //     if player1_payout > U256::ZERO {
    //         if call::transfer_eth(self.players.get(1).unwrap().playerAddress.get(), player1_payout).is_ok() {
    //             evm::log(Payout {
    //                 player: self.players.get(1).unwrap().playerAddress.get(),
    //                 amount: player1_payout,
    //             });
    //         }
    //     }

    //     self.players.erase(0);
    //     self.players.erase(1);
    //     self.revealDeadline.set(U256::ZERO);
    //     self.stage.set(Stage::FirstCommit.into());

    //     Ok(())
    // }
}

sol! {
    event Payout(address indexed player, uint256 amount);
}