# Stellaroid Earn

> Tamper-proof on-chain credential registry with learn-to-earn XLM rewards for students in Southeast Asia — powered by Stellar Soroban.

---

## Problem

A graduating student in the Philippines cannot easily prove their credentials to employers or access financial opportunities, forcing them to rely on manual verification that delays hiring and limits income.

## Solution

Using Stellar, Stellaroid Earn builds a transparent on-chain system where each certificate has a unique, traceable identity anchored to its rightful owner. Students unlock XLM-based rewards, job payouts, and financial access upon instant credential verification.

---

## Suggested MVP Timeline

| Week | Milestone |
|---|---|
| 1 | Smart contract: `register_certificate`, `verify_certificate` |
| 2 | Smart contract: `reward_student`, `link_payment` + all tests passing |
| 3 | Deploy to Stellar Testnet; wire up React / Next.js frontend |
| 4 | Demo polish: registrar dashboard, student wallet UI, employer verify flow |

---

## Stellar Features Used

| Feature | Purpose |
|---|---|
| **Soroban Smart Contracts** | Core credential registry, duplicate detection, tamper-proofing, reward & payment signals |
| **XLM Transfers** | Reward students instantly upon verified credential issuance |
| **Custom Tokens** | Optional school-issued credential asset (e.g., `UPLB-CERT`) |
| **Trustlines** | Student wallet opts in to receive the school credential asset |

---

## Vision & Purpose

Millions of students across the Philippines, Vietnam, and Indonesia face weeks-long delays from manual credential verification — a bottleneck that delays income and disadvantages fresh graduates from lower-income backgrounds. Stellaroid Earn removes that bottleneck entirely: credentials are issued once, verified instantly, and tied to real financial outcomes — creating a self-sustaining learn-to-earn loop anchored to verifiable achievement.

---

## Prerequisites

- **Rust toolchain** with Wasm target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- **Stellar CLI v22+**:
  ```bash
  cargo install --locked stellar-cli --features opt
  stellar --version
  ```

---

## Build

```bash
stellar contract build
```

Output: `target/wasm32-unknown-unknown/release/stellaroid_earn.wasm`

---

## Test

```bash
cargo test
```

Runs all 3 tests:
- `test_happy_path_register_and_reward`
- `test_duplicate_registration_rejected`
- `test_state_reflects_correct_owner_after_registration`

---

## Deploy to Testnet

### 1. Clone the repo

```bash
git clone https://github.com/armlynobinguar/Stellar-Bootcamp-2026
cd stellaroid_earn
```

### 2. Fund your identity

```bash
stellar keys generate --global registrar --network testnet
stellar keys fund registrar --network testnet
```

### 3. Deploy

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellaroid_earn.wasm \
  --source registrar \
  --network testnet
```

Save the returned `CONTRACT_ID`.

### 4. Initialise (set admin/registrar)

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source registrar \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS>
```

---

## Sample CLI Invocations

### register_certificate

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source registrar \
  --network testnet \
  -- register_certificate \
  --cert_hash "1a2b3c4d5e6f7a8b9cadbecfd0e1f203142536475869" \
  --student_wallet "GCEZWKCA5VLDNRLN3RPRJMRZOX3Z6G5CHCGEWDHDAEYOP3Y8IFNV8MS"
```

### verify_certificate

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source registrar \
  --network testnet \
  -- verify_certificate \
  --cert_hash "1a2b3c4d5e6f7a8b9cadbecfd0e1f203142536475869" \
  --expected_owner "GCEZWKCA5VLDNRLN3RPRJMRZOX3Z6G5CHCGEWDHDAEYOP3Y8IFNV8MS"
```

### reward_student

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source registrar \
  --network testnet \
  -- reward_student \
  --cert_hash "1a2b3c4d5e6f7a8b9cadbecfd0e1f203142536475869"
```

### link_payment (employer-triggered)

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source employer_key \
  --network testnet \
  -- link_payment \
  --cert_hash "1a2b3c4d5e6f7a8b9cadbecfd0e1f203142536475869" \
  --employer "GBemployer..." \
  --amount_stroops 50000000
```

> 50 000 000 stroops = 5 XLM

---

## project description

## future scope 

## Project Structure

```
stellaroid_earn/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs      ← Soroban smart contract
    └── test.rs     ← 3 unit tests
```

---
 
## Deployed Contract Details
[1] https://stellar.expert/explorer/testnet/tx/996dbeeb285df8757c7c11d817e8935138be2daa6f1c05a9f0263b5aff466d34
[2] https://lab.stellar.org/r/testnet/contract/CB7GMFBOEW3CYATQOPGEZSJQ5FKW5PDGISTIJYNYVB74IHBIELWFIWAS
 
![screenshot](path/to/image.png) 

## License

MIT © 2025 Stellaroid Earn Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.