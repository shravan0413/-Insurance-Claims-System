#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short};

// Structure to store claim information
#[contracttype]
#[derive(Clone)]
pub struct Claim {
    pub claim_id: u64,
    pub policy_holder: String,
    pub claim_amount: u64,
    pub description: String,
    pub timestamp: u64,
    pub is_verified: bool,
    pub is_paid: bool,
}

// Structure to track overall system statistics
#[contracttype]
#[derive(Clone)]
pub struct ClaimStats {
    pub total_claims: u64,
    pub verified_claims: u64,
    pub paid_claims: u64,
    pub total_paid_amount: u64,
}

// Mapping claim_id to Claim
#[contracttype]
pub enum ClaimBook {
    Claim(u64)
}

// Constants for storage keys
const CLAIM_COUNT: Symbol = symbol_short!("CL_COUNT");
const CLAIM_STATS: Symbol = symbol_short!("CL_STATS");

#[contract]
pub struct InsuranceClaimContract;

#[contractimpl]
impl InsuranceClaimContract {

    // Function 1: Submit a new insurance claim
    pub fn submit_claim(env: Env, policy_holder: String, claim_amount: u64, description: String) -> u64 {
        
        // Get and increment claim counter
        let mut claim_count: u64 = env.storage().instance().get(&CLAIM_COUNT).unwrap_or(0);
        claim_count += 1;

        // Get current timestamp
        let timestamp = env.ledger().timestamp();

        // Create new claim
        let new_claim = Claim {
            claim_id: claim_count,
            policy_holder: policy_holder.clone(),
            claim_amount,
            description,
            timestamp,
            is_verified: false,
            is_paid: false,
        };

        // Update statistics
        let mut stats = Self::get_claim_stats(env.clone());
        stats.total_claims += 1;

        // Store claim and updated data
        env.storage().instance().set(&ClaimBook::Claim(claim_count), &new_claim);
        env.storage().instance().set(&CLAIM_COUNT, &claim_count);
        env.storage().instance().set(&CLAIM_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Claim submitted successfully with ID: {}", claim_count);
        
        claim_count
    }

    // Function 2: Verify a claim (Admin/Insurer action)
    pub fn verify_claim(env: Env, claim_id: u64) {
        
        // Retrieve the claim
        let mut claim = Self::view_claim(env.clone(), claim_id);

        // Ensure claim exists and is not already verified
        if claim.claim_id == 0 {
            log!(&env, "Claim not found!");
            panic!("Claim does not exist!");
        }

        if claim.is_verified {
            log!(&env, "Claim already verified!");
            panic!("Claim is already verified!");
        }

        // Mark claim as verified
        claim.is_verified = true;

        // Update statistics
        let mut stats = Self::get_claim_stats(env.clone());
        stats.verified_claims += 1;

        // Store updated claim and stats
        env.storage().instance().set(&ClaimBook::Claim(claim_id), &claim);
        env.storage().instance().set(&CLAIM_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Claim ID: {} verified successfully!", claim_id);
    }

    // Function 3: Process payment for verified claim
    pub fn process_payment(env: Env, claim_id: u64) {
        
        // Retrieve the claim
        let mut claim = Self::view_claim(env.clone(), claim_id);

        // Validation checks
        if claim.claim_id == 0 {
            log!(&env, "Claim not found!");
            panic!("Claim does not exist!");
        }

        if !claim.is_verified {
            log!(&env, "Claim must be verified before payment!");
            panic!("Claim is not verified!");
        }

        if claim.is_paid {
            log!(&env, "Claim already paid!");
            panic!("Payment already processed!");
        }

        // Mark claim as paid
        claim.is_paid = true;

        // Update statistics
        let mut stats = Self::get_claim_stats(env.clone());
        stats.paid_claims += 1;
        stats.total_paid_amount += claim.claim_amount;

        // Store updated claim and stats
        env.storage().instance().set(&ClaimBook::Claim(claim_id), &claim);
        env.storage().instance().set(&CLAIM_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "Payment of {} processed for Claim ID: {}", claim.claim_amount, claim_id);
    }

    // Function 4: View claim details by ID
    pub fn view_claim(env: Env, claim_id: u64) -> Claim {
        
        let key = ClaimBook::Claim(claim_id);
        
        env.storage().instance().get(&key).unwrap_or(Claim {
            claim_id: 0,
            policy_holder: String::from_str(&env, "Not_Found"),
            claim_amount: 0,
            description: String::from_str(&env, "Not_Found"),
            timestamp: 0,
            is_verified: false,
            is_paid: false,
        })
    }

    // Helper function: Get overall claim statistics
    pub fn get_claim_stats(env: Env) -> ClaimStats {
        
        env.storage().instance().get(&CLAIM_STATS).unwrap_or(ClaimStats {
            total_claims: 0,
            verified_claims: 0,
            paid_claims: 0,
            total_paid_amount: 0,
        })
    }
}