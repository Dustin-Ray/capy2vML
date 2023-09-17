#![no_main]
// If you want to try std support, also update the guest Cargo.toml file
#![no_std] // std support is experimental

extern crate alloc;
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
        // NoisyStats for DP Model
        self.intercept = (self.sum_y - self.slope * self.sum_x) / self.n as f32
            + laplace_mechanism(2.0, self.n as f32, self.slope);
    }

    // fn predict(&self, x: f32) -> f32 {
    //     self.slope * x + self.intercept
    // }
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

fn abs(x: f32) -> f32 {
    let result = match x {
        n if n < 0.0 => -n,
        _ => x,
    };
    result
}

fn powf(x: f32, y: f32) -> f32 {
    if y == 0.0 {
        // Anything raised to the power of 0 is 1
        return 1.0;
    } else if y == 1.0 {
        // Anything raised to the power of 1 is itself
        return x;
    } else if y.is_infinite() {
        // Handle infinite y
        return if x == 1.0 { 1.0 } else { 0.0 };
    } else if x == 0.0 {
        // Handle x == 0
        return 0.0;
    }

    let mut result = 1.0;
    let mut exp = abs(y) as u32;

    let mut base = x;

    while exp > 0 {
        if exp % 2 == 1 {
            // If exp is odd, multiply the result by the base
            result *= base;
        }
        // Square the base and halve the exponent
        base *= base;
        exp /= 2;
    }

    if y < 0.0 {
        // If y is negative, take the reciprocal of the result
        result = 1.0 / result;
    }

    result
}

pub fn main() {
    let mut lr = LinearRegression::new();
    //assumes a single column stream of interleaved data i.e. (x,y,x,y...etc), 1200 is length of data
    let training_data = (0..1200).map(|_| (env::read(), env::read()));
    training_data.for_each(|(x, y)| lr.train(x, y));
    let cycles = env::get_cycle_count();
    let result = (lr.intercept, lr.slope, cycles);
    env::commit(&(result));
}
