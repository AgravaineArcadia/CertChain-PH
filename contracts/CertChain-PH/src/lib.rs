#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, contracttype,
    symbol_short,
    Address, BytesN, Env, Symbol,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

/// Key to store a certificate record, keyed by its hash
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Certificate(BytesN<32>), // cert_hash → CertRecord
    Admin,                   // single admin address (the university registry)
}

// ─── Data Types ───────────────────────────────────────────────────────────────

/// On-chain certificate record
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CertRecord {
    /// Stellar wallet of the certificate owner (the student)
    pub owner: Address,
    /// Timestamp of issuance (ledger timestamp)
    pub issued_at: u64,
    /// Whether the XLM reward has already been paid out
    pub reward_paid: bool,
}

// ─── Error Codes ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// Certificate hash already exists — duplicate detected
    AlreadyRegistered = 1,
    /// Certificate hash not found — cannot verify or pay
    NotFound = 2,
    /// Caller is not the authorised admin
    Unauthorized = 3,
    /// Reward was already disbursed for this certificate
    AlreadyRewarded = 4,
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct CertChainPH;

#[contractimpl]
impl CertChainPH {
    // ── Initialise ──────────────────────────────────────────────────────────

    /// Set the admin (university registrar) once on deployment.
    /// All privileged operations require the admin's signature.
    pub fn initialize(env: Env, admin: Address) {
        // Prevent re-initialisation
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialised");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    // ── register_certificate ────────────────────────────────────────────────

    /// Called by the university registrar to issue a certificate on-chain.
    ///
    /// Steps:
    ///   1. Require admin auth.
    ///   2. Reject duplicates (tamper / re-issue detection).
    ///   3. Store CertRecord keyed by cert_hash.
    ///   4. Emit a `cert_reg` event so off-chain indexers can react.
    pub fn register_certificate(
        env: Env,
        cert_hash: BytesN<32>,
        student_wallet: Address,
    ) -> Result<(), Error> {
        // Only the admin (registrar) may issue certificates
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let key = DataKey::Certificate(cert_hash.clone());

        // Duplicate / tamper detection: reject if already stored
        if env.storage().persistent().has(&key) {
            return Err(Error::AlreadyRegistered);
        }

        // Build and persist the record
        let record = CertRecord {
            owner: student_wallet.clone(),
            issued_at: env.ledger().timestamp(),
            reward_paid: false,
        };
        env.storage().persistent().set(&key, &record);

        // Emit event: (topic1, topic2) → data
        env.events().publish(
            (symbol_short!("cert_reg"), student_wallet),
            cert_hash,
        );

        Ok(())
    }

    // ── verify_certificate ──────────────────────────────────────────────────

    /// Anyone (employer, DAO, dApp) can call this to verify a certificate.
    ///
    /// Returns `true` if the hash is registered and the stored owner matches
    /// `expected_owner`; `false` otherwise.
    /// Emits a `cert_ver` event so the verification is auditable on-chain.
    pub fn verify_certificate(
        env: Env,
        cert_hash: BytesN<32>,
        expected_owner: Address,
    ) -> bool {
        let key = DataKey::Certificate(cert_hash.clone());

        let valid = if let Some(record) = env
            .storage()
            .persistent()
            .get::<DataKey, CertRecord>(&key)
        {
            record.owner == expected_owner
        } else {
            false
        };

        // Emit audit event regardless of outcome
        env.events().publish(
            (symbol_short!("cert_ver"), expected_owner),
            (cert_hash, valid),
        );

        valid
    }

    // ── reward_student ───────────────────────────────────────────────────────

    /// Called by the admin after a certificate is verified to transfer
    /// an XLM reward to the student.
    ///
    /// In the Soroban model the contract itself does not hold native XLM;
    /// instead the admin account authorises a transfer via `token::transfer`.
    /// Here we mark the reward as paid and emit the event so the off-chain
    /// relayer (or the dApp frontend) can trigger the actual XLM transfer
    /// through the Stellar native asset contract.
    ///
    /// To keep the MVP compile-ready without a deployed token contract address
    /// on every test environment, the XLM disbursement event (`cert_rwd`) is
    /// what the frontend listens to in order to execute the payment.
    pub fn reward_student(
        env: Env,
        cert_hash: BytesN<32>,
    ) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let key = DataKey::Certificate(cert_hash.clone());

        let mut record: CertRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotFound)?;

        if record.reward_paid {
            return Err(Error::AlreadyRewarded);
        }

        // Mark reward as paid before emitting event (checks-effects-interactions)
        record.reward_paid = true;
        env.storage().persistent().set(&key, &record);

        // Emit reward event — the dApp relayer executes the XLM transfer
        env.events().publish(
            (symbol_short!("cert_rwd"), record.owner),
            cert_hash,
        );

        Ok(())
    }

    // ── link_payment ─────────────────────────────────────────────────────────

    /// Employer-triggered payment signal.
    ///
    /// An employer calls this to signal they wish to pay a verified student.
    /// The contract confirms the certificate is valid for the given owner,
    /// then emits a `cert_pay` event that the dApp frontend uses to execute
    /// the actual XLM / USDC transfer from the employer's wallet.
    ///
    /// `amount_stroops` — payment amount expressed in stroops (1 XLM = 10_000_000)
    pub fn link_payment(
        env: Env,
        cert_hash: BytesN<32>,
        employer: Address,
        amount_stroops: i128,
    ) -> Result<(), Error> {
        // Employer must authorise this call
        employer.require_auth();

        let key = DataKey::Certificate(cert_hash.clone());

        let record: CertRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotFound)?;

        // Emit payment intent event — frontend / relayer executes the transfer
        env.events().publish(
            (symbol_short!("cert_pay"), employer),
            (cert_hash, record.owner, amount_stroops),
        );

        Ok(())
    }

    // ── get_certificate ──────────────────────────────────────────────────────

    /// Read-only helper: returns the stored CertRecord for a given hash.
    /// Returns None if not found.
    pub fn get_certificate(
        env: Env,
        cert_hash: BytesN<32>,
    ) -> Option<CertRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::Certificate(cert_hash))
    }
}

mod test;