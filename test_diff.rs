// Test file for gitai_diff functionality
fn main() {
    println!("Testing gitai_diff tool - MODIFIED VERSION");
    
    // Improved version with better algorithm
    let mut counter = 0;
    for i in 0..20 {  // Changed from 10 to 20
        counter += i * 2;  // Added multiplication
    }
    println!("Enhanced Counter: {}", counter);
    
    // Some code to modify
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];  // Added more numbers
    let sum: i32 = data.iter().sum();
    let average = sum as f64 / data.len() as f64;  // Added average calculation
    println!("Sum: {}, Average: {:.2}", sum, average);
    
    // New functionality
    let max_value = data.iter().max().unwrap_or(&0);
    println!("Maximum value: {}", max_value);
}