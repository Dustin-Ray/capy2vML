#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
#![no_std] // std support is experimental

extern crate alloc;
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);

/// Linear regression fits a line to a dataset such that OLS
/// is minimized.
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

    /// Trains the regression one coordinate pair at a time,
    /// accumulating the values in the struct fields
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
        // NoisyStats for DP Model
        self.intercept = (self.sum_y - self.slope * self.sum_x) / self.n as f32 + laplace_mechanism(2.0, self.n as f32, self.slope);
    }

    // fn predict(&self, x: f32) -> f32 {
    //     self.slope * x + self.intercept
    // }
}

/// Forms a laplace distribution
/// ## Arguments:
/// * x: mean of distribution
/// * b: location paramter of distribution
fn laplace_noise(x: f32, b: f32) -> f32 {
    let exponent = abs(x / b);
    (1.0 / 2.0 * b) * powf(2.71828182845904523536028747135266250f32, -exponent)
}

/// Additive noise mechanism using laplace distribution
/// ## Arguments:
/// * epsilon: privacy paramter of mechanism, a good default value is 2
/// * n: size of dataset
/// * alpha: slope of regression to perturb
/// ## Returns:
/// * perturbed additive noise value
fn laplace_mechanism(epsilon: f32, n: f32, alpha: f32) -> f32 {
    let delta = 1.0 - (1.0 / n);
    let l = laplace_noise(0.0, (3.0 * delta) / epsilon);
    let delta_3 = (1.0 / n) * (1.0 + (abs(alpha) + l));
    laplace_noise(0.0, 3.0 * delta_3 / epsilon)
}

/// Calculates |x|
fn abs(x: f32) -> f32 {
    let result = match x {
        n if n < 0.0 => -n,
        _ => x,
    };
    result
}

/// exp by squares calculates x^n
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

/// Guest main runs the regression
pub fn main() {
    let mut lr = LinearRegression::new();
    //assumes a single column stream of interleaved data i.e. (x,y,x,y...etc), 1200 is length of data
    let training_data = (0..1200).map(|_| (env::read(), env::read()));
    training_data.for_each(|(x, y)| lr.train(x, y));
    let cycles = env::get_cycle_count();
    let result = (lr.slope, lr.intercept, cycles);
    env::commit(&(result));
}
