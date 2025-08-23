fn calculate_total(prices: &[f64]) -> f64 {
    let mut total = 0.0;
    for price in prices {
        total += price;
    }
    total
}

fn main() {
    let prices = vec![10.0, 20.0, 30.0];
    let result = calculate_total(&prices);
    println!("Total: {}", result);
}