// Comprehensive ES2025 JavaScript Engine Test

console.log("=== ES2025 Feature Compliance Test ===\n");

let passed = 0;
let failed = 0;

function test(name, fn) {
    try {
        fn();
        console.log(`✓ ${name}`);
        passed++;
    } catch (e) {
        console.log(`✗ ${name}: ${e}`);
        failed++;
    }
}

// ===== GENERATORS =====
console.log("--- Generators ---");
test("Basic generator", () => {
    function* gen() { yield 1; yield 2; }
    const g = gen();
    if (g.next().value !== 1) throw "Wrong value";
    if (g.next().value !== 2) throw "Wrong value";
});

test("Generator with return", () => {
    function* gen() { yield 1; return 42; }
    const g = gen();
    g.next();
    const r = g.next();
    if (r.value !== 42 || !r.done) throw "Wrong return";
});

test("yield* delegation", () => {
    function* inner() { yield 1; yield 2; }
    function* outer() { yield* inner(); }
    const g = outer();
    if (g.next().value !== 1) throw "Wrong delegation";
    if (g.next().value !== 2) throw "Wrong delegation";
});

// ===== ASYNC/AWAIT =====
console.log("\n--- Async/Await ---");
test("Async function returns Promise", () => {
    async function f() { return 42; }
    const p = f();
    if (typeof p?.then !== 'function') throw "Not a Promise";
});

test("Await expression", () => {
    async function f() {
        const x = await Promise.resolve(42);
        return x;
    }
    // Just test it doesn't throw
    f();
});

// ===== FUNCTION.PROTOTYPE =====
console.log("\n--- Function.prototype ---");
test("Function.prototype.call", () => {
    function f() { return this.x; }
    const result = f.call({ x: 42 });
    if (result !== 42) throw "call failed";
});

test("Function.prototype.apply", () => {
    function sum(a, b) { return a + b; }
    const result = sum.apply(null, [10, 20]);
    if (result !== 30) throw "apply failed";
});

test("Function.prototype.bind", () => {
    function f(a, b) { return this.x + a + b; }
    const bound = f.bind({ x: 1 }, 10);
    const result = bound(20);
    if (result !== 31) throw "bind failed";
});

// ===== MODERN OPERATORS =====
console.log("\n--- Modern Operators ---");
test("Optional chaining (?.)​", () => {
    const obj = { a: { b: 42 } };
    if (obj?.a?.b !== 42) throw "Failed";
    if (obj?.x?.y !== undefined) throw "Failed";
});

test("Nullish coalescing (??)", () => {
    if ((null ?? 42) !== 42) throw "Failed";
    if ((0 ?? 42) !== 0) throw "Failed";
});

test("Logical assignment (||=)", () => {
    let x = null;
    x ||= 42;
    if (x !== 42) throw "Failed";
});

test("Logical assignment (&&=)", () => {
    let x = 10;
    x &&= 20;
    if (x !== 20) throw "Failed";
});

test("Logical assignment (??=)", () => {
    let x = null;
    x ??= 42;
    if (x !== 42) throw "Failed";
});

// ===== DESTRUCTURING =====
console.log("\n--- Destructuring ---");
test("Array destructuring", () => {
    const [a, b] = [1, 2];
    if (a !== 1 || b !== 2) throw "Failed";
});

test("Object destructuring", () => {
    const {x, y} = {x: 1, y: 2};
    if (x !== 1 || y !== 2) throw "Failed";
});

test("Rest parameters", () => {
    function f(...args) { return args.length; }
    if (f(1, 2, 3) !== 3) throw "Failed";
});

// ===== ARROW FUNCTIONS =====
console.log("\n--- Arrow Functions ---");
test("Arrow function expression", () => {
    const f = (x) => x * 2;
    if (f(21) !== 42) throw "Failed";
});

test("Arrow function implicit return", () => {
    const f = (x) => x + 1;
    if (f(41) !== 42) throw "Failed";
});

// ===== CLASSES =====
console.log("\n--- Classes ---");
test("Class declaration", () => {
    class C { method() { return 42; } }
    const c = new C();
    if (c.method() !== 42) throw "Failed";
});

test("Class inheritance", () => {
    class A { m() { return 1; } }
    class B extends A { m() { return super.m() + 1; } }
    const b = new B();
    if (b.m() !== 2) throw "Failed";
});

test("Class constructor", () => {
    class C { constructor(x) { this.x = x; } }
    const c = new C(42);
    if (c.x !== 42) throw "Failed";
});

// ===== TEMPLATE LITERALS =====
console.log("\n--- Template Literals ---");
test("Template literal", () => {
    const x = 42;
    const s = `value: ${x}`;
    if (s !== "value: 42") throw "Failed";
});

// ===== SPREAD =====
console.log("\n--- Spread Operator ---");
test("Array spread", () => {
    const a = [1, 2];
    const b = [...a, 3];
    if (b.length !== 3 || b[2] !== 3) throw "Failed";
});

test("Object spread", () => {
    const a = {x: 1};
    const b = {...a, y: 2};
    if (b.x !== 1 || b.y !== 2) throw "Failed";
});

// ===== PROMISES =====
console.log("\n--- Promises ---");
test("Promise.resolve", () => {
    const p = Promise.resolve(42);
    if (typeof p?.then !== 'function') throw "Not a Promise";
});

test("Promise.reject", () => {
    const p = Promise.reject(new Error("test"));
    if (typeof p?.catch !== 'function') throw "Not a Promise";
});

test("Promise.then", () => {
    Promise.resolve(42).then((x) => {
        if (x !== 42) throw "Wrong value";
    });
});

test("Promise.all", () => {
    const p = Promise.all([Promise.resolve(1), Promise.resolve(2)]);
    if (typeof p?.then !== 'function') throw "Not a Promise";
});

test("Promise.race", () => {
    const p = Promise.race([Promise.resolve(1), Promise.resolve(2)]);
    if (typeof p?.then !== 'function') throw "Not a Promise";
});

test("Promise.allSettled", () => {
    const p = Promise.allSettled([Promise.resolve(1), Promise.reject(2)]);
    if (typeof p?.then !== 'function') throw "Not a Promise";
});

test("Promise.any", () => {
    const p = Promise.any([Promise.reject(1), Promise.resolve(2)]);
    if (typeof p?.then !== 'function') throw "Not a Promise";
});

// ===== ARRAY METHODS =====
console.log("\n--- Array Methods ---");
test("Array.prototype.map", () => {
    const a = [1, 2, 3].map(x => x * 2);
    if (a[0] !== 2 || a[1] !== 4) throw "Failed";
});

test("Array.prototype.filter", () => {
    const a = [1, 2, 3, 4].filter(x => x > 2);
    if (a.length !== 2 || a[0] !== 3) throw "Failed";
});

test("Array.prototype.reduce", () => {
    const sum = [1, 2, 3].reduce((a, b) => a + b, 0);
    if (sum !== 6) throw "Failed";
});

// ===== SUMMARY =====
console.log("\n=== RESULTS ===");
console.log(`Passed: ${passed}`);
console.log(`Failed: ${failed}`);
console.log(`Total:  ${passed + failed}`);
console.log(`Success Rate: ${((passed / (passed + failed)) * 100).toFixed(1)}%`);

if (failed === 0) {
    console.log("\n🎉 ALL TESTS PASSED! 🎉");
} else {
    console.log(`\n⚠️  ${failed} test(s) failed`);
}
