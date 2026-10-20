// Test multi-yield generator execution with PC-based resumption
console.log("=== Generator Multi-Yield Test ===\n");

// Test 1: Simple multi-yield
console.log("Test 1: Simple Multi-Yield");
function* simpleGen() {
    console.log("  Yielding 1");
    yield 1;
    console.log("  Yielding 2");
    yield 2;
    console.log("  Yielding 3");
    yield 3;
    console.log("  Generator done");
}

const g1 = simpleGen();
console.log("g1.next():", g1.next()); // Should log "Yielding 1" and return {value: 1, done: false}
console.log("g1.next():", g1.next()); // Should log "Yielding 2" and return {value: 2, done: false}
console.log("g1.next():", g1.next()); // Should log "Yielding 3" and return {value: 3, done: false}
console.log("g1.next():", g1.next()); // Should log "Generator done" and return {value: undefined, done: true}

console.log("\nTest 2: Generator with Computations");
function* computeGen() {
    console.log("  Computing first value");
    const a = 10;
    yield a;
    
    console.log("  Computing second value");
    const b = a * 2;
    yield b;
    
    console.log("  Computing final value");
    const c = a + b;
    yield c;
    
    console.log("  Returning sum");
    return a + b + c;
}

const g2 = computeGen();
console.log("g2.next():", g2.next()); // {value: 10, done: false}
console.log("g2.next():", g2.next()); // {value: 20, done: false}
console.log("g2.next():", g2.next()); // {value: 30, done: false}
console.log("g2.next():", g2.next()); // {value: 60, done: true}

console.log("\nTest 3: Generator with Loop");
function* loopGen() {
    for (let i = 0; i < 5; i++) {
        console.log("  Loop iteration", i);
        yield i;
    }
    console.log("  Loop complete");
}

const g3 = loopGen();
console.log("g3.next():", g3.next());
console.log("g3.next():", g3.next());
console.log("g3.next():", g3.next());
console.log("g3.next():", g3.next());
console.log("g3.next():", g3.next());
console.log("g3.next():", g3.next());

console.log("\n=== All Generator Tests Complete ===");
