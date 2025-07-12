// Basic function definitions
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Functions with multiple parameters
fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

// Functions with no return value
fn print_message(message: &str) {
    println!("{}", message);
}

// Functions with early return
fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        return None;
    }
    Some(a / b)
}

// Functions with pattern matching
fn describe_point(point: (i32, i32)) -> String {
    match point {
        (0, 0) => "Origin".to_string(),
        (0, y) => format!("On Y-axis at {}", y),
        (x, 0) => format!("On X-axis at {}", x),
        (x, y) => format!("Point at ({}, {})", x, y),
    }
}

// Generic functions
fn swap<T>(a: T, b: T) -> (T, T) {
    (b, a)
}

fn find_max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// Functions with lifetimes
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

// Functions with closures
fn apply_operation<F>(x: i32, y: i32, op: F) -> i32
where
    F: Fn(i32, i32) -> i32,
{
    op(x, y)
}

// Higher-order functions
fn create_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
    move |x| x * factor
}

// Functions with references and borrowing
fn process_data(data: &mut Vec<i32>) {
    data.push(42);
    data.sort();
}

// Functions with error handling
fn parse_number(s: &str) -> Result<i32, std::num::ParseIntError> {
    s.parse::<i32>()
}

// Recursive functions
fn factorial(n: u32) -> u32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

// Functions with destructuring
fn process_tuple((x, y): (i32, i32)) -> i32 {
    x + y
}

// Associated functions (static methods)
impl Calculator {
    fn new() -> Self {
        Calculator { value: 0 }
    }
    
    fn add(&mut self, x: i32) {
        self.value += x;
    }
    
    fn get_value(&self) -> i32 {
        self.value
    }
}

struct Calculator {
    value: i32,
}

// Functions with attributes
#[inline]
fn fast_operation(x: i32) -> i32 {
    x * 2
}

#[deprecated(note = "Use new_function instead")]
fn old_function() {
    println!("This is deprecated");
}

// Unsafe functions
unsafe fn dangerous_operation(ptr: *const i32) -> i32 {
    *ptr
}

// Async functions
async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    Ok(text)
}