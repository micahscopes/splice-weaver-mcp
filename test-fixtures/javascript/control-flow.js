// If statements
if (condition) {
  console.log("Simple if");
}

if (condition) {
  console.log("If with else");
} else {
  console.log("Else branch");
}

if (condition1) {
  console.log("First condition");
} else if (condition2) {
  console.log("Second condition");
} else {
  console.log("Default");
}

// Ternary operator
const result = condition ? "true" : "false";
const nested = condition1 ? (condition2 ? "both" : "first") : "neither";

// Switch statements
switch (value) {
  case 1:
    console.log("One");
    break;
  case 2:
  case 3:
    console.log("Two or three");
    break;
  default:
    console.log("Other");
}

// For loops
for (let i = 0; i < 10; i++) {
  console.log(i);
}

for (const item of items) {
  console.log(item);
}

for (const key in object) {
  console.log(key, object[key]);
}

// While loops
let i = 0;
while (i < 10) {
  console.log(i);
  i++;
}

let j = 0;
do {
  console.log(j);
  j++;
} while (j < 5);

// Break and continue
for (let i = 0; i < 10; i++) {
  if (i === 3) continue;
  if (i === 7) break;
  console.log(i);
}

// Labeled statements
outer: for (let i = 0; i < 3; i++) {
  inner: for (let j = 0; j < 3; j++) {
    if (i === 1 && j === 1) break outer;
    console.log(i, j);
  }
}

// Try-catch-finally
try {
  riskyOperation();
} catch (error) {
  console.error("Error occurred:", error);
} finally {
  console.log("Always runs");
}

try {
  riskyOperation();
} catch (error) {
  if (error instanceof TypeError) {
    console.log("Type error");
  } else if (error instanceof ReferenceError) {
    console.log("Reference error");
  } else {
    console.log("Other error");
  }
}

// Throw statements
function validateAge(age) {
  if (age < 0) {
    throw new Error("Age cannot be negative");
  }
  if (age > 150) {
    throw new Error("Age seems unrealistic");
  }
  return age;
}