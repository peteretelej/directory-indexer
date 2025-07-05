// Simple Rust hello world program
fn main() {
    println!("Hello, world!");
    let greeting = "Welcome to Rust programming";
    println!("{}", greeting);
}

// Function to demonstrate error handling
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Cannot divide by zero".to_string())
    } else {
        Ok(a / b)
    }
}