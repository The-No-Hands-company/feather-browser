// Test async/await rejection handling
console.log("=== Testing Async/Await Rejection ===\n");

// Test 1: Basic rejection
async function testReject() {
    console.log("Test 1: Starting async function");
    try {
        const result = await Promise.reject("Error occurred!");
        console.log("This should not print:", result);
    } catch (error) {
        console.log("Caught error:", error);
    }
    console.log("Test 1: After try/catch");
}

// Test 2: Mixed resolve/reject
async function testMixed() {
    console.log("\nTest 2: Starting mixed test");
    try {
        const a = await Promise.resolve(10);
        console.log("Got a:", a);
        
        const b = await Promise.reject("Failed at b");
        console.log("This should not print:", b);
        
        const c = await Promise.resolve(30);
        console.log("This should not print:", c);
    } catch (error) {
        console.log("Caught error in mixed:", error);
    }
    console.log("Test 2: Complete");
}

// Run tests
console.log("Starting Test 1...");
testReject();

console.log("\n\nStarting Test 2...");
testMixed();

console.log("\n\n=== Test Complete ===");
