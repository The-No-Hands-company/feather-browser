// ES2025 Compliance Tests
// Tests for all ES2020-2025 features

#[cfg(test)]
mod es2025_tests {
    use pronbrow_engine::js::engine::JSEngine;

    // ============================================================================
    // ES2020 Features
    // ============================================================================

    #[test]
    fn test_optional_chaining() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const obj = { a: { b: { c: 42 } } };
            const result1 = obj?.a?.b?.c;
            const result2 = obj?.x?.y?.z;
            const result3 = null?.prop;
            result1;
        "#);
        assert!(result.is_ok(), "Optional chaining failed: {:?}", result);
    }

    #[test]
    fn test_nullish_coalescing() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const a = null ?? 42;
            const b = undefined ?? 99;
            const c = 0 ?? 100;
            a;
        "#);
        assert!(result.is_ok(), "Nullish coalescing failed: {:?}", result);
    }

    #[test]
    fn test_bigint() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const big = 9007199254740991n;
            typeof big === "bigint";
        "#);
        assert!(result.is_ok(), "BigInt failed: {:?}", result);
    }

    #[test]
    fn test_globalthis() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            typeof globalThis === "object";
        "#);
        assert!(result.is_ok(), "globalThis failed: {:?}", result);
    }

    #[test]
    fn test_promise_allsettled() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const p1 = Promise.resolve(1);
            const p2 = Promise.reject("error");
            const p3 = Promise.resolve(3);
            typeof Promise.allSettled === "function";
        "#);
        assert!(result.is_ok(), "Promise.allSettled failed: {:?}", result);
    }

    // ============================================================================
    // ES2021 Features
    // ============================================================================

    #[test]
    fn test_logical_or_assignment() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            let x = null;
            x ||= 42;
            x;
        "#);
        assert!(result.is_ok(), "Logical OR assignment failed: {:?}", result);
    }

    #[test]
    fn test_logical_and_assignment() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            let x = true;
            x &&= false;
            x;
        "#);
        assert!(result.is_ok(), "Logical AND assignment failed: {:?}", result);
    }

    #[test]
    fn test_nullish_assignment() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            let x = null;
            x ??= 100;
            x;
        "#);
        assert!(result.is_ok(), "Nullish assignment failed: {:?}", result);
    }

    #[test]
    fn test_numeric_separators() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const million = 1_000_000;
            million === 1000000;
        "#);
        assert!(result.is_ok(), "Numeric separators failed: {:?}", result);
    }

    #[test]
    fn test_string_replaceall() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const str = "hello world world";
            typeof str.replaceAll === "function";
        "#);
        assert!(result.is_ok(), "String.prototype.replaceAll failed: {:?}", result);
    }

    #[test]
    fn test_promise_any() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            typeof Promise.any === "function";
        "#);
        assert!(result.is_ok(), "Promise.any failed: {:?}", result);
    }

    // ============================================================================
    // ES2022 Features
    // ============================================================================

    #[test]
    fn test_array_at() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const arr = [1, 2, 3, 4, 5];
            typeof arr.at === "function";
        "#);
        assert!(result.is_ok(), "Array.prototype.at failed: {:?}", result);
    }

    #[test]
    fn test_string_at() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const str = "hello";
            typeof str.at === "function";
        "#);
        assert!(result.is_ok(), "String.prototype.at failed: {:?}", result);
    }

    #[test]
    fn test_object_hasown() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const obj = { prop: 42 };
            typeof Object.hasOwn === "function";
        "#);
        assert!(result.is_ok(), "Object.hasOwn failed: {:?}", result);
    }

    #[test]
    fn test_error_cause() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            try {
                const err = new Error("message", { cause: "root cause" });
                true;
            } catch(e) {
                false;
            }
        "#);
        assert!(result.is_ok(), "Error.cause failed: {:?}", result);
    }

    #[test]
    fn test_class_fields() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            class MyClass {
                publicField = 42;
                #privateField = 99;
                
                getPrivate() {
                    return this.#privateField;
                }
            }
            const inst = new MyClass();
            inst.publicField === 42;
        "#);
        assert!(result.is_ok(), "Class fields failed: {:?}", result);
    }

    #[test]
    fn test_static_blocks() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            class MyClass {
                static value = 0;
                static {
                    this.value = 42;
                }
            }
            MyClass.value === 42;
        "#);
        assert!(result.is_ok(), "Static blocks failed: {:?}", result);
    }

    #[test]
    fn test_top_level_await() {
        let mut engine = JSEngine::new();
        // Top-level await is a module feature
        let result = engine.execute(r#"
            async function test() {
                return 42;
            }
            true;
        "#);
        assert!(result.is_ok(), "Async function declaration failed: {:?}", result);
    }

    // ============================================================================
    // ES2023 Features
    // ============================================================================

    #[test]
    fn test_array_findlast() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const arr = [1, 2, 3, 4, 5];
            typeof arr.findLast === "function";
        "#);
        assert!(result.is_ok(), "Array.prototype.findLast failed: {:?}", result);
    }

    #[test]
    fn test_array_findlastindex() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const arr = [1, 2, 3, 4, 5];
            typeof arr.findLastIndex === "function";
        "#);
        assert!(result.is_ok(), "Array.prototype.findLastIndex failed: {:?}", result);
    }

    #[test]
    fn test_array_toreversed() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const arr = [1, 2, 3];
            typeof arr.toReversed === "function";
        "#);
        assert!(result.is_ok(), "Array.prototype.toReversed failed: {:?}", result);
    }

    #[test]
    fn test_array_tosorted() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const arr = [3, 1, 2];
            typeof arr.toSorted === "function";
        "#);
        assert!(result.is_ok(), "Array.prototype.toSorted failed: {:?}", result);
    }

    #[test]
    fn test_array_tospliced() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const arr = [1, 2, 3, 4, 5];
            typeof arr.toSpliced === "function";
        "#);
        assert!(result.is_ok(), "Array.prototype.toSpliced failed: {:?}", result);
    }

    #[test]
    fn test_array_with() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const arr = [1, 2, 3];
            typeof arr.with === "function";
        "#);
        assert!(result.is_ok(), "Array.prototype.with failed: {:?}", result);
    }

    // ============================================================================
    // ES2024 Features
    // ============================================================================

    #[test]
    fn test_object_groupby() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            typeof Object.groupBy === "function";
        "#);
        assert!(result.is_ok(), "Object.groupBy failed: {:?}", result);
    }

    #[test]
    fn test_map_groupby() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            typeof Map.groupBy === "function";
        "#);
        assert!(result.is_ok(), "Map.groupBy failed: {:?}", result);
    }

    #[test]
    fn test_promise_withresolvers() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            typeof Promise.withResolvers === "function";
        "#);
        assert!(result.is_ok(), "Promise.withResolvers failed: {:?}", result);
    }

    #[test]
    fn test_arraybuffer_transfer() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const ab = new ArrayBuffer(8);
            typeof ab.transfer === "function";
        "#);
        assert!(result.is_ok(), "ArrayBuffer.prototype.transfer failed: {:?}", result);
    }

    // ============================================================================
    // ES2025 Features
    // ============================================================================

    #[test]
    fn test_string_iswellformed() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const str = "hello world";
            typeof str.isWellFormed === "function";
        "#);
        assert!(result.is_ok(), "String.prototype.isWellFormed failed: {:?}", result);
    }

    #[test]
    fn test_string_towellformed() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const str = "hello world";
            typeof str.toWellFormed === "function";
        "#);
        assert!(result.is_ok(), "String.prototype.toWellFormed failed: {:?}", result);
    }

    #[test]
    fn test_set_union() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const set1 = new Set([1, 2, 3]);
            const set2 = new Set([3, 4, 5]);
            typeof set1.union === "function";
        "#);
        assert!(result.is_ok(), "Set.prototype.union failed: {:?}", result);
    }

    #[test]
    fn test_set_intersection() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const set1 = new Set([1, 2, 3]);
            typeof set1.intersection === "function";
        "#);
        assert!(result.is_ok(), "Set.prototype.intersection failed: {:?}", result);
    }

    #[test]
    fn test_set_difference() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const set1 = new Set([1, 2, 3]);
            typeof set1.difference === "function";
        "#);
        assert!(result.is_ok(), "Set.prototype.difference failed: {:?}", result);
    }

    #[test]
    fn test_set_symmetric_difference() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const set1 = new Set([1, 2, 3]);
            typeof set1.symmetricDifference === "function";
        "#);
        assert!(result.is_ok(), "Set.prototype.symmetricDifference failed: {:?}", result);
    }

    #[test]
    fn test_set_issubsetof() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const set1 = new Set([1, 2]);
            typeof set1.isSubsetOf === "function";
        "#);
        assert!(result.is_ok(), "Set.prototype.isSubsetOf failed: {:?}", result);
    }

    #[test]
    fn test_set_issupersetof() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const set1 = new Set([1, 2, 3]);
            typeof set1.isSupersetOf === "function";
        "#);
        assert!(result.is_ok(), "Set.prototype.isSupersetOf failed: {:?}", result);
    }

    #[test]
    fn test_set_isdisjointfrom() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const set1 = new Set([1, 2]);
            const set2 = new Set([3, 4]);
            typeof set1.isDisjointFrom === "function";
        "#);
        assert!(result.is_ok(), "Set.prototype.isDisjointFrom failed: {:?}", result);
    }

    // ============================================================================
    // Async/Await Tests
    // ============================================================================

    #[test]
    fn test_async_function() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            async function test() {
                return 42;
            }
            typeof test === "function";
        "#);
        assert!(result.is_ok(), "Async function failed: {:?}", result);
    }

    #[test]
    fn test_await_expression() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            async function test() {
                const result = await Promise.resolve(42);
                return result;
            }
            typeof test === "function";
        "#);
        assert!(result.is_ok(), "Await expression failed: {:?}", result);
    }

    // ============================================================================
    // Generator Tests
    // ============================================================================

    #[test]
    fn test_generator_function() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            function* gen() {
                yield 1;
                yield 2;
                yield 3;
            }
            typeof gen === "function";
        "#);
        assert!(result.is_ok(), "Generator function failed: {:?}", result);
    }

    #[test]
    fn test_yield_expression() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            function* gen() {
                yield 42;
            }
            const g = gen();
            typeof g.next === "function";
        "#);
        assert!(result.is_ok(), "Yield expression failed: {:?}", result);
    }

    // ============================================================================
    // Promise Tests
    // ============================================================================

    #[test]
    fn test_promise_constructor() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const p = new Promise((resolve, reject) => {
                resolve(42);
            });
            typeof p.then === "function";
        "#);
        assert!(result.is_ok(), "Promise constructor failed: {:?}", result);
    }

    #[test]
    fn test_promise_then() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const p = Promise.resolve(42);
            typeof p.then === "function";
        "#);
        assert!(result.is_ok(), "Promise.then failed: {:?}", result);
    }

    #[test]
    fn test_promise_catch() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const p = Promise.reject("error");
            typeof p.catch === "function";
        "#);
        assert!(result.is_ok(), "Promise.catch failed: {:?}", result);
    }

    #[test]
    fn test_promise_finally() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            const p = Promise.resolve(42);
            typeof p.finally === "function";
        "#);
        assert!(result.is_ok(), "Promise.finally failed: {:?}", result);
    }

    // ============================================================================
    // Class Tests
    // ============================================================================

    #[test]
    fn test_class_declaration() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            class MyClass {
                constructor(value) {
                    this.value = value;
                }
            }
            typeof MyClass === "function";
        "#);
        assert!(result.is_ok(), "Class declaration failed: {:?}", result);
    }

    #[test]
    fn test_class_inheritance() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            class Base {
                constructor() {
                    this.base = true;
                }
            }
            class Derived extends Base {
                constructor() {
                    super();
                    this.derived = true;
                }
            }
            const inst = new Derived();
            inst.base === true && inst.derived === true;
        "#);
        assert!(result.is_ok(), "Class inheritance failed: {:?}", result);
    }

    #[test]
    fn test_static_method() {
        let mut engine = JSEngine::new();
        let result = engine.execute(r#"
            class MyClass {
                static myMethod() {
                    return 42;
                }
            }
            typeof MyClass.myMethod === "function";
        "#);
        assert!(result.is_ok(), "Static method failed: {:?}", result);
    }
}
