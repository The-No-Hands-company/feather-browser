// Test Function.prototype.call/apply/bind
console.log("=== Function.prototype Methods Test ===\n");

// Test 1: Function.prototype.call
console.log("Test 1: Function.prototype.call");
function greet(greeting, punctuation) {
    console.log("  this.name:", this.name);
    return greeting + " " + this.name + punctuation;
}

const person1 = { name: "Alice" };
const person2 = { name: "Bob" };

console.log("  Direct call result:", greet.call(person1, "Hello", "!"));
console.log("  Second call result:", greet.call(person2, "Hi", "."));

// Test 2: Function.prototype.apply
console.log("\nTest 2: Function.prototype.apply");
function sum(a, b, c) {
    console.log("  this:", this);
    return a + b + c;
}

const numbers = [1, 2, 3];
const context = { name: "sumContext" };
console.log("  Apply result:", sum.apply(context, numbers));

// Test 3: Function.prototype.bind
console.log("\nTest 3: Function.prototype.bind");
function multiply(a, b) {
    return a * b;
}

const double = multiply.bind(null, 2);
console.log("  double(5):", double(5)); // Should be 10
console.log("  double(10):", double(10)); // Should be 20

// Test 4: Bind with this context
console.log("\nTest 4: Bind with 'this' context");
const obj = {
    value: 42,
    getValue: function() {
        return this.value;
    }
};

const boundGetValue = obj.getValue.bind(obj);
console.log("  Bound getValue:", boundGetValue()); // Should be 42

// Test 5: Partial application with bind
console.log("\nTest 5: Partial application");
function add(a, b, c) {
    return a + b + c;
}

const add5 = add.bind(null, 5);
const add5And10 = add5.bind(null, 10);
console.log("  add5And10(3):", add5And10(3)); // Should be 5 + 10 + 3 = 18

console.log("\n=== All Function.prototype Tests Complete ===");
