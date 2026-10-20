// Test advanced generator scenarios

// Test 1: Generator with loop
function* count(n) {
    for (let i = 0; i < n; i++) {
        console.log(`Yielding ${i}`);
        yield i;
    }
    console.log("Count complete");
}

console.log("=== Test 1: Generator with loop ===");
const counter = count(3);
console.log(counter.next()); // 0
console.log(counter.next()); // 1
console.log(counter.next()); // 2
console.log(counter.next()); // done

// Test 2: Generator with conditionals
function* conditional() {
    console.log("Start");
    const x = 10;
    if (x > 5) {
        console.log("x is large");
        yield "large";
    } else {
        console.log("x is small");
        yield "small";
    }
    console.log("After if");
    yield "done";
}

console.log("\n=== Test 2: Generator with conditional ===");
const cond = conditional();
console.log(cond.next());
console.log(cond.next());
console.log(cond.next());

// Test 3: Generator with return
function* withReturn() {
    yield 1;
    yield 2;
    return 42; // Should set value on final next()
    yield 3; // Should never execute
}

console.log("\n=== Test 3: Generator with return ===");
const ret = withReturn();
console.log(ret.next()); // {value: 1, done: false}
console.log(ret.next()); // {value: 2, done: false}
console.log(ret.next()); // {value: 42, done: true}
console.log(ret.next()); // {value: undefined, done: true}

console.log("\n=== ALL TESTS COMPLETE ===");
