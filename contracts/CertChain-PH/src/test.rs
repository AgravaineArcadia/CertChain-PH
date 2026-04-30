#[cfg(test)]
mod tests {
    use soroban_sdk::{
        testutils::{Address as _, Events},
        Address, BytesN, Env,
    };

    use crate::{CertChainPH, CertChainPHClient, Error};

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Spin up a fresh environment and deploy the contract, returning
    /// (env, client, admin_address, a sample cert_hash, a sample student_wallet).
    fn setup() -> (Env, CertChainPHClient<'static>, Address, BytesN<32>, Address) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(CertChainPH, ());
        let client = CertChainPHClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let student = Address::generate(&env);

        // A deterministic 32-byte certificate hash (SHA-256 of "CERT-2025-UP-001")
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

    // ── Test 1: Happy Path ───────────────────────────────────────────────────
    //
    // A certificate is successfully registered and the student receives an XLM
    // reward signal (cert_rwd event emitted after reward_student is called).

    #[test]
    fn test_happy_path_register_and_reward() {
        let (env, client, _admin, cert_hash, student) = setup();

        // 1. Register the certificate
        let result = client.try_register_certificate(&cert_hash, &student);
        assert!(result.is_ok(), "registration should succeed");

        // 2. Verify the certificate is on-chain and owned by the student
        let is_valid = client.verify_certificate(&cert_hash, &student);
        assert!(is_valid, "certificate should be valid for the registered student");

        // 3. Trigger XLM reward — should succeed (first time)
        let reward_result = client.try_reward_student(&cert_hash);
        assert!(reward_result.is_ok(), "reward should be paid successfully");

        // 4. Confirm reward_paid flag is now true in storage
        let record = client.get_certificate(&cert_hash).unwrap();
        assert!(record.reward_paid, "reward_paid flag should be true after reward");

        // 5. At least one event should have been emitted during the flow
        assert!(
            !env.events().all().is_empty(),
            "at least one event should be emitted"
        );
    }

    // ── Test 2: Edge Case ────────────────────────────────────────────────────
    //
    // A duplicate certificate registration is rejected with AlreadyRegistered.

    #[test]
    fn test_duplicate_registration_rejected() {
        let (_env, client, _admin, cert_hash, student) = setup();

        // First registration — must succeed
        client.register_certificate(&cert_hash, &student);

        // Second registration with the same hash — must fail
        let duplicate_result = client.try_register_certificate(&cert_hash, &student);

        match duplicate_result {
            Err(Ok(Error::AlreadyRegistered)) => {
                // ✅ Expected error returned
            }
            other => panic!(
                "expected AlreadyRegistered error, got: {:?}",
                other
            ),
        }
    }

    // ── Test 3: State Verification ───────────────────────────────────────────
    //
    // Contract storage correctly reflects the certificate owner and hash after
    // a successful registration.

    #[test]
    fn test_state_reflects_correct_owner_after_registration() {
        let (_env, client, _admin, cert_hash, student) = setup();

        // Register the certificate
        client.register_certificate(&cert_hash, &student);

        // Fetch the stored record
        let record = client
            .get_certificate(&cert_hash)
            .expect("record should exist after registration");

        // Owner must match the student wallet passed in
        assert_eq!(
            record.owner, student,
            "stored owner must match the registered student wallet"
        );

        // Reward must not have been paid yet at registration time
        assert!(
            !record.reward_paid,
            "reward_paid should be false immediately after registration"
        );

        // issued_at must be a non-zero ledger timestamp
        assert!(
            record.issued_at > 0,
            "issued_at should be a positive ledger timestamp"
        );

        // Verification with the correct owner should return true
        let valid = client.verify_certificate(&cert_hash, &student);
        assert!(valid, "verify_certificate should return true for the correct owner");

        // Verification with a different address should return false
        let impostor = soroban_sdk::testutils::Address::generate(&_env);
        let invalid = client.verify_certificate(&cert_hash, &impostor);
        assert!(!invalid, "verify_certificate should return false for an impostor");
    }
}