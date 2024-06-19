# Creating Your First Stylus Smart Contract in Rust

This tutorial will guide you through the process of writing and deploying your first smart contract using the Stylus SDK and Rust programming language. We'll create a simple Rock Paper Scissors game contract that allows two players to commit their choices and determines the winner based on the classic rules of the game.

## Prerequisites

Before getting started, make sure you have the following prerequisites set up:

1. Rust toolchain: Install the Rust programming language and its package manager, Cargo, by following the instructions on the official Rust website (https://www.rust-lang.org/tools/install).

2. Stylus CLI: Install the Stylus command-line interface (CLI) tool, which simplifies the process of building, verifying, and deploying Stylus contracts. Run the following command to install it:

   ```
   cargo install --force cargo-stylus cargo-stylus-check
   ```

3. WASM target: Add the WebAssembly (WASM) target to your Rust compiler by running:

   ```
   rustup target add wasm32-unknown-unknown
   ```

4. Developer wallet: Set up a separate developer wallet for testing and deploying contracts on the Stylus testnet. Follow the steps in the [Quickstart tutorial](https://docs.arbitrum.io/stylus/stylus-quickstart) to create a new account in MetaMask and acquire testnet ETH.

## Step 1: Create a New Stylus Project

Start by creating a new Stylus project using the Stylus CLI. Open your terminal and run the following command:

```
cargo stylus new rps-game
```

This command will create a new directory called `rps-game` with the necessary project structure and files.

## Step 2: Define the Game Logic

In this step, we'll dive into the details of defining the game logic for our Rock Paper Scissors contract. Open the `src/lib.rs` file in your preferred text editor or IDE.

First, we need to define an enum to represent the possible choices in the game. Add the following code at the top of the file:

```rust
#[derive(Copy, Clone, PartialEq)]
pub enum Choice {
    None,
    Rock,
    Paper,
    Scissors,
}
```

The `Choice` enum has four variants: `None`, `Rock`, `Paper`, and `Scissors`. We derive the `Copy`, `Clone`, and `PartialEq` traits to allow the enum to be copied, cloned, and compared for equality.

Next, we need to implement the `From` trait for converting between `U256` (a 256-bit unsigned integer) and `Choice`. Add the following code after the `Choice` enum:

```rust
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
            panic!("Invalid choice");
        }
    }
}

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
```

These implementations allow us to convert between `U256` and `Choice` easily. When converting from `U256` to `Choice`, we map the values 0, 1, 2, and 3 to the corresponding `Choice` variants. If an invalid value is provided, we panic with an error message. When converting from `Choice` to `U256`, we map each variant to its corresponding numeric value.

Now, let's define the storage layout for our contract using the `sol_storage!` macro. Add the following code after the `From` trait implementations:

```rust
sol_storage! {
    #[entrypoint]
    pub struct RPS {
        mapping(address => uint256) player_balances;
        mapping(uint256 => uint256) player_choices;
        mapping(uint256 => address) player_addresses;
        uint256 bet;
        uint256 stage;
        bool locked;
    }
}
```

The `sol_storage!` macro allows us to define the storage layout of our contract using Solidity-like syntax. We define a struct called `RPS` with the following fields:

- `player_balances`: A mapping from player addresses to their balances.
- `player_choices`: A mapping from player indices to their choices.
- `player_addresses`: A mapping from player indices to their addresses.
- `bet`: The bet amount for the game.
- `stage`: The current stage of the game.
- `locked`: A flag indicating whether the contract is locked.

The `#[entrypoint]` attribute specifies that the `RPS` struct is the entry point of our contract.

With the game logic defined, we're ready to move on to implementing the contract methods.

## Step 3: Implement the Contract Methods

In this step, we'll implement the external methods for our Rock Paper Scissors contract. These methods will allow players to interact with the contract and play the game.

Add the following code after the `sol_storage!` macro:

```rust
#[external]
impl RPS {
    // Implement the contract methods here
}
```

The `#[external]` attribute indicates that the methods inside the `impl` block are externally accessible and can be called by other contracts or users.

Let's implement each method one by one:

1. `new` method:
```rust
pub fn new(&mut self, bet: U256) -> Result<(), Vec<u8>> {
    self.bet.set(bet);
    self.stage.set(U256::from(0));
    self.locked.set(false);
    Ok(())
}
```
The `new` method is used to initialize the contract with a bet amount. It takes a `U256` parameter representing the bet amount and sets the initial state of the contract. It performs the following actions:
- Sets the `bet` amount using `self.bet.set(bet)`.
- Initializes the `stage` to 0 using `self.stage.set(U256::from(0))`.
- Sets the `locked` flag to `false` using `self.locked.set(false)`.
- Returns `Ok(())` to indicate successful initialization.

2. `lock` method:
```rust
pub fn lock(&mut self) -> Result<(), Vec<u8>> {
    self.locked.set(true);
    Ok(())
}
```
The `lock` method is used to lock the contract, preventing further commits from players. It sets the `locked` flag to `true` using `self.locked.set(true)` and returns `Ok(())` to indicate success.

3. `unlock` method:
```rust
pub fn unlock(&mut self) -> Result<(), Vec<u8>> {
    self.locked.set(false);
    Ok(())
}
```
The `unlock` method is used to unlock the contract, allowing players to commit their choices. It sets the `locked` flag to `false` using `self.locked.set(false)` and returns `Ok(())` to indicate success.

4. `commit` method:
```rust
#[payable]
pub fn commit(&mut self, choice: U256) -> Result<(), Vec<u8>> {
    if self.locked.get() {
        return Err("Contract is locked".into());
    }

    let player_index = self.stage.get();
    if player_index > U256::from(1) {
        return Err("Invalid stage for commit".into());
    }

    if msg::value() < self.bet.get() {
        return Err("Insufficient funds committed".into());
    }

    if msg::value() > self.bet.get() {
        call::transfer_eth(msg::sender(), msg::value() - self.bet.get())?;
    }

    self.player_choices.insert(player_index, choice);
    self.player_addresses.insert(player_index, msg::sender());

    self.stage.set(player_index + U256::from(1));

    Ok(())
}
```
The `commit` method allows players to commit their choices and place bets. It is marked as `#[payable]`, which means it can receive Ether along with the function call. Here's how it works:
- It first checks if the contract is locked using `self.locked.get()`. If the contract is locked, it returns an error with the message "Contract is locked".
- It retrieves the current player index based on the `stage` using `self.stage.get()`.
- It ensures that the current stage is valid for committing (0 or 1) by checking if `player_index` is greater than 1. If it is, it returns an error with the message "Invalid stage for commit".
- It checks if the committed funds (`msg::value()`) are less than the required bet amount (`self.bet.get()`). If the funds are insufficient, it returns an error with the message "Insufficient funds committed".
- If the player sends more than the required bet amount, the excess amount is refunded using `call::transfer_eth(msg::sender(), msg::value() - self.bet.get())?`.
- The player's choice (`choice`) and address (`msg::sender()`) are stored in the corresponding mappings using `self.player_choices.insert(player_index, choice)` and `self.player_addresses.insert(player_index, msg::sender())`.
- The `stage` is advanced to the next player or the distribute stage by incrementing `player_index` by 1 using `self.stage.set(player_index + U256::from(1))`.
- Finally, it returns `Ok(())` to indicate a successful commit.

5. `distribute` method:
```rust
pub fn distribute(&mut self) -> Result<(), Vec<u8>> {
    if self.stage.get() != U256::from(2) {
        return Err("Invalid stage for distribute".into());
    }

    let player0_choice = Choice::from(self.player_choices.get(U256::from(0)));
    let player1_choice = Choice::from(self.player_choices.get(U256::from(1)));

    let winner = match (player0_choice, player1_choice) {
        (Choice::Rock, Choice::Scissors) | (Choice::Paper, Choice::Rock) | (Choice::Scissors, Choice::Paper) => U256::from(0),
        (Choice::Rock, Choice::Paper) | (Choice::Paper, Choice::Scissors) | (Choice::Scissors, Choice::Rock) => U256::from(1),
        _ => return Err("Draw".into()),
    };

    let winning_amount = self.bet.get() * U256::from(2);
    let winner_address = self.player_addresses.get(winner);
    call::transfer_eth(winner_address, winning_amount)?;

    self.stage.set(U256::from(0));

    Ok(())
}
```
The `distribute` method is responsible for determining the winner and distributing the winnings. Here's how it works:
- It first checks if the current stage is valid for distribution (stage 2) by comparing `self.stage.get()` with `U256::from(2)`. If the stage is not 2, it returns an error with the message "Invalid stage for distribute".
- It retrieves the choices made by both players using `self.player_choices.get(U256::from(0))` and `self.player_choices.get(U256::from(1))`, and converts them from `U256` to `Choice` using the `From` trait.
- The winner is determined based on the classic rules of Rock Paper Scissors using a `match` expression. If player 0 wins, `U256::from(0)` is returned. If player 1 wins, `U256::from(1)` is returned. If there is a draw, an error with the message "Draw" is returned.
- The winning amount is calculated as twice the bet amount using `self.bet.get() * U256::from(2)`.
- The winner's address is retrieved from the `player_addresses` mapping using `self.player_addresses.get(winner)`.
- The winnings are transferred to the winner using `call::transfer_eth(winner_address, winning_amount)?`.
- The `stage` is reset to 0 for a new game using `self.stage.set(U256::from(0))`.
- Finally, it returns `Ok(())` to indicate a successful distribution.

These methods cover the core functionality of the Rock Paper Scissors game. Players can commit their choices and place bets using the `commit` method, and the winner is determined and winnings are distributed using the `distribute` method. The `lock` and `unlock` methods provide additional control over the game state.

With the contract methods implemented, you can now proceed to check the contract's validity and deploy it to the Stylus network as described in the previous steps.


## Step 4: Check the Contract Validity

Before deploying the contract, let's check if it's valid and can be activated on the Stylus network. Run the following command in your terminal:

```
cargo stylus check
```

If the contract passes the validation, you should see a success message.

## Step 5: Deploy the Contract

To deploy the contract to the Stylus testnet, you'll need to have some testnet ETH in your developer wallet. Follow the steps in the Quickstart tutorial to acquire and bridge testnet ETH to your wallet.

Once you have testnet ETH, run the following command to deploy the contract:

```
cargo stylus deploy --private-key-path=<PRIVKEY_FILE_PATH>
```

Replace `<PRIVKEY_FILE_PATH>` with the path to a file containing your private key.

The deployment process will estimate the gas required and send two transactions: one for deployment and another for activation. Once the transactions are confirmed, your contract will be live on the Stylus testnet.

Congratulations! You've successfully written and deployed your first Stylus smart contract using Rust. You can now interact with the contract using the exported Solidity ABI or by calling the methods directly from Rust.

For more advanced topics and features, refer to the Stylus SDK documentation and explore the various examples and tutorials available in the Stylus repository.

Remember to always test your contracts thoroughly and handle errors appropriately before deploying them to a production environment.

Happy coding with Stylus and Rust!