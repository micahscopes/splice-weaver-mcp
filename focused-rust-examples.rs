// Focused Rust examples for LLM usability testing
// These represent common patterns LLMs need to work with

// 1. Error handling patterns - common refactoring target
fn process_data(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parsed = input.parse::<i32>()?;
    let result = format!("Processed: {}", parsed);
    Ok(result)
}

fn risky_operation() -> Result<i32, &'static str> {
    if true {
        Ok(42)
    } else {
        Err("Something went wrong")
    }
}

// 2. Function patterns - signature changes, async conversion
fn synchronous_function(x: i32, y: i32) -> i32 {
    x + y
}

fn another_sync_function(data: Vec<String>) -> Vec<String> {
    data.into_iter().map(|s| s.to_uppercase()).collect()
}

// 3. Struct patterns - adding fields, changing visibility
struct User {
    name: String,
    age: u32,
}

struct Product {
    id: u64,
    name: String,
    price: f64,
}

// 4. Implementation blocks - adding methods, changing signatures
impl User {
    fn new(name: String, age: u32) -> Self {
        User { name, age }
    }
    
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn is_adult(&self) -> bool {
        self.age >= 18
    }
}

// 5. Match expressions - pattern matching refactoring
fn handle_option(value: Option<i32>) -> String {
    match value {
        Some(x) if x > 0 => format!("Positive: {}", x),
        Some(x) => format!("Non-positive: {}", x),
        None => "No value".to_string(),
    }
}

// 6. Iterator chains - common optimization target
fn process_numbers(numbers: Vec<i32>) -> Vec<String> {
    numbers
        .iter()
        .filter(|&&x| x > 0)
        .map(|&x| x * 2)
        .filter(|&x| x < 100)
        .map(|x| format!("Result: {}", x))
        .collect()
}

// 7. Nested function calls - refactoring target
fn complex_calculation(data: Vec<i32>) -> i32 {
    data.iter()
        .map(|x| x * 2)
        .filter(|&x| x > 10)
        .fold(0, |acc, x| acc + x)
}

// 8. Variable declarations - type annotations, mutability
fn variable_examples() {
    let x = 42;
    let mut y = String::new();
    let z: Vec<i32> = vec![1, 2, 3];
    
    y.push_str("hello");
    println!("{}, {}, {:?}", x, y, z);
}