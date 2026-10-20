// Test async/await with true suspension
// This tests the newly implemented Promise integration

console.log("=== Async/Await Suspension Test ===");

// Test 1: Simple async function
async function simpleAsync() {
    console.log("Before await");
    const result = await Promise.resolve(42);
    console.log("After await, result:", result);
    return result;
}

// Test 2: Multiple awaits
async function multipleAwaits() {
    console.log("First await");
    const a = await Promise.resolve(1);
    console.log("Second await");
    const b = await Promise.resolve(2);
    console.log("Third await");
    const c = await Promise.resolve(3);
    return a + b + c;
}

// Test 3: Await in expression
async function awaitInExpression() {
    const sum = (await Promise.resolve(10)) + (await Promise.resolve(20));
    return sum;
}

// Run tests
console.log("Test 1: Simple async");
simpleAsync().then(result => {
    console.log("Test 1 complete, result:", result);
});

console.log("Test 2: Multiple awaits");
multipleAwaits().then(result => {
    console.log("Test 2 complete, result:", result);
});

console.log("Test 3: Await in expression");
awaitInExpression().then(result => {
    console.log("Test 3 complete, result:", result);
});

console.log("=== Tests Queued ===");
