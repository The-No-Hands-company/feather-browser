// Test ES2021 Logical Assignment Operators
console.log("=== ES2021 Logical Assignment Operators Test ===\n");

// Test 1: ||= (Logical OR Assignment)
console.log("Test 1: ||= (Logical OR Assignment)");
let a = 0;
let b = 5;
let c = null;
let d = "hello";

console.log("Before: a =", a, "b =", b, "c =", c, "d =", d);

a ||= 10;
b ||= 20;
c ||= "default";
d ||= "world";

console.log("After ||=:");
console.log("  a =", a, "(0 is falsy, so assigned 10)");
console.log("  b =", b, "(5 is truthy, kept as 5)");
console.log("  c =", c, "(null is falsy, so assigned 'default')");
console.log("  d =", d, "('hello' is truthy, kept as 'hello')");

// Test 2: &&= (Logical AND Assignment)
console.log("\nTest 2: &&= (Logical AND Assignment)");
let x = 0;
let y = 5;
let z = "test";

console.log("Before: x =", x, "y =", y, "z =", z);

x &&= 10;
y &&= 20;
z &&= "modified";

console.log("After &&=:");
console.log("  x =", x, "(0 is falsy, kept as 0)");
console.log("  y =", y, "(5 is truthy, assigned 20)");
console.log("  z =", z, "('test' is truthy, assigned 'modified')");

// Test 3: ??= (Nullish Coalescing Assignment)
console.log("\nTest 3: ??= (Nullish Coalescing Assignment)");
let p = null;
let q = undefined;
let r = 0;
let s = "";
let t = false;

console.log("Before: p =", p, "q =", q, "r =", r, "s =", s, "t =", t);

p ??= "assigned";
q ??= "assigned";
r ??= 100;
s ??= "assigned";
t ??= true;

console.log("After ??=:");
console.log("  p =", p, "(null → 'assigned')");
console.log("  q =", q, "(undefined → 'assigned')");
console.log("  r =", r, "(0 is not nullish, kept as 0)");
console.log("  s =", s, "(empty string is not nullish, kept as '')");
console.log("  t =", t, "(false is not nullish, kept as false)");

// Test 4: Comparison of all three
console.log("\nTest 4: Comparison of all three operators");
function testAll(initialValue) {
    let or = initialValue;
    let and = initialValue;
    let nullish = initialValue;
    
    or ||= "OR";
    and &&= "AND";
    nullish ??= "NULLISH";
    
    console.log("  Initial:", initialValue);
    console.log("    ||=:", or);
    console.log("    &&=:", and);
    console.log("    ??=:", nullish);
}

console.log("\nWith null:");
testAll(null);

console.log("\nWith 0:");
testAll(0);

console.log("\nWith false:");
testAll(false);

console.log("\nWith '':");
testAll("");

console.log("\nWith 'value':");
testAll("value");

// Test 5: Complex expressions
console.log("\nTest 5: Complex expressions");
const obj = {
    a: null,
    b: 0,
    c: "existing"
};

obj.a ??= "new value";
obj.b ||= 42;
obj.c &&= "modified";

console.log("Object after assignments:");
console.log("  obj.a =", obj.a, "(null → 'new value' with ??=)");
console.log("  obj.b =", obj.b, "(0 → 42 with ||=)");
console.log("  obj.c =", obj.c, "('existing' → 'modified' with &&=)");

console.log("\n=== All Logical Assignment Tests Complete ===");
