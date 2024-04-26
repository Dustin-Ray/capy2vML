use std::fs::File;
use std::time::Instant;

use methods::{TEST_ELF, TEST_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use std::io::{BufReader, BufRead};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let file_path = "age_vs_insurance_costs.csv";
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    // Collect lines, assuming each line is a valid CSV entry
    let all_data: Vec<(f32, f32)> = reader.lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() == 2 {
                parts[0].parse::<f32>().ok().and_then(|x| {
                    parts[1].parse::<f32>().ok().map(|y| (x, y))
                })
            } else {
                None
            }
        })
        .collect();

    let mut total_prove_time = 0.0;
    let chunk_size = 175;
    let mut receipts: Vec<Receipt> = vec![];
    for chunk in all_data.chunks(chunk_size) {
        let (x, y): (Vec<_>, Vec<_>) = chunk.iter().cloned().unzip();
        let interleaved = interleave_vectors(&x, &y);

        let env = ExecutorEnv::builder()
            .write(&interleaved)
            .unwrap()
            .build()
            .unwrap();

        let prover = default_prover();
        let start_time = Instant::now();
        let receipt = prover.prove(env, TEST_ELF).unwrap();
        receipts.push(receipt);
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f32();
        println!("Proved batch in: {}", elapsed_time);
        total_prove_time += elapsed_time;

    }
    println!("Total time for proving: {}", total_prove_time);

    let mut total_verify_time = 0.0;
    for receipt in receipts {
        // optionally access the model
        let c: (f32, f32) = receipt.journal.decode().unwrap();
        println!("{:?}", c);
        let start_time = Instant::now();
        receipt.verify(TEST_ID).unwrap();
        let end_time = Instant::now();
        let elapsed_time = end_time.duration_since(start_time).as_secs_f32();
        println!("Verified batch in: {}", elapsed_time);
        total_verify_time += elapsed_time;
    }

    println!("Total time for verification: {}", total_verify_time);

}

/// Interleaves two vectors, x,y into x,y,x,y...etc
fn interleave_vectors<T: Clone>(x: &[T], y: &[T]) -> Vec<T> {
    let mut result = Vec::with_capacity(x.len() * 2);
    for (a, b) in x.iter().zip(y) {
        result.push(a.clone());
        result.push(b.clone());
    }
    result
}