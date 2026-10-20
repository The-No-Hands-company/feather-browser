// Test ES2020 Optional Chaining and Nullish Coalescing Operators
console.log("=== ES2020 Modern Operators Test ===\n");

// Test 1: Optional Chaining with Objects
console.log("Test 1: Optional Chaining with Objects");
const obj = {
    name: "Alice",
    address: {
        city: "New York",
        zip: 10001
    },
    getGreeting: function() {
        return "Hello!";
    }
};

console.log("obj.name:", obj.name);
console.log("obj.address.city:", obj.address.city);
console.log("obj.address?.city:", obj.address?.city);
console.log("obj.missing?.property:", obj.missing?.property);  // Should be undefined

// Test 2: Optional Chaining with null/undefined
console.log("\nTest 2: Optional Chaining with null/undefined");
const nullObj = null;
const undefinedObj = undefined;

console.log("nullObj?.property:", nullObj?.property);  // Should be undefined
console.log("undefinedObj?.property:", undefinedObj?.property);  // Should be undefined

// Test 3: Optional Call Expression
console.log("\nTest 3: Optional Call Expression");
const objWithMethod = {
    greet: function() {
        return "Hello from method!";
    }
};

console.log("objWithMethod.greet():", objWithMethod.greet());
console.log("objWithMethod.greet?.():", objWithMethod.greet?.());
console.log("objWithMethod.missing?.():", objWithMethod.missing?.());  // Should be undefined

const nullFunc = null;
console.log("nullFunc?.():", nullFunc?.());  // Should be undefined

// Test 4: Nullish Coalescing Operator
console.log("\nTest 4: Nullish Coalescing Operator");
const value1 = null;
const value2 = undefined;
const value3 = 0;
const value4 = "";
const value5 = false;

console.log("null ?? 'default':", value1 ?? 'default');  // 'default'
console.log("undefined ?? 'default':", value2 ?? 'default');  // 'default'
console.log("0 ?? 'default':", value3 ?? 'default');  // 0 (not 'default'!)
console.log("'' ?? 'default':", value4 ?? 'default');  // '' (not 'default'!)
console.log("false ?? 'default':", value5 ?? 'default');  // false (not 'default'!)

// Test 5: Combined Optional Chaining and Nullish Coalescing
console.log("\nTest 5: Combined Optional Chaining and Nullish Coalescing");
const user = {
    name: "Bob",
    settings: null
};

console.log("user.settings?.theme ?? 'dark':", user.settings?.theme ?? 'dark');
console.log("user.settings?.notifications ?? true:", user.settings?.notifications ?? true);
console.log("user.name ?? 'Anonymous':", user.name ?? 'Anonymous');

// Test 6: Deep Optional Chaining
console.log("\nTest 6: Deep Optional Chaining");
const data = {
    user: {
        profile: {
            email: "test@example.com"
        }
    }
};

console.log("data.user?.profile?.email:", data.user?.profile?.email);
console.log("data.user?.settings?.theme:", data.user?.settings?.theme);  // undefined
console.log("data.missing?.deep?.property:", data.missing?.deep?.property);  // undefined

// Test 7: Optional Chaining with Computed Properties
console.log("\nTest 7: Optional Chaining with Computed Properties");
const arr = [1, 2, 3];
const index = 1;

console.log("arr?.[index]:", arr?.[index]);  // 2
console.log("arr?.[10]:", arr?.[10]);  // undefined

const nullArray = null;
console.log("nullArray?.[0]:", nullArray?.[0]);  // undefined

console.log("\n=== All Modern Operators Tests Complete ===");
