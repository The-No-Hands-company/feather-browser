// Test yield* delegation (currently not fully supported)

function* inner() {
    yield 1;
    yield 2;
    yield 3;
}

function* outer() {
    console.log("Before yield*");
    yield* inner(); // Should delegate to inner generator
    console.log("After yield*");
    yield 4;
}

console.log("=== Testing yield* delegation ===");
const gen = outer();

console.log(gen.next()); // Should yield 1
console.log(gen.next()); // Should yield 2
console.log(gen.next()); // Should yield 3
console.log(gen.next()); // Should yield 4
console.log(gen.next()); // Should be done

console.log("\n=== Expected output ===");
console.log("Before yield*");
console.log("{ value: 1, done: false }");
console.log("{ value: 2, done: false }");
console.log("{ value: 3, done: false }");
console.log("After yield*");
console.log("{ value: 4, done: false }");
console.log("{ value: undefined, done: true }");
