#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
#![no_std]  // std support is experimental

extern crate alloc;
use alloc::vec::Vec;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

struct LinearRegression {
    slope: f32,
    intercept: f32,
    n: usize,
    sum_x: f32,
    sum_y: f32,
    sum_x_squared: f32,
    sum_xy: f32,
}

impl LinearRegression {
    fn new() -> Self {
        LinearRegression {
            slope: 0.0,
            intercept: 0.0,
            n: 0,
            sum_x: 0.0,
            sum_y: 0.0,
            sum_x_squared: 0.0,
            sum_xy: 0.0,
        }
    }

    fn train(&mut self, x: f32, y: f32) {
        self.n += 1;
        self.sum_x += x;
        self.sum_y += y;
        self.sum_x_squared += x * x;
        self.sum_xy += x * y;

        // Calculate the slope and intercept using the least squares method
        self.slope = (self.n as f32 * self.sum_xy - self.sum_x * self.sum_y)
            / (self.n as f32 * self.sum_x_squared - self.sum_x * self.sum_x);

        // β = ( ̃y −  ̃α ̃x) + L3
        self.intercept = (self.sum_y - self.slope * self.sum_x) / self.n as f32
            - laplace_mechanism(2.0, 1200.0, self.slope);
    }

    fn predict(&self, x: f32) -> f32 {
        self.slope * x + self.intercept
    }

    fn get_slope(&self) -> f32 {
        self.slope
    }

    fn get_intercept(&self) -> f32 {
        self.intercept
    }
}

fn laplace_noise(x: f32, b: f32) -> f32 {
    let exponent = abs(x / b);
    (1.0 / 2.0 * b) * powf(2.7182817_f32, -exponent)
}

fn laplace_mechanism(epsilon: f32, n: f32, alpha: f32) -> f32 {
    let delta = 1.0 - (1.0 / n);
    let l = laplace_noise(0.0, (3.0 * delta) / epsilon);
    let delta_3 = (1.0 / n) * (1.0 + (abs(alpha) + l));
    laplace_noise(0.0, 3.0 * delta_3 / epsilon)
}

// Get variance and covariance of two vectors
fn get_cov_and_var(x: &Vec<f32>, y: &[f32], l: f32) -> Option<f32> {
    let cov_xy = x
        .iter()
        .zip(y.iter())
        .fold(0.0, |acc, (xi, yi)| acc + xi * yi)
        / (x.len() as f32)
        + l;
    let var_x = x.iter().fold(0.0, |acc, xi| acc + xi * xi) / (x.len() as f32) + l;

    if var_x != 0.0 {
        let ratio = cov_xy / var_x;
        Some(ratio)
    } else {
        None
    }
}

fn abs(x: f32) -> f32{

    let result = match x {
        n if n < 0.0 => -n,
        _ => x,
    };
    result
}

fn powf(x: f32, n: f32) -> f32 {
    if n == 0.0 {
        return 1.0;
    }

    if n < 0.0 {
        return 1.0 / powf(x, -n);
    }

    let mut result = 1.0;
    let mut base = x;
    let mut exponent = n;

    while exponent > 0.0 {
        if exponent % 2.0 == 1.0 {
            result *= base;
        }

        base *= base;
        exponent /= 2.0;
    }

    result
}


pub fn main() {

    let mut x_vec = Vec::new();
    let mut y_vec = Vec::new();

    let mut lr = LinearRegression::new();

    for _ in 0..1200 {
        let x: f32 = env::read();
        x_vec.push(x);
    }

    for _ in 0..1200 {
        let y: f32 = env::read();
        y_vec.push(y);
    }

    for i in 0..1200 {
        lr.train(x_vec[i], y_vec[i])
    }

    env::commit(&(lr.intercept, lr.slope));
}
