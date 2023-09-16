// TODO: Update the name of the method loaded by the prover. E.g., if the method
// is `multiply`, replace `METHOD_NAME_ELF` with `MULTIPLY_ELF` and replace
// `METHOD_NAME_ID` with `MULTIPLY_ID`
use csv::ReaderBuilder;
use methods::{METHOD_NAME_ELF, METHOD_NAME_ID};
use risc0_zkvm::{default_prover, serde::from_slice, ExecutorEnv};
use std::error::Error;
use std::fs::File;
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "age_vs_insurance_costs.csv"; // Replace with the path to your CSV file
    let (x, y) = read_csv_file(file_path)?;

    // TODO: add guest input to the executor environment using
    // ExecutorEnvBuilder::add_input().
    // To access this method, you'll need to use the alternate construction
    // ExecutorEnv::builder(), which creates an ExecutorEnvBuilder. When you're
    // done adding input, call ExecutorEnvBuilder::build().

    let env = ExecutorEnv::builder()
        // Send a & b to the guest
        .add_input(&x)
        .add_input(&y)
        .build()
        .unwrap();

    // For example:
    // let env = ExecutorEnv::builder().add_input(&vec).build().unwrap();

    // Obtain the default prover.
    let prover = default_prover();

    // Start the timer
    let start_time = Instant::now();
    // Produce a receipt by proving the specified ELF binary.
    let receipt = prover.prove_elf(env, METHOD_NAME_ELF).unwrap();
    let end_time = Instant::now();

    // Calculate the elapsed time in seconds
    let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
    println!("Elapsed proving time: {} seconds", elapsed_time);

    // TODO: Implement code for transmitting or serializing the receipt for
    // other parties to verify here

    let start_time = Instant::now();
    // Optional: Verify receipt to confirm that recipients will also be able to
    // verify your receipt
    receipt.verify(METHOD_NAME_ID).unwrap();
    let end_time = Instant::now();

    // Calculate the elapsed time in seconds
    let elapsed_time = end_time.duration_since(start_time).as_secs_f64();
    println!("Elapsed verification time: {} seconds", elapsed_time);

    let _c: Vec<f32> = from_slice(&receipt.journal).expect(
        "Journal output should deserialize into the same types (& order) that it was written",
    );

    Ok(())
}

fn read_csv_file(file_path: &str) -> Result<(Vec<f64>, Vec<f64>), Box<dyn Error>> {
    let mut x_values = Vec::new();
    let mut y_values = Vec::new();

    let file = File::open(file_path)?;
    let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);

    for result in rdr.records() {
        let record = result?;
        if let Some(x_str) = record.get(0) {
            if let Some(y_str) = record.get(1) {
                let x: f64 = x_str.parse()?;
                let y: f64 = y_str.parse()?;
                x_values.push(x);
                y_values.push(y);
            }
        }
    }
    println!("X values len: {:?}", x_values.len());
    println!("Y values len: {:?}", y_values.len());
    Ok((x_values, y_values))
}
