# Logos Execution Zone (LEZ)

Logos Execution Zone (LEZ) is a programmable blockchain that cleanly separates public and private state while keeping them fully interoperable. Developers can build apps that operate across transparent and privacy-preserving accounts without changing their logic. Privacy is enforced by the protocol itself through zero-knowledge proofs (ZKPs), so it is always available and automatic.


## Background

These features are provided by the Logos Execution Environment (LEE). Traditional public blockchains expose a fully transparent state: the mapping from account IDs to account values is entirely visible. LEE introduces a parallel *private state* that coexists with the public one. Together, public and private accounts form a partition of the account ID space: public IDs are visible on-chain, while private accounts are accessible only to holders of the corresponding viewing keys. Consistency across both states is enforced by ZKPs.

Public accounts are stored on-chain as a visible map from IDs to account states, and their values are updated in place. Private accounts are never stored on-chain in raw form. Each update produces a new commitment that binds the current value while keeping it hidden. Previous commitments remain on-chain, but a nullifier set marks old versions as spent, ensuring that only the most recent private state can be used in execution.


### Programmability and selective privacy

LEZ aims to deliver full programmability in a hybrid public/private model, with the same flexibility and composability as public blockchains. Developers write and deploy programs in LEZ without addressing privacy concerns. The protocol automatically supports executions that involve any combination of public and private accounts. From the program’s perspective, all accounts look the same, and privacy is enforced transparently. This lets developers focus on business logic while the system guarantees privacy and correctness.

To our knowledge, this design is unique to LEZ. Other privacy-focused programmable blockchains often require developers to explicitly handle private inputs inside their app logic. In LEZ, privacy is protocol-level: programs do not change, accounts are treated uniformly, and private execution works out of the box.

## LP-0015: General Cross-Program Calls via Tail Calls (CPS)
This repository includes a fully functional, zero-knowledge compatible Continuation-Passing Style (CPS) mechanism. It allows programs to make general calls (call another program and return) while keeping tail calls as the only underlying primitive. 

This is achieved using **Unforgeable Capability Tickets** evaluated at the host/sequencer level, ensuring safe execution without sacrificing encapsulation.

### Developer Usage (SDK)
Developers can easily declare public vs internal entrypoints and compose call chains using the provided `lez_sdk_macros`.

**1. Declaring Entrypoints:**
Use the `lez_dispatcher!` macro to route functions safely:
```bash
lez_dispatcher! {
    public: [ start_chain ],
    internal: [ continue_chain ] // Cannot be directly called by users
}
```

**2. Composing a Call Chain (Program A):**
To call Program B and return to an internal function in Program A, use `call_program!`. This automatically generates a secure capability ticket and suspends the VM:
```bash
#[public]
fn start_chain(ctx: ExecCtx, (amount, target_b_id): (u64, ProgramId)) -> Vec<AccountPostState> {
    let local_state = MyContext { initial_balance: 1000 };
    
    call_program!(
        ctx: ctx,
        target: target_b_id,
        func: process_funds(amount) => then continue_chain(local_state)
    );
}
```

**3. Returning to Caller (Program B):**
Program B processes the data and uses `return_to_caller!` to pass control and the ticket back to Program A:
```bash
#[public]
fn process_funds(ctx: ExecCtx, amount: u64) -> Vec<AccountPostState> {
    let instruction: GeneralCallInstruction = risc0_zkvm::serde::from_slice(&ctx.raw_instruction_data).unwrap();
    let route = instruction.route.unwrap();
    
    let is_success = amount > 0;
    return_to_caller!(ctx: ctx, route: route, result: is_success);
}
```

### Running the End-to-End Demo (with ZK Proofs)
To evaluate the cross-program invocation demo, you must run the components in standalone mode with **RISC0_DEV_MODE=0** (proving fully enabled).

Open 5 separate terminals from the root directory:

**Terminal 1: Start Bedrock (Logos L1 Node)**
```Bash
cd bedrock
docker compose up
```

**Terminal 2: Start the Sequencer**
```Bash
cd sequencer/service
RUST_LOG=info RISC0_DEV_MODE=0 cargo run --release -p sequencer_service configs/debug/sequencer_config.json
```

**Terminal 3: Start the Indexer**
```Bash
cd indexer/service
RUST_LOG=info RISC0_DEV_MODE=0 cargo run --release -p indexer_service configs/indexer_config.json
```
**Terminal 4: Deploy Program A & B**
```Bash
just run-wallet deploy-program ../artifacts/test_program_methods/demo_call_b.bin

```
**Terminal 5: Run the E2E Client Demo**
``` Bash
RUST_LOG=info RISC0_DEV_MODE=0 cargo run --release --bin run_general_calls_demo
```

### Demo Expected Output:
1. **Scenario 1 (Accepted):** A successful multi-hop execution (User -> A -> B -> A). The terminal will wait for ZK Proof generation.

2. **Scenario 2 (Rejected):** A simulated attack where a user directly calls the `#[internal]` continuation handler. It deterministically fails and is rejected by the sequencer.

### Running Tests
To run the integration tests covering Capability Checks, Replay Attacks, Forged Tickets, and Direct-Call prevention:
```Bash
cd integration_tests
RUST_LOG=info RISC0_DEV_MODE=1 cargo run $(pwd)/configs/debug all
# Or simply run the standard test suite:
cargo test --package integration_tests --test general_calls
```
---

## Example: Creating and transferring tokens across states

1. Token creation (public execution)
   - Alice submits a transaction that executes the token program `New` function on-chain.
   - A new public token definition account is created.
   - The minted tokens are recorded on-chain in Alice’s public account.

2. Transfer from public to private (local / privacy-preserving execution)
   - Alice runs the token program `Transfer` function locally, sending to Bob’s private account.
   - A ZKP of correct execution is generated.
   - The proof is submitted to the blockchain and verified by validators.
   - Alice’s public balance is updated on-chain.
   - Bob’s private balance remains hidden, while the transfer is provably correct.

3. Transferring private to public (local / privacy-preserving execution)
   - Bob executes the token program `Transfer` function locally, sending to Charlie’s public account.
   - A ZKP of correct execution is generated.
   - Bob’s private account and balance still remain hidden.
   - Charlie's public account is modified with the new tokens added.
4. Transferring public to public (public execution):
   - Alice submits a transaction to execute the token program `Transfer` function on-chain, specifying Charlie's public account as recipient.
   - The execution is handled on-chain without ZKPs involved.
   - Alice's and Charlie's accounts are modified according to the transaction.

4. Transfer from public to public (public execution)
   - Alice submits an on-chain transaction to run `Transfer`, sending to Charlie’s public account.
   - Execution is handled fully on-chain without ZKPs.
   - Alice’s and Charlie’s public balances are updated.


### Key points:
- The same token program is used in every execution.
- The only difference is execution mode: public execution updates visible state on-chain, while private execution relies on ZKPs.
- Validators verify proofs only for privacy-preserving transactions, keeping processing efficient.

---

## The account’s model

To achieve both state separation and full programmability, LEZ uses a stateless program model. Programs hold no internal state. All persistent data is stored in accounts passed explicitly into each execution. This enables precise access control and visibility while preserving composability across public and private states.

### Execution types

LEZ supports two execution types:
- Public execution runs transparently on-chain.
- Private execution runs off-chain and is verified on-chain with ZKPs.

Both public and private executions use the same Risc0 VM bytecode. Public transactions are executed directly on-chain like any standard RISC-V VM call, without proof generation. Private transactions are executed locally by users, who generate Risc0 proofs that validators verify instead of re-executing the program.

This design keeps public transactions as fast as any RISC-V–based VM and makes private transactions efficient for validators. It also supports parallel execution similar to Solana, improving throughput. The main computational cost for privacy-preserving transactions is on the user side, where ZK proofs are generated.

---
---
---

# Versioning

We release versions as git tags (e.g. `v0.1.0`). If no critical issues with version is found you can expect it to be immutable. All further features and fixes will be a part of the next tag. As the project is in active development we don't provide backward compatibility yet.
For each tag we publish docker images of our services.
If you depend on this project you can pin your rust dependency to a git tag like this:

```toml
nssa_core = { git = "https://github.com/logos-blockchain/logos-execution-zone.git", tag = "v0.1.0" }
```

# Install dependencies
### Install build dependencies

- On Linux
Ubuntu / Debian
```sh
apt install build-essential clang libclang-dev libssl-dev pkg-config
```

- On Fedora
```sh
sudo dnf install clang clang-devel openssl-devel pkgconf
```

- On Mac
```sh
xcode-select --install
brew install pkg-config openssl
```

### Install Rust

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install Risc0

```sh
curl -L https://risczero.com/install | bash
```

### Then restart your shell and run
```sh
rzup install
```

# Run tests

The LEZ repository includes both unit and integration test suites.

### Unit tests

```bash
# RISC0_DEV_MODE=1 is used to skip proof generation and reduce test runtime overhead
RISC0_DEV_MODE=1 cargo test --release
```

### Integration tests

```bash
export NSSA_WALLET_HOME_DIR=$(pwd)/integration_tests/configs/debug/wallet/
cd integration_tests
# RISC0_DEV_MODE=1 skips proof generation; RUST_LOG=info enables runtime logs
RUST_LOG=info RISC0_DEV_MODE=1 cargo run $(pwd)/configs/debug all
```

# Run the sequencer and node
## Running Manually
### Normal mode
The sequencer and logos blockchain node can be run locally:
 1. On one terminal go to the `logos-blockchain/logos-blockchain` repo and run a local logos blockchain node:
    - `git checkout master; git pull`
    - `cargo clean`
    - `rm -r ~/.logos-blockchain-circuits`
    - `./scripts/setup-logos-blockchain-circuits.sh`
    - `cargo build --all-features`
    - `./target/debug/logos-blockchain-node --deployment nodes/node/standalone-deployment-config.yaml nodes/node/standalone-node-config.yaml`

 - Alternatively (WARNING: This node is outdated) go to `logos-blockchain/lssa/` repo and run the node from docker:
    - `cd bedrock`
    - Change line 14 of `docker-compose.yml` from `"0:18080/tcp"` into `"8080:18080/tcp"`
    - `docker compose up`

 2. On another terminal go to the `logos-blockchain/lssa` repo and run indexer service:
      - `RUST_LOG=info cargo run -p indexer_service indexer/service/configs/indexer_config.json`

 3. On another terminal go to the `logos-blockchain/lssa` repo and run the sequencer:
      - `RUST_LOG=info cargo run -p sequencer_service sequencer/service/configs/debug/sequencer_config.json`
 4. (To run the explorer): on another terminal go to `logos-blockchain/lssa/explorer_service` and run the following:
      - `cargo install cargo-leptos`
      - `cargo leptos build --release`
      - `cargo leptos serve --release`

### Notes on cleanup

After stopping services above you need to remove 3 folders to start cleanly:
 1. In the `logos-blockchain/logos-blockchain` folder `state` (not needed in case of docker setup)
 2. In the `lssa` folder `sequencer/service/rocksdb`
 3. In the `lssa` file `sequencer/service/bedrock_signing_key`
 4. In the `lssa` folder `indexer/service/rocksdb`

### Normal mode (`just` commands)
We provide a `Justfile` for developer and user needs, you can run the whole setup with it. The only difference will be that logos-blockchain (bedrock) will be started from docker.

#### 1'st Terminal

```bash
just run-bedrock
```

#### 2'nd Terminal

```bash
just run-indexer
```

#### 3'rd Terminal

```bash
just run-sequencer
```

#### 4'th Terminal

```bash
just run-explorer
```

#### 5'th Terminal

You can run any command our wallet support by passing it as an argument for `just run-wallet`, for example:

```bash
just run-wallet check-health
```

This will use a wallet binary built from this repo and not the one installed in your system if you have some. Also another wallet home directory will be used. This is done to not to mess up with your local wallet and to easily clean generated files (see next section).

#### Shutdown

1. Press `ctrl-c` in every terminal
2. Run `just clean` to clean runtime data

### Standalone mode
The sequencer can be run in standalone mode with:
```bash
RUST_LOG=info cargo run --features standalone -p sequencer_service sequencer/service/configs/debug
```

## Running with Docker

You can run the whole setup with Docker:

```bash
docker compose up
```

With that you can send transactions from local wallet to the Sequencer running inside Docker using `wallet/configs/debug` as well as exploring blocks by opening `http://localhost:8080`.

## Caution for local image builds

If you're going to build sequencer image locally you should better adjust default docker settings and set `defaultKeepStorage` at least `25GB` so that it can keep layers properly cached.

