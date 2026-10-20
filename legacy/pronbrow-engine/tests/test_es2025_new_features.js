// Test ES2023-2025 newer features

console.log("=== Testing Newer ES Features ===\n");

let passed = 0;
let failed = 0;

function test(name, fn) {
    try {
        fn();
        console.log(`✓ ${name}`);
        passed++;
        return true;
    } catch (e) {
        console.log(`✗ ${name}: ${e}`);
        failed++;
        return false;
    }
}

// ===== ES2023 FEATURES =====
console.log("--- ES2023 Features ---");

test("Array.prototype.findLast", () => {
    const arr = [1, 2, 3, 4];
    if (typeof arr.findLast !== 'function') throw "Method not found";
    const result = arr.findLast(x => x > 2);
    if (result !== 4) throw `Expected 4, got ${result}`;
});

test("Array.prototype.findLastIndex", () => {
    const arr = [1, 2, 3, 4];
    if (typeof arr.findLastIndex !== 'function') throw "Method not found";
    const result = arr.findLastIndex(x => x > 2);
    if (result !== 3) throw `Expected 3, got ${result}`;
});

test("Array.prototype.toSorted", () => {
    const arr = [3, 1, 2];
    if (typeof arr.toSorted !== 'function') throw "Method not found";
    const sorted = arr.toSorted();
    if (sorted[0] !== 1 || arr[0] !== 3) throw "toSorted failed";
});

test("Array.prototype.toReversed", () => {
    const arr = [1, 2, 3];
    if (typeof arr.toReversed !== 'function') throw "Method not found";
    const reversed = arr.toReversed();
    if (reversed[0] !== 3 || arr[0] !== 1) throw "toReversed failed";
});

test("Array.prototype.toSpliced", () => {
    const arr = [1, 2, 3];
    if (typeof arr.toSpliced !== 'function') throw "Method not found";
    const spliced = arr.toSpliced(1, 1, 99);
    if (spliced[1] !== 99 || arr[1] !== 2) throw "toSpliced failed";
});

test("Array.prototype.with", () => {
    const arr = [1, 2, 3];
    if (typeof arr.with !== 'function') throw "Method not found";
    const updated = arr.with(1, 99);
    if (updated[1] !== 99 || arr[1] !== 2) throw "with failed";
});

// ===== ES2024 FEATURES =====
console.log("\n--- ES2024 Features ---");

test("Object.groupBy", () => {
    if (typeof Object.groupBy !== 'function') throw "Method not found";
    const arr = [
        { type: 'a', value: 1 },
        { type: 'b', value: 2 },
        { type: 'a', value: 3 }
    ];
    const grouped = Object.groupBy(arr, x => x.type);
    if (grouped.a?.length !== 2) throw "groupBy failed";
});

test("Map.groupBy", () => {
    if (typeof Map.groupBy !== 'function') throw "Method not found";
    const arr = [1, 2, 3, 4];
    const grouped = Map.groupBy(arr, x => x % 2 === 0 ? 'even' : 'odd');
    if (grouped.get('even')?.length !== 2) throw "Map.groupBy failed";
});

test("Promise.withResolvers", () => {
    if (typeof Promise.withResolvers !== 'function') throw "Method not found";
    const { promise, resolve, reject } = Promise.withResolvers();
    if (typeof promise?.then !== 'function') throw "Not a Promise";
    if (typeof resolve !== 'function') throw "resolve not a function";
    if (typeof reject !== 'function') throw "reject not a function";
});

test("Atomics.waitAsync", () => {
    if (typeof Atomics?.waitAsync !== 'function') throw "Method not found";
    // Basic check - actual usage requires SharedArrayBuffer
});

// ===== ES2025 FEATURES =====
console.log("\n--- ES2025 Features ---");

test("String.prototype.isWellFormed", () => {
    if (typeof "test".isWellFormed !== 'function') throw "Method not found";
    if (!"test".isWellFormed()) throw "Should be well-formed";
});

test("String.prototype.toWellFormed", () => {
    if (typeof "test".toWellFormed !== 'function') throw "Method not found";
    const result = "test".toWellFormed();
    if (result !== "test") throw "toWellFormed failed";
});

test("ArrayBuffer.prototype.transfer", () => {
    const buffer = new ArrayBuffer(8);
    if (typeof buffer.transfer !== 'function') throw "Method not found";
    const transferred = buffer.transfer();
    if (!(transferred instanceof ArrayBuffer)) throw "transfer failed";
});

test("ArrayBuffer.prototype.transferToFixedLength", () => {
    const buffer = new ArrayBuffer(8);
    if (typeof buffer.transferToFixedLength !== 'function') throw "Method not found";
});

test("Set.prototype.difference", () => {
    const a = new Set([1, 2, 3]);
    if (typeof a.difference !== 'function') throw "Method not found";
    const b = new Set([2, 3, 4]);
    const diff = a.difference(b);
    if (!diff.has(1) || diff.has(2)) throw "difference failed";
});

test("Set.prototype.intersection", () => {
    const a = new Set([1, 2, 3]);
    if (typeof a.intersection !== 'function') throw "Method not found";
    const b = new Set([2, 3, 4]);
    const inter = a.intersection(b);
    if (!inter.has(2) || inter.has(1)) throw "intersection failed";
});

test("Set.prototype.union", () => {
    const a = new Set([1, 2]);
    if (typeof a.union !== 'function') throw "Method not found";
    const b = new Set([2, 3]);
    const uni = a.union(b);
    if (!uni.has(1) || !uni.has(3)) throw "union failed";
});

test("Set.prototype.symmetricDifference", () => {
    const a = new Set([1, 2]);
    if (typeof a.symmetricDifference !== 'function') throw "Method not found";
    const b = new Set([2, 3]);
    const symDiff = a.symmetricDifference(b);
    if (!symDiff.has(1) || !symDiff.has(3) || symDiff.has(2)) throw "symmetricDifference failed";
});

test("Set.prototype.isSubsetOf", () => {
    const a = new Set([1, 2]);
    if (typeof a.isSubsetOf !== 'function') throw "Method not found";
    const b = new Set([1, 2, 3]);
    if (!a.isSubsetOf(b)) throw "isSubsetOf failed";
});

test("Set.prototype.isSupersetOf", () => {
    const a = new Set([1, 2, 3]);
    if (typeof a.isSupersetOf !== 'function') throw "Method not found";
    const b = new Set([1, 2]);
    if (!a.isSupersetOf(b)) throw "isSupersetOf failed";
});

test("Set.prototype.isDisjointFrom", () => {
    const a = new Set([1, 2]);
    if (typeof a.isDisjointFrom !== 'function') throw "Method not found";
    const b = new Set([3, 4]);
    if (!a.isDisjointFrom(b)) throw "isDisjointFrom failed";
});

// ===== SUMMARY =====
console.log("\n=== RESULTS ===");
console.log(`Passed: ${passed}`);
console.log(`Failed: ${failed}`);
console.log(`Total:  ${passed + failed}`);
if (passed + failed > 0) {
    console.log(`Success Rate: ${((passed / (passed + failed)) * 100).toFixed(1)}%`);
}

console.log("\n=== ANALYSIS ===");
if (failed > 0) {
    console.log(`These ${failed} features are missing and should be implemented for full ES2025 compliance.`);
    console.log("Most are newer Array/Set/String prototype methods from ES2023-2025.");
} else {
    console.log("🎉 All ES2023-2025 features are implemented!");
}
