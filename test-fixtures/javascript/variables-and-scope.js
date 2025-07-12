// Variable declarations
var oldStyleVar = "old";
let blockScoped = "block";
const constant = "constant";

// Different assignment patterns
let a = 1;
let b = 2, c = 3;
let [x, y] = [10, 20];
let {name, age} = {name: "John", age: 30};

// Object property assignments
const obj = {};
obj.property = "value";
obj["dynamic"] = "property";

// Array operations
const arr = [1, 2, 3];
arr.push(4);
arr[0] = 10;

// Template literals
const message = `Hello, ${name}! You are ${age} years old.`;

// Conditional assignments
const result = condition ? "true" : "false";
const value = someValue || "default";
const nullish = someValue ?? "default";

// Type conversions
const stringNum = String(42);
const numString = Number("42");
const boolValue = Boolean(1);

// Operator assignments
let counter = 0;
counter += 5;
counter -= 2;
counter *= 3;
counter /= 2;
counter %= 3;
counter **= 2;

// Logical assignments (ES2021)
let config = {};
config.debug ??= false;
config.timeout ||= 5000;
config.retries &&= 3;

// Hoisting examples
console.log(hoistedVar); // undefined
var hoistedVar = "hoisted";

// Block scope examples
if (true) {
  let blockVar = "block";
  const blockConst = "block";
}

// Function scope
function scopeExample() {
  var functionScoped = "function";
  if (true) {
    var stillFunctionScoped = "function";
    let blockScoped = "block";
  }
}