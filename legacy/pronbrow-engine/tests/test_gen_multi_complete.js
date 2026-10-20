// Test multi-yield generator with console.log to verify PC tracking

function* gen() {
    console.log("Statement 1");
    yield 1;
    console.log("Statement 2");
    yield 2;
    console.log("Statement 3");
    yield 3;
    console.log("Done");
}

console.log("Creating generator");
const g = gen();

console.log("\n--- First next() ---");
const r1 = g.next();
console.log("Result:", JSON.stringify(r1));

console.log("\n--- Second next() ---");
const r2 = g.next();
console.log("Result:", JSON.stringify(r2));

console.log("\n--- Third next() ---");
const r3 = g.next();
console.log("Result:", JSON.stringify(r3));

console.log("\n--- Fourth next() ---");
const r4 = g.next();
console.log("Result:", JSON.stringify(r4));

console.log("\n=== TEST COMPLETE ===");
console.log("Expected behavior:");
console.log("  Each console.log should appear only once");
console.log("  Each next() should return sequential values");
