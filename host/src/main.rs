use csv::ReaderBuilder;
use methods::{METHOD_NAME_ELF, METHOD_NAME_ID};
use risc0_zkvm::serde::from_slice;
use risc0_zkvm::{default_prover, ExecutorEnv};
use std::error::Error;
use std::fs::File;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "age_vs_insurance_costs.csv";
    let (x, y) = read_csv_file(file_path)?;
    let interleaved = interleave_vectors(&x, &y);

    let env = ExecutorEnv::builder()
        .add_input(&interleaved)
        .build()
        .unwrap();

    let prover = default_prover();

    let start_time = Instant::now();

    let receipt = prover.prove_elf(env, METHOD_NAME_ELF).unwrap();
    let end_time = Instant::now();


    let elapsed_time = end_time.duration_since(start_time).as_secs_f32();
    println!("Elapsed proving time: {} seconds", elapsed_time);

    let start_time = Instant::now();
    receipt.verify(METHOD_NAME_ID).expect(
        "Check Image ID?",
    );
    let end_time = Instant::now();
    let c: (f32, f32, usize) = from_slice(&receipt.journal).expect(
        "Journal output should deserialize into the same types (& order) that it was written",
    );

    let elapsed_time = end_time.duration_since(start_time).as_secs_f32();
    println!("Elapsed verification time: {} seconds", elapsed_time);
    println!("slope: {:.4}", c.0);
    println!("intercept: {:.4}", c.1);
    println!("cycles: {:?}", c.2);
    Ok(())

}


/// Opens a csv file with two columns and returns a tuple of Vec<f32>
fn read_csv_file(file_path: &str) -> Result<(Vec<f32>, Vec<f32>), Box<dyn Error>> {
    let mut x_values = Vec::new();
    let mut y_values = Vec::new();

    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);

    for result in rdr.records() {
        let record = result?;
        if let Some(x_str) = record.get(0) {
            if let Some(y_str) = record.get(1) {
                let x: f32 = x_str.parse()?;
                let y: f32 = y_str.parse()?;
                x_values.push(x);
                y_values.push(y);
            }
        }
    }
    println!("X values length: {:?}", x_values.len());
    println!("Y values length: {:?}", y_values.len());
    Ok((x_values, y_values))
}

/// Interleaves two vectors, x,y into x,y,x,y...etc
fn interleave_vectors<T: Clone>(x: &[T], y: &[T]) -> Vec<T> {
    let mut result = Vec::with_capacity(x.len() + y.len());
    let mut iter_x = x.iter();
    let mut iter_y = y.iter();

    loop {
        match (iter_x.next(), iter_y.next()) {
            (Some(a), Some(b)) => {
                result.push(a.clone());
                result.push(b.clone());
            }
            (None, None) => break,
            (Some(a), None) => {
                result.push(a.clone());
                while let Some(a) = iter_x.next() {
                    result.push(a.clone());
                }
                break;
            }
            (None, Some(b)) => {
                result.push(b.clone());
                while let Some(b) = iter_y.next() {
                    result.push(b.clone());
                }
                break;
            }
        }
    }
    println!("interleaved values length: {:?}", result.len());
    result
}
