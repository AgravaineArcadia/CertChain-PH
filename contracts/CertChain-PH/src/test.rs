#[cfg(test)]
mod tests {
    use soroban_sdk::{
        testutils::Address as _,
        Address, BytesN, Env,
    };

    use crate::{StellaroidEarn, StellaroidEarnClient, Error};

    // ── Shared Setup ─────────────────────────────────────────────────────────

    /// Deploy the contract and return commonly used values.
    /// Returns (env, client, admin, cert_hash, student).
    fn setup() -> (Env, StellaroidEarnClient<'static>, Address, BytesN<32>, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(StellaroidEarn, ());
        let client     = StellaroidEarnClient::new(&env, &contract_id);

        let admin   = Address::generate(&env);
        let student = Address::generate(&env);

        // Deterministic 32-byte hash representing "CERT-2025-PH-UPLB-001"
        let cert_hash: BytesN<32> = BytesN::from_array(
            &env,
            &[
                0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x7a, 0x8b,
                0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, 0x03,
                0x14, 0x25, 0x36, 0x47, 0x58, 0x69, 0x7a, 0x8b,
                0x9c, 0xad, 0xbe, 0xcf, 0xd0, 0xe1, 0xf2, 0x03,
            ],
        );

        client.initialize(&admin);

        (env, client, admin, cert_hash, student)
    }

    // ── Test 1: Happy Path ────────────────────────────────────────────────────
    //
    // A certificate is successfully registered and the student receives an XLM
    // reward signal (reward_paid flag set + CertRewarded event emitted).

    #[test]
    fn test_happy_path_register_and_reward() {
        let (_env, client, _admin, cert_hash, student) = setup();

        // Step 1 — Register the certificate
        let reg = client.try_register_certificate(&cert_hash, &student);
        assert!(reg.is_ok(), "registration should succeed on first call");

        // Step 2 — Verify it is on-chain and correctly owned
        let is_valid = client.verify_certificate(&cert_hash, &student);
        assert!(is_valid, "certificate should be valid for the registered student");

        // Step 3 — Trigger XLM reward (first time, must succeed)
        let reward = client.try_reward_student(&cert_hash);
        assert!(reward.is_ok(), "reward should succeed on first call");

        // Step 4 — reward_paid flag must now be true in storage
        let record = client.get_certificate(&cert_hash).expect("record must exist");
        assert!(record.reward_paid, "reward_paid must be true after reward_student");
    }

    // ── Test 2: Edge Case ─────────────────────────────────────────────────────
    //
    // A duplicate certificate registration is rejected with AlreadyRegistered.

    #[test]
    fn test_duplicate_registration_rejected() {
        let (_env, client, _admin, cert_hash, student) = setup();

        // First registration must succeed
        client.register_certificate(&cert_hash, &student);

        // Second registration with the identical hash must fail
        let duplicate = client.try_register_certificate(&cert_hash, &student);

        match duplicate {
            Err(Ok(Error::AlreadyRegistered)) => {
                // ✅ Correct — duplicate correctly rejected
            }
            other => panic!(
                "expected AlreadyRegistered, got: {:?}",
                other
            ),
        }
    }

    // ── Test 3: State Verification ────────────────────────────────────────────
    //
    // Contract storage correctly reflects the certificate owner and hash after
    // a successful registration, and verify_certificate returns false for any
    // address that is not the registered owner (tamper detection).

    #[test]
    fn test_state_reflects_correct_owner_after_registration() {
        let (env, client, _admin, cert_hash, student) = setup();

        // Register
        client.register_certificate(&cert_hash, &student);

        // Fetch the stored record
        let record = client
            .get_certificate(&cert_hash)
            .expect("record must exist after registration");

        // Owner must match exactly
        assert_eq!(record.owner, student, "stored owner must match student wallet");

        // Reward must not be paid yet at registration time
        assert!(!record.reward_paid, "reward_paid must be false after registration");

        // Ledger timestamp must be set (non-zero)
        assert!(record.issued_at > 0, "issued_at must be a positive ledger timestamp");

        // verify_certificate must return true for the correct owner
        assert!(
            client.verify_certificate(&cert_hash, &student),
            "verify must return true for the correct owner"
        );

        // verify_certificate must return false for any other address (tamper check)
        let impostor = Address::generate(&env);
        assert!(
            !client.verify_certificate(&cert_hash, &impostor),
            "verify must return false for an impostor address"
        );
    }
}