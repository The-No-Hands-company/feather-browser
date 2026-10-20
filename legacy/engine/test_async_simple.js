console.log("=== Test: Simple Async/Await ===");

async function testSimple() {
    console.log("1. Before await");
    const result = await Promise.resolve(42);
    console.log("2. After await, result:", result);
    return result;
}

const p = testSimple();
console.log("3. Async function returned");
console.log("4. Done");
