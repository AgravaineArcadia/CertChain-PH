# CertChain PH

> Tamper-proof on-chain credential registry with learn-to-earn XLM rewards for students in Southeast Asia — powered by Stellar Soroban.

---

## Problem

A fresh graduate in Cebu, Philippines spends **2–4 weeks** waiting for manual diploma authentication from their university registrar before employers will process their job application — costing them their first paycheck and sometimes the job offer entirely.

## Solution

CertChain PH lets universities issue tamper-proof certificates directly onto Stellar via a Soroban smart contract. Employers verify a graduate's credentials **instantly on-chain**, and the student **automatically receives an XLM reward** the moment verification is confirmed. Employers can also trigger direct wallet-to-wallet payments to verified students.

---

## Stellar Features Used

| Feature | Purpose |
|---|---|
| **Soroban Smart Contracts** | Core logic — registration, duplicate detection, tamper-proofing, reward & payment signals |
| **XLM Transfers** | Reward students upon verified credential issuance |
| **Custom Tokens** | Optional school-issued credential asset (e.g., `UPLB-CERT`) |
| **Trustlines** | Student wallet opts in to receive the credential asset |

---

## Suggested MVP Timeline

| Week | Milestone |
|---|---|
| 1 | Smart contract: `register_certificate`, `verify_certificate` |
| 2 | Smart contract: `reward_student`, `link_payment` + all tests passing |
| 3 | Deploy to Stellar Testnet; wire up React/Next.js frontend |
| 4 | Demo polish: registrar dashboard, student wallet UI, employer verify flow |

---

## Vision & Purpose

Millions of students across the Philippines, Vietnam, and Indonesia face weeks-long delays waiting for manual credential verification — a bottleneck that delays income and disadvantages fresh graduates from lower-income backgrounds. CertChain PH removes that bottleneck entirely: credentials are issued once, verified instantly, and tied to real financial outcomes (XLM rewards, employer payments) — creating a self-sustaining learn-to-earn loop anchored to verifiable achievement.

---

## Prerequisites

- **Rust toolchain** — `rustup target add wasm32-unknown-unknown`
- **Soroban CLI** — `cargo install --locked stellar-cli --features opt` (v22+)
- **Node.js 18+** — for the frontend (optional for contract-only work)

Verify your CLI version:
```bash
stellar --version
```

---

## Build

```bash
# From the project root
stellar contract build
```

Output WASM: `target/wasm32-unknown-unknown/release/certchain_ph.wasm`

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

### 1. Fund your identity

```bash
stellar keys generate --global alice --network testnet
stellar keys fund alice --network testnet
```

### 2. Deploy the contract

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/certchain_ph.wasm \
  --source alice \
  --network testnet
```

Save the returned `CONTRACT_ID`.

### 3. Initialise (set admin/registrar)

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
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
  --source alice \
  --network testnet \
  -- register_certificate \
  --cert_hash "1a2b3c4d5e6f7a8b9cadbecfd0e1f203142536475869" \
  --student_wallet "GCEZWKCA5VLDNRLN3RPRJMRZOX3Z6G5CHCGEWDHDAEYOP3Y8IFNV8MS"
```

### verify_certificate

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- verify_certificate \
  --cert_hash "1a2b3c4d5e6f7a8b9cadbecfd0e1f203142536475869" \
  --expected_owner "GCEZWKCA5VLDNRLN3RPRJMRZOX3Z6G5CHCGEWDHDAEYOP3Y8IFNV8MS"
```

### reward_student

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
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

## Project Structure

```
certchain_ph/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs      ← Soroban smart contract
    └── test.rs     ← 3 unit tests
```

---

## How to Contribute

1. Fork the repo
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit your changes: `git commit -m "feat: add X"`
4. Push and open a Pull Request

Reference repo for deploy guide: https://github.com/armlynobinguar/Stellar-Bootcamp-2026

---

## License

MIT © 2025 CertChain PH Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.