// Test file for ast-grep Rust functionality
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

fn main() {
    println!("Testing Rust ast-grep analysis");

    // This should trigger unwrap warning
    let file = File::open("test.txt").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // Some normal code
    let numbers = vec![1, 2, 3, 4, 5];
    let result = process_numbers(&numbers);

    match result {
        Ok(value) => println!("Result: {}", value),
        Err(e) => eprintln!("Error: {}", e),
    }

    // This should trigger todo warning
    todo!("Implement proper error handling");
}

fn process_numbers(numbers: &[i32]) -> Result<i32, String> {
    if numbers.is_empty() {
        return Err("Empty vector".to_string());
    }

    // More unwrap usage that should be detected
    let first = numbers.first().unwrap();
    let last = numbers.last().unwrap();

    Ok(first + last)
}

struct DataProcessor {
    data: HashMap<String, i32>,
}

impl DataProcessor {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn add_entry(&mut self, key: String, value: i32) {
        self.data.insert(key, value);
    }

    fn get_value(&self, key: &str) -> i32 {
        // Another unwrap that should be caught
        *self.data.get(key).unwrap()
    }

    fn calculate_sum(&self) -> i32 {
        self.data.values().sum()
    }

    fn find_max(&self) -> Option<i32> {
        self.data.values().max().copied()
    }

    fn unimplemented_feature(&self) -> String {
        // Another todo that should be detected
        todo!("This feature is not yet implemented")
    }
}

// Generic function
fn generic_function<T>(value: T) -> T
where
    T: Clone,
{
    value.clone()
}

// Function with error handling (good practice)
fn safe_divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// Async function
async fn async_operation() -> Result<String, Box<dyn std::error::Error>> {
    // Simulated async work
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok("Async operation completed".to_string())
}

// Macro usage
macro_rules! debug_print {
    ($($arg:tt)*) => {
        println!("[DEBUG] {}", format!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_divide() {
        assert_eq!(safe_divide(10.0, 2.0).unwrap(), 5.0);
        assert!(safe_divide(10.0, 0.0).is_err());
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        processor.add_entry("test".to_string(), 42);

        // This unwrap in test is somewhat acceptable but should still be detected
        assert_eq!(processor.get_value("test"), 42);
    }

    #[test]
    fn test_with_panic() {
        // This should also trigger unwrap detection
        let result = Some(42);
        assert_eq!(result.unwrap(), 42);
    }
}
