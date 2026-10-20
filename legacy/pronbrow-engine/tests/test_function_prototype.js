// Test Function.prototype.call/apply/bind
console.log("=== Test 1: Function.prototype.call ===");

function greet(greeting, punctuation) {
    console.log(greeting + ", " + this.name + punctuation);
}

const person1 = { name: "Alice" };
const person2 = { name: "Bob" };

greet.call(person1, "Hello", "!");
greet.call(person2, "Hi", ".");

console.log("\n=== Test 2: Function.prototype.apply ===");

function sum(a, b, c) {
    return a + b + c;
}

const numbers = [1, 2, 3];
const result = sum.apply(null, numbers);
console.log("Sum:", result);

console.log("\n=== Test 3: Function.prototype.bind ===");

function multiply(a, b) {
    return a * b;
}

const double = multiply.bind(null, 2);
console.log("double(5):", double(5));
console.log("double(10):", double(10));

const triple = multiply.bind(null, 3);
console.log("triple(4):", triple(4));

console.log("\n=== Test 4: bind with 'this' ===");

const module = {
    x: 42,
    getX: function() {
        return this.x;
    }
};

const retrieveX = module.getX;
console.log("retrieveX():", retrieveX()); // Should be undefined

const boundGetX = retrieveX.bind(module);
console.log("boundGetX():", boundGetX()); // Should be 42

console.log("\n=== Test 5: Multiple arguments with bind ===");

function greetWith(greeting, greeting2, name) {
    console.log(greeting + " " + greeting2 + ", " + name + "!");
}

const sayHelloTo = greetWith.bind(null, "Hello", "there");
sayHelloTo("World");

console.log("\nAll Function.prototype tests complete!");
