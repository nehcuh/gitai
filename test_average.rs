fn calculate_average(numbers: &[i32]) -> f64 {
    if numbers.is_empty() {
        return 0.0;
    }
    
    let sum: i32 = numbers.iter().sum();
    sum as f64 / numbers.len() as f64
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let average = calculate_average(&numbers);
    println!("Average: {}", average);
}