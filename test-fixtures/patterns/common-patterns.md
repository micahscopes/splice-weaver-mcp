# Common AST-Grep Patterns

Pattern examples for use with the MCP ast-grep server. For complete documentation: https://ast-grep.github.io/

This document contains examples of ast-grep patterns for different programming languages.

## JavaScript/TypeScript Patterns

### Function Declarations
```javascript
// Pattern: function $NAME($PARAMS) { $BODY }
function greet(name) {
  return "Hello, " + name;
}

// Pattern: const $NAME = ($PARAMS) => $BODY
const add = (a, b) => a + b;

// Pattern: async function $NAME($PARAMS) { $BODY }
async function fetchData(url) {
  return await fetch(url);
}
```

### Variable Declarations
```javascript
// Pattern: var $NAME = $VALUE
var oldStyle = "value";

// Pattern: let $NAME = $VALUE
let blockScoped = "value";

// Pattern: const $NAME = $VALUE
const constant = "value";
```

### Console Operations
```javascript
// Pattern: console.log($ARG)
console.log("Debug message");

// Pattern: console.$METHOD($ARGS)
console.error("Error message");
console.warn("Warning message");
```

### Object Operations
```javascript
// Pattern: {$KEY: $VALUE}
const obj = {name: "John", age: 30};

// Pattern: $OBJ.$PROP = $VALUE
obj.name = "Jane";

// Pattern: $OBJ[$KEY] = $VALUE
obj["dynamic"] = "value";
```

## Python Patterns

### Function Definitions
```python
# Pattern: def $NAME($PARAMS): $$$
def greet(name):
    return f"Hello, {name}"

# Pattern: def $NAME(self, $PARAMS): $$$
def method(self, value):
    return value * 2
```

### Class Definitions
```python
# Pattern: class $NAME: $$$
class Simple:
    pass

# Pattern: class $NAME($BASE): $$$
class Child(Parent):
    pass
```

### Import Statements
```python
# Pattern: import $MODULE
import os

# Pattern: from $MODULE import $NAME
from datetime import datetime

# Pattern: from $MODULE import $NAME as $ALIAS
from datetime import datetime as dt
```

## Rust Patterns

### Function Definitions
```rust
// Pattern: fn $NAME($PARAMS) -> $RETURN { $BODY }
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Pattern: fn $NAME($PARAMS) { $BODY }
fn greet(name: &str) {
    println!("Hello, {}!", name);
}

// Pattern: pub fn $NAME($PARAMS) -> $RETURN { $BODY }
pub fn public_function() -> String {
    "public".to_string()
}
```

### Struct Definitions
```rust
// Pattern: struct $NAME { $FIELDS }
struct Point {
    x: i32,
    y: i32,
}

// Pattern: pub struct $NAME { $FIELDS }
pub struct PublicPoint {
    pub x: i32,
    pub y: i32,
}
```

### Match Expressions
```rust
// Pattern: match $EXPR { $ARMS }
match value {
    0 => "zero",
    1 => "one",
    _ => "other",
}
```

## Go Patterns

### Function Definitions
```go
// Pattern: func $NAME($PARAMS) $RETURN { $BODY }
func add(a, b int) int {
    return a + b
}

// Pattern: func $NAME($PARAMS) { $BODY }
func greet(name string) {
    fmt.Printf("Hello, %s\n", name)
}

// Pattern: func ($RECEIVER) $NAME($PARAMS) $RETURN { $BODY }
func (p Person) greet() string {
    return fmt.Sprintf("Hello, %s", p.name)
}
```

### Type Definitions
```go
// Pattern: type $NAME struct { $FIELDS }
type Person struct {
    name string
    age  int
}

// Pattern: type $NAME interface { $METHODS }
type Reader interface {
    Read([]byte) (int, error)
}
```

## Java Patterns

### Method Definitions
```java
// Pattern: public $RETURN $NAME($PARAMS) { $BODY }
public String greet(String name) {
    return "Hello, " + name;
}

// Pattern: private $RETURN $NAME($PARAMS) { $BODY }
private void helper() {
    System.out.println("Helper method");
}

// Pattern: public static $RETURN $NAME($PARAMS) { $BODY }
public static int add(int a, int b) {
    return a + b;
}
```

### Class Definitions
```java
// Pattern: public class $NAME { $BODY }
public class Calculator {
    // methods
}

// Pattern: public class $NAME extends $SUPER { $BODY }
public class Advanced extends Calculator {
    // methods
}
```

## Common Transformation Patterns

### Modernization
```javascript
// var to const/let
// Pattern: var $NAME = $VALUE
// Replace: const $NAME = $VALUE

// ES5 to ES6 functions
// Pattern: function($PARAMS) { $BODY }
// Replace: ($PARAMS) => { $BODY }
```

### Debugging
```javascript
// Add logging
// Pattern: function $NAME($PARAMS) { $BODY }
// Replace: function $NAME($PARAMS) { console.log('Entering', '$NAME'); $BODY }

// Remove console.log
// Pattern: console.log($ARGS);
// Replace: 
```

### Error Handling
```javascript
// Wrap in try-catch
// Pattern: $STATEMENT
// Replace: try { $STATEMENT } catch (error) { console.error(error); }
```

## Pattern Writing Tips

1. **Start Simple**: Begin with basic patterns and gradually add complexity
2. **Test Incrementally**: Test each pattern with simple examples first
3. **Use Specific Patterns**: `console.log($ARG)` is better than `$ANY`
4. **Capture What You Need**: Only capture variables you'll use in replacements
5. **Mind the Whitespace**: Patterns need to match the exact structure
6. **Language-Specific**: Each language has different AST structures