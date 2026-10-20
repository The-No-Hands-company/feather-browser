const pronbrow = require('./pronbrow_engine.linux-x64-gnu.node');

const code = `
const p = new Promise((resolve, reject) => {
    resolve(42);
});
typeof p.then === "function";
`;

try {
    const result = pronbrow.execute(code);
    console.log('Result:', result);
} catch (e) {
    console.error('Error:', e.message);
}
