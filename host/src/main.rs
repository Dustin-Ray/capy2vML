use std::time::Instant;

use methods::{TEST_ELF, TEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let mut total_prove_time = 0.0;
    let mut receipts: Vec<Receipt> = vec![];
    let env = ExecutorEnv::builder().build().unwrap();
    let prover = default_prover();
    let start_time = Instant::now();
    let receipt = prover.prove(env, TEST_ELF).unwrap();
    receipts.push(receipt.clone());
    let end_time = Instant::now();
    let elapsed_time = end_time.duration_since(start_time).as_secs_f32();
    println!("Proved batch in: {}", elapsed_time);
    total_prove_time += elapsed_time;
    println!("Total time for proving: {}", total_prove_time);

    let mut total_verify_time = 0.0;
    for _ in 0..100 {
        // optionally access the model
        let start_time = Instant::now();
        receipt.verify(TEST_ID).unwrap();
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f32();
        total_verify_time += elapsed_time;
    
    }
    total_verify_time /= 100f32;
    println!("Average time for verification: {:?}", total_verify_time);
}
