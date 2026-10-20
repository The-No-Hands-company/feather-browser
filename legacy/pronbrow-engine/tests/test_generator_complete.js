// Test multi-yield generator execution
function* multiYield() {
    console.log("Before yield 1");
    yield 1;
    console.log("Before yield 2");
    yield 2;
    console.log("Before yield 3");
    yield 3;
    console.log("After all yields");
    return "done";
}

console.log("=== Testing Multi-Yield Generator ===");
const gen = multiYield();

console.log("\nCalling next() #1:");
const r1 = gen.next();
console.log("Result:", r1);

console.log("\nCalling next() #2:");
const r2 = gen.next();
console.log("Result:", r2);

console.log("\nCalling next() #3:");
const r3 = gen.next();
console.log("Result:", r3);

console.log("\nCalling next() #4:");
const r4 = gen.next();
console.log("Result:", r4);

console.log("\n=== Test Complete ===");
