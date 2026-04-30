#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, contracterror,
    Address, BytesN, Env,
};

// ─── Storage Keys ─────────────────────────────────────────────────────────────

/// Keys used to read/write contract state in persistent storage.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Maps a certificate hash (BytesN<32>) → CertRecord
    Certificate(BytesN<32>),
    /// Stores the single admin address (university registrar)
    Admin,
}

// ─── Data Types ───────────────────────────────────────────────────────────────

/// On-chain record for a single issued certificate.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CertRecord {
    /// Stellar wallet of the certificate owner (the student)
    pub owner: Address,
    /// Ledger timestamp at issuance — used for tamper detection
    pub issued_at: u64,
    /// Whether the XLM reward has already been disbursed
    pub reward_paid: bool,
}

// ─── Error Codes ──────────────────────────────────────────────────────────────
// #[contracterror] is required so Soroban can convert this enum to/from
// soroban_sdk::Error. Without it, functions returning Result<_, Error>
// will not compile under #[contractimpl].

#[contracterror]
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// Certificate hash already on-chain — duplicate / tamper attempt
    AlreadyRegistered = 1,
    /// Certificate hash not found in storage
    NotFound          = 2,
    /// Caller is not the authorised admin
    Unauthorized      = 3,
    /// XLM reward was already paid for this certificate
    AlreadyRewarded   = 4,
}

// ─── Events ───────────────────────────────────────────────────────────────────
// SDK v22 deprecates env.events().publish(). Use #[contractevent] structs
// with .emit(&env) instead — each struct is one strongly-typed on-chain event.

/// Emitted when a certificate is successfully registered on-chain.
#[contractevent]
pub struct CertRegistered {
    pub cert_hash: BytesN<32>,
    pub student:   Address,
}

/// Emitted when a certificate is verified (valid or invalid).
#[contractevent]
pub struct CertVerified {
    pub cert_hash:      BytesN<32>,
    pub expected_owner: Address,
    pub valid:          bool,
}

/// Emitted when the XLM reward signal fires for a student.
/// The dApp relayer listens for this to execute the actual transfer.
#[contractevent]
pub struct CertRewarded {
    pub cert_hash: BytesN<32>,
    pub student:   Address,
}

/// Emitted when an employer signals a payment intent to a verified student.
#[contractevent]
pub struct CertPayment {
    pub cert_hash:      BytesN<32>,
    pub employer:       Address,
    pub student:        Address,
    pub amount_stroops: i128,
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct StellaroidEarn;

#[contractimpl]
impl StellaroidEarn {

    // ── initialize ──────────────────────────────────────────────────────────

    /// One-time setup: store the admin (registrar) address.
    /// Must be called immediately after deployment.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialised");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    // ── register_certificate ────────────────────────────────────────────────

    /// Registers a certificate hash + student wallet on-chain.
    ///
    /// - Requires admin (registrar) auth.
    /// - Rejects duplicate hashes — prevents re-issuance and tamper attempts.
    /// - Stores a CertRecord keyed by cert_hash.
    /// - Emits CertRegistered so off-chain indexers can react instantly.
    pub fn register_certificate(
        env: Env,
        cert_hash:      BytesN<32>,
        student_wallet: Address,
    ) -> Result<(), Error> {
        // Only the admin (university registrar) may issue certificates
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let key = DataKey::Certificate(cert_hash.clone());

        // Duplicate / tamper detection: reject if the hash already exists
        if env.storage().persistent().has(&key) {
            return Err(Error::AlreadyRegistered);
        }

        // Persist the certificate record
        let record = CertRecord {
            owner:       student_wallet.clone(),
            issued_at:   env.ledger().timestamp(),
            reward_paid: false,
        };
        env.storage().persistent().set(&key, &record);

        // Emit on-chain event for dApp / indexer
        CertRegistered {
            cert_hash,
            student: student_wallet,
        }
        .emit(&env);

        Ok(())
    }

    // ── verify_certificate ──────────────────────────────────────────────────

    /// Returns true if cert_hash is registered AND the stored owner matches
    /// expected_owner. Returns false otherwise (no panic).
    ///
    /// Emits CertVerified in all cases — employers and DAOs use this event
    /// as an auditable proof of verification.
    pub fn verify_certificate(
        env:            Env,
        cert_hash:      BytesN<32>,
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

        // Always emit — verification attempts are permanently auditable
        CertVerified {
            cert_hash,
            expected_owner,
            valid,
        }
        .emit(&env);

        valid
    }

    // ── reward_student ───────────────────────────────────────────────────────

    /// Marks the reward as paid and emits CertRewarded.
    ///
    /// The contract signals intent; the dApp relayer executes the actual XLM
    /// transfer from the admin wallet to the student wallet on Stellar.
    /// Follows checks-effects-interactions: storage is updated before emit.
    pub fn reward_student(
        env:       Env,
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

        // Prevent double-payment
        if record.reward_paid {
            return Err(Error::AlreadyRewarded);
        }

        // Update state BEFORE emitting (checks-effects-interactions pattern)
        record.reward_paid = true;
        env.storage().persistent().set(&key, &record);

        CertRewarded {
            cert_hash,
            student: record.owner,
        }
        .emit(&env);

        Ok(())
    }

    // ── link_payment ─────────────────────────────────────────────────────────

    /// Employer-triggered payment intent.
    ///
    /// Confirms the certificate exists, then emits CertPayment so the dApp
    /// can execute the XLM / USDC transfer from the employer's wallet to the
    /// verified student wallet. amount_stroops: 1 XLM = 10_000_000 stroops.
    pub fn link_payment(
        env:            Env,
        cert_hash:      BytesN<32>,
        employer:       Address,
        amount_stroops: i128,
    ) -> Result<(), Error> {
        // Employer must sign this transaction
        employer.require_auth();

        let key = DataKey::Certificate(cert_hash.clone());

        let record: CertRecord = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotFound)?;

        CertPayment {
            cert_hash,
            employer,
            student: record.owner,
            amount_stroops,
        }
        .emit(&env);

        Ok(())
    }

    // ── get_certificate ──────────────────────────────────────────────────────

    /// Read-only helper — returns the stored CertRecord or None.
    /// Used by the frontend to display certificate status.
    pub fn get_certificate(
        env:       Env,
        cert_hash: BytesN<32>,
    ) -> Option<CertRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::Certificate(cert_hash))
    }
}

mod test;