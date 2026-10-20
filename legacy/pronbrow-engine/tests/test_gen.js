// Test generator PC-based resumption
console.log("=== Generator PC-Based Resumption Test ===");

// Test 1: Simple multiple yields
console.log("\n--- Test 1: Simple Multiple Yields ---");
function* simpleGen() {
    console.log("Generator started");
    yield 1;
    console.log("After yield 1");
    yield 2;
    console.log("After yield 2");
    yield 3;
    console.log("After yield 3");
    return "done";
}

const gen1 = simpleGen();
console.log("First next():");
const r1 = gen1.next();
console.log("Value:", r1.value, "Done:", r1.done);

console.log("\nSecond next():");
const r2 = gen1.next();
console.log("Value:", r2.value, "Done:", r2.done);

console.log("\nThird next():");
const r3 = gen1.next();
console.log("Value:", r3.value, "Done:", r3.done);

console.log("\nFourth next():");
const r4 = gen1.next();
console.log("Value:", r4.value, "Done:", r4.done);

// Test 2: Generator with local variables
console.log("\n--- Test 2: Local Variables ---");
function* counterGen() {
    let count = 0;
    yield ++count;
    yield ++count;
    yield ++count;
    return count;
}

const gen2 = counterGen();
console.log("gen2.next():", gen2.next().value);
console.log("gen2.next():", gen2.next().value);
console.log("gen2.next():", gen2.next().value);
console.log("gen2.next() [done]:", gen2.next().value);

// Test 3: Generator with expressions
console.log("\n--- Test 3: Expressions ---");
function* exprGen() {
    yield 10 + 5;
    yield 20 * 2;
    yield 30 / 3;
}

const gen3 = exprGen();
console.log("10 + 5 =", gen3.next().value);
console.log("20 * 2 =", gen3.next().value);
console.log("30 / 3 =", gen3.next().value);

console.log("\n=== All Tests Complete ===");
