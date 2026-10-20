// Test async/await rejection handling
console.log("=== Test 1: Simple rejection ===");

async function testReject() {
    try {
        const result = await Promise.reject("Test error");
        console.log("This should not execute");
    } catch (e) {
        console.log("Caught rejection:", e);
    }
}

testReject();

console.log("\n=== Test 2: Rejection without catch ===");

async function testUnhandled() {
    const result = await Promise.reject("Unhandled error");
    console.log("This should not execute");
}

// This should cause an unhandled rejection
testUnhandled();

console.log("\n=== Test 3: Multiple awaits with rejection in middle ===");

async function testMultiAwait() {
    try {
        const a = await Promise.resolve(1);
        console.log("Got a:", a);
        
        const b = await Promise.reject("Error in middle");
        console.log("This should not execute");
        
        const c = await Promise.resolve(3);
        console.log("This should not execute either");
    } catch (e) {
        console.log("Caught error:", e);
    }
}

testMultiAwait();

console.log("\n=== Test 4: Rejection with Error object ===");

async function testErrorObject() {
    try {
        const err = { message: "Custom error message", code: 42 };
        await Promise.reject(err);
    } catch (e) {
        console.log("Caught error object:", e);
    }
}

testErrorObject();

console.log("\nAll rejection tests complete!");
