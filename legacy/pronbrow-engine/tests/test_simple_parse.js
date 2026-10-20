const { JSEngine } = require('./pronbrow_engine.linux-x64-gnu.node');

const engine = new JSEngine();
try {
    const result = engine.execute("var x = 5");
    console.log("Success:", result);
} catch (e) {
    console.error("Error:", e);
}
