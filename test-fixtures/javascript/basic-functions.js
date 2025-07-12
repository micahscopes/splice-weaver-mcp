// Basic function declarations
function greet(name) {
  console.log("Hello, " + name);
  return "Hello, " + name;
}

// Arrow functions
const add = (a, b) => a + b;
const multiply = (x, y) => {
  return x * y;
};

// Function expressions
const subtract = function(a, b) {
  return a - b;
};

// Methods in objects
const calculator = {
  add: function(a, b) {
    return a + b;
  },
  subtract(a, b) {
    return a - b;
  }
};

// Async functions
async function fetchData(url) {
  try {
    const response = await fetch(url);
    return await response.json();
  } catch (error) {
    console.error("Error fetching data:", error);
    throw error;
  }
}

// Generator functions
function* numberGenerator() {
  yield 1;
  yield 2;
  yield 3;
}

// Higher-order functions
function createMultiplier(factor) {
  return function(x) {
    return x * factor;
  };
}

// Destructuring in parameters
function processUser({name, age, email}) {
  console.log(`Processing user: ${name}, ${age}, ${email}`);
}

// Rest parameters
function sum(...numbers) {
  return numbers.reduce((total, num) => total + num, 0);
}

// Default parameters
function greetUser(name = "World", greeting = "Hello") {
  return `${greeting}, ${name}!`;
}