#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
#![no_std]  // std support is experimental


use core::mem;
use include_bytes_aligned::include_bytes_aligned;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);
const ARRAY_SIZE: usize = 6000;
const SCALE_FACTOR: i32 = 100000;
static BYTES: &[u8] = include_bytes_aligned!(32, "../src/bytes_scaled");

/// Linear regression fits a line to a dataset such that OLS
/// is minimized.
struct LinearRegression {
    slope: i32,
    intercept: i32, 
    n: i32,
    sum_x: i32,
    sum_y: i32,
    sum_x_squared: i32,
    sum_xy: i32,
}

// A type containing pairs of x,y coordinates. It exists as a wrapper
// so that we can use a pointer-cast for super risky unsafe deserialization.
struct DataPairs {
    data: [(i32, i32); ARRAY_SIZE],
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
    // /// accumulating the values in the struct fields
    // fn train(&mut self, x: i32, y: i32) {
    //     self.n += 1;
    //     self.sum_x += x;
    //     self.sum_y += y;
    //     self.sum_x_squared += x * x;
    //     self.sum_xy += x * y;

    //     // Calculate the slope and intercept using the least squares method

    // let denominator = self.n * self.sum_x_squared - self.sum_x * self.sum_x;
    // if denominator != 0 {
    //     self.slope = (self.n * self.sum_xy - self.sum_x * self.sum_y) * SCALE_FACTOR / denominator;
    // } else {
    //     // Handle the case where denominator is zero
    //     // Perhaps set slope to some default or error value
    //     self.slope = 0;  // Example: default to 0 or consider other handling strategies
    // }

    //     // β = ( ̃y −  ̃α ̃x) + L3
    //     // NoisyStats for DP Model
    //     self.intercept = (self.sum_y - (self.slope * self.sum_x / SCALE_FACTOR)) / self.n + laplace_mechanism(2, self.n, self.slope);
    // }

    fn linear_regression(&self, data: &[(i32, i32)]) -> (i32, i32) {
        let scale = SCALE_FACTOR; // scale factor for fixed-point precision
        let n = ARRAY_SIZE as i32;
    
        let sum_x = data.iter().map(|(x, _)| x).sum::<i32>();
        let sum_y = data.iter().map(|(_, y)| y).sum::<i32>();
        let sum_xy = data.iter().map(|(x, y)| x * y).sum::<i32>();
        let sum_x2 = data.iter().map(|(x, _)| x.pow(2)).sum::<i32>();
    
        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = n * sum_x2 - sum_x.pow(2);
    
        // Scale down the slope and intercept to account for fixed-point arithmetic
        let m = (numerator * scale) / denominator;
        let b = (sum_y - m * sum_x) / n;
    
        (m, b)
    }

    // fn predict(&self, x: i32) -> i32 {
    //     self.slope * x + self.intercept
    // }
}



// /// Additive noise mechanism using laplace distribution
// /// ## Arguments:
// /// * epsilon: privacy paramter of mechanism, a good default value is 2
// /// * n: size of dataset
// /// * alpha: slope of regression to perturb
// /// ## Returns:
// /// * perturbed additive noise value
// fn laplace_mechanism(epsilon: i32, n: i32, alpha: i32) -> i32 {
//     let delta = SCALE_FACTOR - (SCALE_FACTOR / n);
//     let l = laplace_noise(0, (3 * delta) * SCALE_FACTOR / epsilon);
//     let delta_3 = (SCALE_FACTOR / n) * (1 + (abs(alpha) + l));
//     laplace_noise(0, 3 * delta_3 / epsilon)
// }

// /// Forms a laplace distribution
// /// ## Arguments:
// /// * x: mean of distribution
// /// * b: location paramter of distribution
// fn laplace_noise(x: i32, b: i32) -> i32 {

//     if b == 0 {
//         // Handle division by zero or return a default error value or notification
//         return 0; // Example: return zero or an appropriate error code
//     }

//     let exponent = abs(x / b);
//     // Ensure that b is not zero before division
//     // Also adjust the order of operations to avoid premature integer division result of zero
//     let base_scale = SCALE_FACTOR; // Scale to prevent integer division issues
//     let pow_result = powf(271828182i32, -exponent); // Calculate the exponential decay

//     // Adjust the multiplication and division to account for the scaling factor
//     (base_scale / (2 * b)) * pow_result / base_scale
// }



// /// Calculates |x|
// fn abs(x: i32) -> i32 {
//     x & 0x7fffffff
// }

// /// exp by squares calculates x^n

// fn powf(x: i32, n: i32) -> i32 {
//     if n == 0 {
//         return SCALE_FACTOR; // Equivalent to 1 in scaled terms
//     }
//     if n < 0 {
//         return SCALE_FACTOR / powf(x, -n); // Handling for negative exponent
//     }

//     let mut result = SCALE_FACTOR; // Scaled 1 for fixed-point arithmetic
//     let mut base = x;
//     let mut exponent = n;

//     while exponent > 0 {
//         if exponent % 2 == 1 {
//             result = result * base / SCALE_FACTOR; // Scale back down after multiplication
//         }
//         base = base * base / SCALE_FACTOR; // Prevent overflow by scaling down each step
//         exponent /= 2;
//     }
//     result
// }

/// Guest main runs the regression
pub fn main() {
    let lr = LinearRegression::new();
    let training_data: &DataPairs = unsafe_deserialize(BYTES);
    let (slope, intercept) = lr.linear_regression(&training_data.data);

    let result = (slope, intercept);
    env::commit(&(result));
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