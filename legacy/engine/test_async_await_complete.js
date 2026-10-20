console.log("=== Test 1: Single Await ===");
async function testSingleAwait() {
    console.log("Before await");
    const result = await Promise.resolve(42);
    console.log("After await, result:", result);
    return result * 2;
}

const promise1 = testSingleAwait();
console.log("Async function returned a promise");

promise1.then(value => {
    console.log("Final result:", value);
});

console.log("\n=== Test 2: Multiple Awaits (Sequential) ===");
async function testMultipleAwaits() {
    console.log("First await...");
    const a = await Promise.resolve(10);
    console.log("Got a =", a);
    
    const b = await Promise.resolve(20);
    console.log("Got b =", b);
    
    const c = await Promise.resolve(30);
    console.log("Got c =", c);
    
    return a + b + c;
}

const promise2 = testMultipleAwaits();
promise2.then(value => {
    console.log("Sum result:", value);
});

console.log("\n=== Test 3: Await Already Fulfilled Promise ===");
async function testFulfilledPromise() {
    const promise = Promise.resolve(100);
    const result = await promise;
    console.log("Got fulfilled result:", result);
    return result;
}

const promise3 = testFulfilledPromise();
promise3.then(value => {
    console.log("Fulfilled test result:", value);
});

console.log("\n=== Test 4: Async Function Returning Value ===");
async function testDirectReturn() {
    const x = 5;
    const y = 10;
    return x + y;
}

const promise4 = testDirectReturn();
promise4.then(value => {
    console.log("Direct return result:", value);
});

console.log("\nAll tests initiated. Processing microtasks...");
