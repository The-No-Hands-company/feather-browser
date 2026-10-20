#!/usr/bin/env node

// Test script to verify pronbrow-engine integration
// Run this to verify everything is set up correctly before launching Electron

const path = require('path');

console.log('🧪 PronBrow Integration Test\n');

// Test 1: Load engine
console.log('Test 1: Loading pronbrow-engine...');
try {
  const engine = require('@pronbrow/engine');
  console.log('✅ Engine loaded successfully');
  console.log(`   Version: ${engine.getVersion()}`);
  console.log(`   Exports: ${Object.keys(engine).join(', ')}\n`);
} catch (error) {
  console.error('❌ Failed to load engine:', error.message);
  process.exit(1);
}

// Test 2: Create engine instance
console.log('Test 2: Creating browser engine instance...');
try {
  const engine = require('@pronbrow/engine');
  const browser = engine.createBrowserEngine();
  console.log('✅ Browser engine created\n');
} catch (error) {
  console.error('❌ Failed to create engine:', error.message);
  process.exit(1);
}

// Test 3: Parse HTML
console.log('Test 3: Testing HTML parsing...');
try {
  const engine = require('@pronbrow/engine');
  const browser = engine.createBrowserEngine();
  const html = '<html><body><h1>Test</h1><p>Hello World</p></body></html>';
  const result = browser.parseHtml(html);
  console.log('✅ HTML parsed successfully');
  console.log(`   Result length: ${result.length} chars\n`);
} catch (error) {
  console.error('❌ HTML parsing failed:', error.message);
  process.exit(1);
}

// Test 4: Parse CSS
console.log('Test 4: Testing CSS parsing...');
try {
  const engine = require('@pronbrow/engine');
  const browser = engine.createBrowserEngine();
  const css = 'body { background: #fff; color: #000; }';
  const result = browser.parseCss(css);
  console.log('✅ CSS parsed successfully');
  console.log(`   Result length: ${result.length} chars\n`);
} catch (error) {
  console.error('❌ CSS parsing failed:', error.message);
  process.exit(1);
}

// Test 5: Render page to pixels
console.log('Test 5: Testing page rendering...');
try {
  const engine = require('@pronbrow/engine');
  const browser = engine.createBrowserEngine();
  
  const html = `
    <html>
      <body>
        <h1 style="color: blue;">Hello PronBrow!</h1>
        <p>This is a test page.</p>
      </body>
    </html>
  `;
  const css = 'body { margin: 20px; background: #f0f0f0; }';
  
  const width = 400;
  const height = 300;
  const rgba_bytes = browser.renderPage(html, css, width, height);
  
  const expectedBytes = width * height * 4;
  console.log('✅ Page rendered successfully');
  console.log(`   Dimensions: ${width}x${height}`);
  console.log(`   Output: ${rgba_bytes.length} bytes (expected ${expectedBytes})`);
  console.log(`   Match: ${rgba_bytes.length === expectedBytes ? '✅' : '❌'}\n`);
  
  if (rgba_bytes.length !== expectedBytes) {
    throw new Error(`Byte count mismatch: got ${rgba_bytes.length}, expected ${expectedBytes}`);
  }
} catch (error) {
  console.error('❌ Rendering failed:', error.message);
  process.exit(1);
}

// Test 6: Check renderer modules
console.log('Test 6: Checking renderer modules...');
try {
  const CanvasRenderer = require('./src/renderer/CanvasRenderer');
  console.log('✅ CanvasRenderer module found');
  
  const PageRenderer = require('./src/renderer/PageRenderer');
  console.log('✅ PageRenderer module found\n');
} catch (error) {
  console.error('❌ Renderer modules missing:', error.message);
  process.exit(1);
}

// Test 7: Test PageRenderer initialization
console.log('Test 7: Testing PageRenderer initialization...');
try {
  const engine = require('@pronbrow/engine');
  const PageRenderer = require('./src/renderer/PageRenderer');
  
  const browser = engine.createBrowserEngine();
  const renderer = new PageRenderer(browser, 800, 600);
  
  console.log('✅ PageRenderer initialized successfully');
  console.log(`   Viewport: ${renderer.canvasRenderer.width}x${renderer.canvasRenderer.height}\n`);
  
  renderer.destroy();
} catch (error) {
  console.error('❌ PageRenderer initialization failed:', error.message);
  process.exit(1);
}

// Test 8: Check UI files
console.log('Test 8: Checking UI files...');
const fs = require('fs');
const files = [
  'browser.html',
  'browser-renderer.js',
  'main-engine.js',
  'engine-test.html',
  'src/renderer/CanvasRenderer.js',
  'src/renderer/PageRenderer.js'
];

let allFilesExist = true;
for (const file of files) {
  const exists = fs.existsSync(path.join(__dirname, file));
  console.log(`   ${exists ? '✅' : '❌'} ${file}`);
  if (!exists) allFilesExist = false;
}

if (!allFilesExist) {
  console.error('\n❌ Some files are missing!');
  process.exit(1);
}

console.log('\n' + '='.repeat(60));
console.log('🎉 ALL TESTS PASSED!');
console.log('='.repeat(60));
console.log('\nThe integration is working correctly!');
console.log('\nTo launch the browser, run:');
console.log('  npm run start:engine\n');
console.log('Or for development mode with DevTools:');
console.log('  npm run start:dev\n');
