console.log("=== Advanced Async/Await Test ===");

async function test1() {
    console.log("Test 1: Start");
    const result = await Promise.resolve(10);
    console.log("Test 1: Got", result);
    return result * 2;
}

async function test2() {
    console.log("Test 2: Start");
    const a = await Promise.resolve(5);
    console.log("Test 2: Got a =", a);
    const b = await Promise.resolve(7);
    console.log("Test 2: Got b =", b);
    return a + b;
}

console.log("Main: Calling async functions");
const p1 = test1();
const p2 = test2();
console.log("Main: Both functions called");
console.log("Main: Script complete");
