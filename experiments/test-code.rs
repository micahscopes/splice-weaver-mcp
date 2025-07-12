// Test code for CLI experiments
fn main() {
    let x = 5;
    let y = 10;
    println!("Hello, world!");
    println!("x = {}, y = {}", x, y);
}

fn risky_function() -> Result<i32, &'static str> {
    let value = some_operation()?;
    Ok(value.unwrap()) // This is bad - double unwrap!
}

fn some_operation() -> Result<Option<i32>, &'static str> {
    Ok(Some(42))
}

fn process_data(items: Vec<i32>) -> Vec<String> {
    items
        .iter()
        .filter(|&&x| x > 0)
        .map(|&x| format!("Item: {}", x))
        .collect()
}