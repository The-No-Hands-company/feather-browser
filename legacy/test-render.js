// Simple test to check what the engine is rendering
const pronbrowEngine = require('@pronbrow/engine');
const fs = require('fs');

console.log('Testing PronBrow Engine Rendering...\n');

// Create engine
const engine = pronbrowEngine.createBrowserEngine();
console.log(`Engine version: ${pronbrowEngine.getVersion()}\n`);

// Simple test HTML
const html = `
<!DOCTYPE html>
<html>
<head>
  <style>
    body { 
      margin: 0; 
      padding: 20px; 
      font-family: Arial; 
      background: #667eea; 
      color: white;
    }
    h1 { 
      font-size: 32px; 
      margin-bottom: 10px;
    }
    p { 
      font-size: 16px; 
    }
  </style>
</head>
<body>
  <h1>Hello PronBrow!</h1>
  <p>This is a test rendering.</p>
</body>
</html>
`;

// Render
const width = 800;
const height = 600;
console.log(`Rendering ${width}x${height}...`);

try {
  const pixels = engine.renderPage(html, '', width, height);
  
  console.log(`✓ Rendered successfully`);
  console.log(`  Pixels returned: ${pixels.length} bytes`);
  console.log(`  Expected: ${width * height * 4} bytes`);
  console.log(`  Match: ${pixels.length === width * height * 4 ? 'YES' : 'NO'}`);
  
  // Sample some pixels to see if they're not all white
  const sampleSize = 100;
  let uniqueColors = new Set();
  for (let i = 0; i < sampleSize; i++) {
    const idx = Math.floor(Math.random() * pixels.length / 4) * 4;
    const color = `${pixels[idx]},${pixels[idx+1]},${pixels[idx+2]},${pixels[idx+3]}`;
    uniqueColors.add(color);
  }
  
  console.log(`  Unique colors in sample: ${uniqueColors.size}`);
  console.log(`  Sample colors:`);
  Array.from(uniqueColors).slice(0, 5).forEach(c => {
    console.log(`    RGBA(${c})`);
  });
  
} catch (error) {
  console.error('✗ Rendering failed:', error.message);
  console.error(error.stack);
}
