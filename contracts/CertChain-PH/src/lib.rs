#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, contracterror,
    Address, BytesN, Env,
};

// ─── Storage Keys ─────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Certificate(BytesN<32>),
    Admin,
}

// ─── Data Types ───────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct CertRecord {
    pub owner: Address,
    pub issued_at: u64,
    pub reward_paid: bool,
}

// ─── Error Codes ──────────────────────────────────────────────────────────────
// #[contracterror] is REQUIRED so Soroban can convert this enum to/from
// soroban_sdk::Error. Without it, #[contractimpl] cannot compile functions
// that return Result<_, Error>.

#[contracterror]
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    AlreadyRegistered = 1,
    NotFound          = 2,
    Unauthorized      = 3,
    AlreadyRewarded   = 4,
}

// ─── Events ───────────────────────────────────────────────────────────────────
// Soroban SDK v22 deprecates env.events().publish() in favour of the
// #[contractevent] macro. Each struct maps to one on-chain event.

/// Emitted when a certificate is successfully registered.
#[contractevent]
pub struct CertRegistered {
    pub cert_hash: BytesN<32>,
    pub student: Address,
}

/// Emitted when a certificate is verified (valid or not).
#[contractevent]
pub struct CertVerified {
    pub cert_hash: BytesN<32>,
    pub expected_owner: Address,
    pub valid: bool,
}

/// Emitted when the XLM reward signal is sent for a student.
#[contractevent]
pub struct CertRewarded {
    pub cert_hash: BytesN<32>,
    pub student: Address,
}

/// Emitted when an employer signals a payment intent.
#[contractevent]
pub struct CertPayment {
    pub cert_hash: BytesN<32>,
    pub employer: Address,
    pub student: Address,
    pub amount_stroops: i128,
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct CertChainPH;

#[contractimpl]
impl CertChainPH {
    // ── initialize ──────────────────────────────────────────────────────────

    /// Set the admin (university registrar) once on deployment.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialised");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    // ── register_certificate ────────────────────────────────────────────────

    /// Registers a certificate hash + student wallet on-chain.
    /// Rejects duplicates (tamper / re-issue detection).
    /// Emits CertRegistered on success.
    pub fn register_certificate(
        env: Env,
        cert_hash: BytesN<32>,
        student_wallet: Address,
    ) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let key = DataKey::Certificate(cert_hash.clone());

        if env.storage().persistent().has(&key) {
            return Err(Error::AlreadyRegistered);
        }

        let record = CertRecord {
            owner: student_wallet.clone(),
            issued_at: env.ledger().timestamp(),
            reward_paid: false,
        };
        env.storage().persistent().set(&key, &record);

        // Emit event using the modern #[contractevent] pattern
        CertRegistered {
            cert_hash,
            student: student_wallet,
        }
        .emit(&env);

        Ok(())
    }

    // ── verify_certificate ──────────────────────────────────────────────────

    /// Verifies a certificate. Returns true if hash is registered and the
    /// stored owner matches expected_owner. Emits CertVerified regardless.
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

        CertVerified {
            cert_hash,
            expected_owner,
            valid,
        }
        .emit(&env);

        valid
    }

    // ── reward_student ───────────────────────────────────────────────────────

    /// Marks the reward as paid and emits CertRewarded so the off-chain
    /// relayer / dApp frontend can execute the actual XLM transfer.
    pub fn reward_student(env: Env, cert_hash: BytesN<32>) -> Result<(), Error> {
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

    /// Employer-triggered payment intent. Confirms certificate exists, then
    /// emits CertPayment for the dApp to execute the XLM/USDC transfer.
    pub fn link_payment(
        env: Env,
        cert_hash: BytesN<32>,
        employer: Address,
        amount_stroops: i128,
    ) -> Result<(), Error> {
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

    /// Read-only: returns the stored CertRecord for a given hash, or None.
    pub fn get_certificate(env: Env, cert_hash: BytesN<32>) -> Option<CertRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::Certificate(cert_hash))
    }
}

mod test;