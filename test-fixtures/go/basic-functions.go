package main

import (
    "fmt"
    "errors"
    "context"
    "time"
)

// Basic function definitions
func greet(name string) string {
    return fmt.Sprintf("Hello, %s!", name)
}

func add(a, b int) int {
    return a + b
}

// Functions with multiple return values
func divide(a, b float64) (float64, error) {
    if b == 0 {
        return 0, errors.New("division by zero")
    }
    return a / b, nil
}

// Functions with named return values
func calculateStats(numbers []int) (sum, count int, average float64) {
    sum = 0
    count = len(numbers)
    for _, num := range numbers {
        sum += num
    }
    if count > 0 {
        average = float64(sum) / float64(count)
    }
    return
}

// Variadic functions
func sum(numbers ...int) int {
    total := 0
    for _, num := range numbers {
        total += num
    }
    return total
}

func printf(format string, args ...interface{}) {
    fmt.Printf(format, args...)
}

// Functions with pointers
func increment(x *int) {
    *x++
}

func swap(a, b *int) {
    *a, *b = *b, *a
}

// Functions with slices
func reverse(slice []int) []int {
    result := make([]int, len(slice))
    for i, v := range slice {
        result[len(slice)-1-i] = v
    }
    return result
}

func filterEven(numbers []int) []int {
    var result []int
    for _, num := range numbers {
        if num%2 == 0 {
            result = append(result, num)
        }
    }
    return result
}

// Functions with maps
func countWords(text string) map[string]int {
    words := make(map[string]int)
    // Simple word counting (would need proper tokenization)
    words["example"] = 1
    return words
}

// Functions with structs
type Person struct {
    Name string
    Age  int
}

func createPerson(name string, age int) Person {
    return Person{Name: name, Age: age}
}

func (p Person) greet() string {
    return fmt.Sprintf("Hello, I'm %s and I'm %d years old", p.Name, p.Age)
}

func (p *Person) birthday() {
    p.Age++
}

// Functions with interfaces
type Shape interface {
    Area() float64
    Perimeter() float64
}

type Rectangle struct {
    Width, Height float64
}

func (r Rectangle) Area() float64 {
    return r.Width * r.Height
}

func (r Rectangle) Perimeter() float64 {
    return 2 * (r.Width + r.Height)
}

func describeShape(s Shape) string {
    return fmt.Sprintf("Area: %.2f, Perimeter: %.2f", s.Area(), s.Perimeter())
}

// Generic functions (Go 1.18+)
func Max[T comparable](a, b T) T {
    if a > b {
        return a
    }
    return b
}

func Map[T, U any](slice []T, fn func(T) U) []U {
    result := make([]U, len(slice))
    for i, v := range slice {
        result[i] = fn(v)
    }
    return result
}

// Functions with channels
func producer(ch chan<- int) {
    for i := 0; i < 5; i++ {
        ch <- i
        time.Sleep(100 * time.Millisecond)
    }
    close(ch)
}

func consumer(ch <-chan int) {
    for value := range ch {
        fmt.Printf("Received: %d\n", value)
    }
}

// Functions with context
func processWithTimeout(ctx context.Context, data string) error {
    select {
    case <-time.After(2 * time.Second):
        fmt.Printf("Processed: %s\n", data)
        return nil
    case <-ctx.Done():
        return ctx.Err()
    }
}

// Functions with error handling
func readFile(filename string) ([]byte, error) {
    // Simulate file reading
    if filename == "" {
        return nil, errors.New("filename cannot be empty")
    }
    return []byte("file content"), nil
}

func processFile(filename string) error {
    data, err := readFile(filename)
    if err != nil {
        return fmt.Errorf("failed to read file %s: %w", filename, err)
    }
    fmt.Printf("File content: %s\n", string(data))
    return nil
}

// Functions with defer
func resourceManager() error {
    resource := "important resource"
    defer func() {
        fmt.Printf("Cleaning up: %s\n", resource)
    }()
    
    // Simulate some work
    fmt.Printf("Using: %s\n", resource)
    return nil
}

// Recursive functions
func factorial(n int) int {
    if n <= 1 {
        return 1
    }
    return n * factorial(n-1)
}

func fibonacci(n int) int {
    if n <= 1 {
        return n
    }
    return fibonacci(n-1) + fibonacci(n-2)
}

// Functions with panic and recover
func safeOperation() {
    defer func() {
        if r := recover(); r != nil {
            fmt.Printf("Recovered from panic: %v\n", r)
        }
    }()
    
    // This might panic
    panic("something went wrong")
}

// Higher-order functions
func applyOperation(a, b int, op func(int, int) int) int {
    return op(a, b)
}

func createMultiplier(factor int) func(int) int {
    return func(x int) int {
        return x * factor
    }
}

// Functions with type assertions
func processValue(v interface{}) {
    switch value := v.(type) {
    case int:
        fmt.Printf("Integer: %d\n", value)
    case string:
        fmt.Printf("String: %s\n", value)
    case bool:
        fmt.Printf("Boolean: %t\n", value)
    default:
        fmt.Printf("Unknown type: %T\n", value)
    }
}

// Main function
func main() {
    fmt.Println(greet("World"))
    fmt.Println(add(5, 3))
    
    result, err := divide(10, 2)
    if err != nil {
        fmt.Printf("Error: %v\n", err)
    } else {
        fmt.Printf("Result: %f\n", result)
    }
    
    fmt.Println(sum(1, 2, 3, 4, 5))
    
    p := createPerson("Alice", 30)
    fmt.Println(p.greet())
    p.birthday()
    fmt.Println(p.greet())
}