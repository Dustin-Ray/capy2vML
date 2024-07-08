#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
#![no_std]  // std support is experimental


use core::mem;
use include_bytes_aligned::include_bytes_aligned;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);
const ARRAY_SIZE: usize = 140 * 10;
const SCALING_FACTOR: i64 = 10000;
static BYTES: &[u8] = include_bytes_aligned!(1024, "../src/bytes");

/// Linear regression fits a line to a dataset such that OLS
/// is minimized.
struct LinearRegression {
    slope: i64,
    intercept: i64, 
    n: i64,
    sum_x: i64,
    sum_y: i64,
    sum_x_squared: i64,
    sum_xy: i64,
}

// A type containing pairs of x,y coordinates. It exists as a wrapper
// so that we can use a pointer-cast for super risky unsafe deserialization.
struct DataPairs {
    data: [(i64, i64); ARRAY_SIZE],
}

impl LinearRegression {
    fn new() -> Self {
        LinearRegression {
            slope: 0,
            intercept: 0,
            n: 0,
            sum_x: 0,
            sum_y: 0,
            sum_x_squared: 0,
            sum_xy: 0,
        }
    }

    /// Trains the regression one coordinate pair at a time,
    /// accumulating the values in the struct fields
    fn train(&mut self, x: i64, y: i64) {
        self.n += 1;
        self.sum_x += x;
        self.sum_y += y;
        self.sum_x_squared += x * x;
        self.sum_xy += x * y;

        // Calculate the slope and intercept using the least squares method
        if self.n > 1 {
            let numerator = self.n * self.sum_xy - self.sum_x * self.sum_y;
            let denominator = self.n * self.sum_x_squared - self.sum_x * self.sum_x;
            self.slope = numerator / denominator;
        }
        // β = ( ̃y −  ̃α ̃x) + L3
        // NoisyStats for DP Model
        let intercept_numerator = self.sum_y - self.slope * self.sum_x;
        self.intercept = (intercept_numerator / self.n) + laplace_mechanism(2 * SCALING_FACTOR, self.n, self.slope);
    }
}

/// Additive noise mechanism using laplace distribution
/// ## Arguments:
/// * epsilon: privacy paramter of mechanism, a good default value is 2
/// * n: size of dataset
/// * alpha: slope of regression to perturb
/// ## Returns:
/// * perturbed additive noise value
fn laplace_mechanism(epsilon: i64, n: i64, alpha: i64) -> i64 {
    let delta = SCALING_FACTOR - (SCALING_FACTOR / n);
    let l = laplace_noise(0, (3 * delta) / epsilon);
    let delta_3 = (SCALING_FACTOR / n) * (SCALING_FACTOR + (abs(alpha) + l) / SCALING_FACTOR);
    laplace_noise(0, 3 * delta_3 / epsilon)
}

/// Forms a laplace distribution
/// ## Arguments:
/// * x: mean of distribution
/// * b: location paramter of distribution
fn laplace_noise(x: i64, b: i64) -> i64 {
    if b == 0 {
        return 0; // Avoid division by zero by returning zero noise.
    }

    let x = x;
    let b = b;
    let exponent = abs(x * SCALING_FACTOR / b);
    let exp_result = powf_fixed(271828, -exponent / SCALING_FACTOR); // Euler's number approximated as 2.71828 * SCALING_FACTOR
    let result = (SCALING_FACTOR / 2 * b * exp_result) / SCALING_FACTOR;
    if !(i64::MIN..=i64::MAX).contains(&result) {
        panic!("Result out of i64 range");
    }

    result
}

/// Calculates |x|
fn abs(x: i64) -> i64 {
    x & 0x7fffffff
}

/// exp by squares calculates x^n
fn powf_fixed(x: i64, n: i64) -> i64 {
    if n == 0 {
        return SCALING_FACTOR; // 1.0 in fixed-point
    }
    if n < 0 {
        return SCALING_FACTOR / powf_fixed(x, -n); // Inverting for negative exponent
    }

    let mut result = SCALING_FACTOR; // Start with 1.0 in fixed-point
    let mut base = x;
    let mut exponent = n;

    while exponent > 0 {
        if exponent % 2 == 1 {
            result = result * base / SCALING_FACTOR;
        }
        base = base * base / SCALING_FACTOR;
        exponent /= 2;
    }
    result
}

/// Guest main runs the regression
pub fn main() {
    let mut lr = LinearRegression::new();
    let training_data: &DataPairs = unsafe_deserialize(BYTES);
    training_data.data.iter().for_each(|&(x, y)| lr.train(x, y));
    let result = (lr.slope, lr.intercept);
    env::commit(&result);
}

// Pointer-cast to the data instead of safe deserialize. This is super risky and generally
// a bad idea. But in this case we are assuming the role of a verifier who has constructed
// this application to be executed by a different party than themselves, and in this scenario
// we assume we have done due dilligence to embed the bytes into the program correctly.
// Further, this is a relatively safe operation in this setting because the datatype is a
// simple array of primitives in contiguous memory. I have not tested this on complex types, 
// something like rkyv might be better but it still has an overhead of about ~5 million cycles 
// regardless of data size.
fn unsafe_deserialize<T>(data: &[u8]) -> &T {
    assert_eq!(
        data.as_ptr() as usize % mem::align_of::<T>(),
        0,
        "Alignment mismatch"
    );
    unsafe { &*(data.as_ptr() as *const T) }
}