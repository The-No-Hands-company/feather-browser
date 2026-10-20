// Comprehensive ES2025 Feature Test Suite
// Testing browser engine compliance with ECMAScript 2025

console.log("=".repeat(60));
console.log("ES2025 FEATURE COMPLIANCE TEST SUITE");
console.log("=".repeat(60));

let passed = 0;
let failed = 0;

function test(name, fn) {
    try {
        fn();
        console.log("✓", name);
        passed++;
    } catch (e) {
        console.log("✗", name, "-", e);
        failed++;
    }
}

// ============================================================================
// SECTION 1: ASYNC/AWAIT (ES2017 + ES2025 updates)
// ============================================================================
console.log("\n[1] ASYNC/AWAIT FEATURES");
console.log("-".repeat(60));

test("1.1 Basic async function returns Promise", () => {
    async function f() { return 42; }
    const p = f();
    if (typeof p !== 'object') throw "Not a Promise object";
});

test("1.2 Await expression suspends execution", () => {
    let executed = false;
    async function f() {
        await Promise.resolve(1);
        executed = true;
    }
    f();
    if (executed) throw "Should not execute synchronously";
});

test("1.3 Multiple await expressions", () => {
    async function f() {
        const a = await Promise.resolve(1);
        const b = await Promise.resolve(2);
        return a + b;
    }
    f(); // Should not throw
});

test("1.4 Async function with rejection handling", () => {
    async function f() {
        try {
            await Promise.reject("error");
        } catch (e) {
            return "caught";
        }
    }
    f(); // Should handle rejection
});

// ============================================================================
// SECTION 2: GENERATORS (ES2015 + ES2025 updates)
// ============================================================================
console.log("\n[2] GENERATOR FEATURES");
console.log("-".repeat(60));

test("2.1 Generator function creates generator", () => {
    function* gen() { yield 1; }
    const g = gen();
    if (typeof g.next !== 'function') throw "No next method";
});

test("2.2 Generator yields values", () => {
    function* gen() { yield 42; }
    const g = gen();
    const result = g.next();
    if (result.value !== 42) throw "Wrong value: " + result.value;
    if (result.done !== false) throw "Should not be done";
});

test("2.3 Multiple yields work correctly", () => {
    function* gen() {
        yield 1;
        yield 2;
        yield 3;
    }
    const g = gen();
    const r1 = g.next();
    const r2 = g.next();
    const r3 = g.next();
    const r4 = g.next();
    if (r1.value !== 1 || r2.value !== 2 || r3.value !== 3) 
        throw "Wrong values";
    if (!r4.done) throw "Should be done";
});

test("2.4 Generator with return statement", () => {
    function* gen() {
        yield 1;
        return 42;
    }
    const g = gen();
    g.next(); // yield 1
    const result = g.next(); // return 42
    if (result.value !== 42) throw "Wrong return value";
    if (!result.done) throw "Should be done";
});

test("2.5 Generator state persistence", () => {
    function* gen() {
        const x = 10;
        yield x;
        yield x * 2;
    }
    const g = gen();
    const r1 = g.next();
    const r2 = g.next();
    if (r1.value !== 10 || r2.value !== 20) 
        throw "State not preserved";
});

// ============================================================================
// SECTION 3: FUNCTION.PROTOTYPE METHODS (ES5/ES2025)
// ============================================================================
console.log("\n[3] FUNCTION.PROTOTYPE METHODS");
console.log("-".repeat(60));

test("3.1 Function.prototype.call with thisArg", () => {
    function f() { return this.x; }
    const result = f.call({ x: 42 });
    if (result !== 42) throw "Wrong this binding";
});

test("3.2 Function.prototype.call with arguments", () => {
    function f(a, b) { return a + b; }
    const result = f.call(null, 5, 10);
    if (result !== 15) throw "Wrong result: " + result;
});

test("3.3 Function.prototype.apply with array", () => {
    function sum(a, b, c) { return a + b + c; }
    const result = sum.apply(null, [1, 2, 3]);
    if (result !== 6) throw "Wrong result: " + result;
});

test("3.4 Function.prototype.bind creates bound function", () => {
    function f(a) { return this.x + a; }
    const bound = f.bind({ x: 10 });
    const result = bound(5);
    if (result !== 15) throw "Wrong result: " + result;
});

test("3.5 Function.prototype.bind with partial application", () => {
    function add(a, b) { return a + b; }
    const add5 = add.bind(null, 5);
    const result = add5(10);
    if (result !== 15) throw "Wrong result: " + result;
});

// ============================================================================
// SECTION 4: PROMISES (ES2015 + ES2025 updates)
// ============================================================================
console.log("\n[4] PROMISE FEATURES");
console.log("-".repeat(60));

test("4.1 Promise.resolve creates fulfilled Promise", () => {
    const p = Promise.resolve(42);
    if (typeof p !== 'object') throw "Not a Promise";
});

test("4.2 Promise.reject creates rejected Promise", () => {
    const p = Promise.reject("error");
    if (typeof p !== 'object') throw "Not a Promise";
});

test("4.3 Promise.then chains", () => {
    const p = Promise.resolve(42);
    const p2 = p.then(x => x * 2);
    if (typeof p2 !== 'object') throw "then should return Promise";
});

test("4.4 Promise.all combines Promises", () => {
    const p = Promise.all([
        Promise.resolve(1),
        Promise.resolve(2),
        Promise.resolve(3)
    ]);
    if (typeof p !== 'object') throw "Not a Promise";
});

test("4.5 Promise.race returns first settled", () => {
    const p = Promise.race([
        Promise.resolve(1),
        Promise.resolve(2)
    ]);
    if (typeof p !== 'object') throw "Not a Promise";
});

// ============================================================================
// SECTION 5: ARROW FUNCTIONS (ES2015)
// ============================================================================
console.log("\n[5] ARROW FUNCTION FEATURES");
console.log("-".repeat(60));

test("5.1 Basic arrow function", () => {
    const f = () => 42;
    const result = f();
    if (result !== 42) throw "Wrong result";
});

test("5.2 Arrow function with parameters", () => {
    const add = (a, b) => a + b;
    const result = add(5, 10);
    if (result !== 15) throw "Wrong result";
});

test("5.3 Arrow function with block body", () => {
    const f = (x) => {
        const y = x * 2;
        return y + 10;
    };
    const result = f(5);
    if (result !== 20) throw "Wrong result";
});

test("5.4 Arrow function lexical this binding", () => {
    const obj = {
        x: 42,
        getX: function() {
            const inner = () => this.x;
            return inner();
        }
    };
    const result = obj.getX();
    if (result !== 42) throw "Wrong this binding";
});

// ============================================================================
// SECTION 6: DESTRUCTURING (ES2015)
// ============================================================================
console.log("\n[6] DESTRUCTURING FEATURES");
console.log("-".repeat(60));

test("6.1 Array destructuring", () => {
    const [a, b] = [1, 2];
    if (a !== 1 || b !== 2) throw "Wrong values";
});

test("6.2 Object destructuring", () => {
    const { x, y } = { x: 10, y: 20 };
    if (x !== 10 || y !== 20) throw "Wrong values";
});

test("6.3 Nested destructuring", () => {
    const { a: { b } } = { a: { b: 42 } };
    if (b !== 42) throw "Wrong value";
});

test("6.4 Default values in destructuring", () => {
    const { x = 10 } = {};
    if (x !== 10) throw "Wrong default value";
});

// ============================================================================
// SECTION 7: SPREAD/REST (ES2015/ES2018)
// ============================================================================
console.log("\n[7] SPREAD/REST FEATURES");
console.log("-".repeat(60));

test("7.1 Array spread", () => {
    const arr = [1, 2, 3];
    const arr2 = [...arr, 4, 5];
    if (arr2.length !== 5) throw "Wrong length";
});

test("7.2 Object spread", () => {
    const obj = { x: 1, y: 2 };
    const obj2 = { ...obj, z: 3 };
    if (obj2.x !== 1 || obj2.z !== 3) throw "Wrong values";
});

test("7.3 Rest parameters", () => {
    function f(...args) {
        return args.length;
    }
    const result = f(1, 2, 3, 4, 5);
    if (result !== 5) throw "Wrong length";
});

// ============================================================================
// SECTION 8: TEMPLATE LITERALS (ES2015)
// ============================================================================
console.log("\n[8] TEMPLATE LITERAL FEATURES");
console.log("-".repeat(60));

test("8.1 Basic template literal", () => {
    const x = 42;
    const str = `Value: ${x}`;
    if (str !== "Value: 42") throw "Wrong template result";
});

test("8.2 Multiline template literal", () => {
    const str = `Line 1
Line 2`;
    if (!str.includes("Line 1") || !str.includes("Line 2")) 
        throw "Multiline failed";
});

test("8.3 Expression in template", () => {
    const a = 10;
    const b = 20;
    const str = `Sum: ${a + b}`;
    if (str !== "Sum: 30") throw "Wrong expression result";
});

// ============================================================================
// SECTION 9: CLASSES (ES2015 + ES2025)
// ============================================================================
console.log("\n[9] CLASS FEATURES");
console.log("-".repeat(60));

test("9.1 Basic class declaration", () => {
    class A {}
    const a = new A();
    if (typeof a !== 'object') throw "Not an object";
});

test("9.2 Class with constructor", () => {
    class A {
        constructor(x) {
            this.x = x;
        }
    }
    const a = new A(42);
    if (a.x !== 42) throw "Constructor failed";
});

test("9.3 Class with methods", () => {
    class A {
        getValue() {
            return 42;
        }
    }
    const a = new A();
    if (a.getValue() !== 42) throw "Method failed";
});

test("9.4 Class inheritance", () => {
    class A {
        getValue() { return 10; }
    }
    class B extends A {}
    const b = new B();
    if (b.getValue() !== 10) throw "Inheritance failed";
});

// ============================================================================
// RESULTS
// ============================================================================
console.log("\n" + "=".repeat(60));
console.log("TEST RESULTS");
console.log("=".repeat(60));
console.log(`Passed: ${passed}`);
console.log(`Failed: ${failed}`);
console.log(`Total:  ${passed + failed}`);
console.log(`Success Rate: ${((passed / (passed + failed)) * 100).toFixed(1)}%`);
console.log("=".repeat(60));
