# WebAssembly Examples

This directory contains examples demonstrating how to use the Claude Agent SDK Rust compiled to WebAssembly.

## 📁 Examples

### simple.html
A basic HTML example showing:
- WASM module initialization
- Simple query execution
- Error handling
- Loading states

### react-app/
Complete React application demonstrating:
- React 18 integration
- Hooks-based usage
- Error boundaries
- Loading states
- TypeScript support

### vue-app/
Vue 3 application demonstrating:
- Composition API usage
- Reactive state management
- Component integration
- Props and events

## 🚀 Running Examples

### Prerequisites

```bash
# Build the WASM package first
cd ../..
./scripts/build_wasm.sh
```

### Running simple.html

```bash
# Start a local server
python -m http.server 8000

# Or with Node.js
npx http-server -p 8000

# Open in browser
open http://localhost:8000/examples/wasm/simple.html
```

### Running React App

```bash
cd react-app
npm install
npm start
# Open http://localhost:3000
```

### Running Vue App

```bash
cd vue-app
npm install
npm run dev
# Open http://localhost:5173
```

## 🎯 Key Concepts

### 1. WASM Initialization

```javascript
import init, { query } from './pkg/claude_agent_sdk_rs.js';

// Initialize the WASM module
await init();

// Now you can use the SDK
const result = await query("What is 2 + 2?", null);
```

### 2. Async/Await Usage

All SDK operations return Promises in JavaScript:

```javascript
async function handleQuery() {
    try {
        const result = await query("Your query", null);
        console.log(result);
    } catch (error) {
        console.error('Error:', error);
    }
}
```

### 3. Error Handling

```javascript
try {
    const result = await query("Test", null);
    // Handle result
} catch (error) {
    // Handle error
    console.error('Query failed:', error);
}
```

## 🔧 Configuration

### TypeScript Support

```typescript
import init, { query } from './pkg/claude_agent_sdk_rs.js';

interface QueryOptions {
    model?: string;
    max_turns?: number;
}

async function runQuery(prompt: string, options?: QueryOptions) {
    await init();
    const result = await query(prompt, options);
    return result;
}
```

### Custom Configuration

```javascript
const options = {
    model: "claude-sonnet-4-5",
    max_turns: 5,
    // ... other options
};

const result = await query("Your query", options);
```

## 📦 Bundle Size

The WASM bundle size varies based on features:

| Build Type | Size | Notes |
|-----------|------|-------|
| Debug | ~8MB | Unoptimized |
| Release | ~4MB | Optimized |
| Minified | ~2MB | With wasm-opt |

### Optimization

```bash
# Optimize with wasm-opt
wasm-opt pkg/claude_agent_sdk_rs_bg.wasm -O3 -o pkg/optimized.wasm

# Enable gzip compression on web server
# Reduces transfer size to ~500KB
```

## 🌐 Browser Support

| Browser | Version | Status |
|---------|---------|--------|
| Chrome | 57+ | ✅ Full support |
| Firefox | 52+ | ✅ Full support |
| Safari | 11+ | ✅ Full support |
| Edge | 16+ | ✅ Full support |

## ⚡ Performance Tips

1. **Initialize early**: Call `init()` during app startup
2. **Reuse connections**: Don't create new clients for each query
3. **Use streaming**: For large responses, use streaming API
4. **Enable caching**: Cache WASM module in browser
5. **Optimize bundle**: Use `wasm-opt` for production

## 🔍 Debugging

### Chrome DevTools

1. Open DevTools (F12)
2. Go to Console tab
3. Check for errors in console
4. Use Network tab to see WASM loading

### WASM Tracing

```javascript
// Enable detailed logging
window.addEventListener('load', () => {
    console.log('Loading WASM module...');
});
```

## 📚 Additional Resources

- [WebAssembly MDN Guide](https://developer.mozilla.org/en-US/docs/WebAssembly)
- [WASM Pack Documentation](https://rustwasm.github.io/wasm-pack/)
- [React + WASM Guide](https://rustwasm.github.io/wasm-bindgen/examples/react.html)
- [Vue + WASM Guide](https://rustwasm.github.io/wasm-bindgen/examples/vue.html)

## 🐛 Troubleshooting

### Common Issues

**Issue**: "TypeError: WebAssembly.instantiate() failed"
```
Solution: Make sure you built the WASM package:
./scripts/build_wasm.sh
```

**Issue**: "Cannot find module './pkg/...'"
```
Solution: Check the path is correct relative to your HTML file
```

**Issue**: "CORS error when loading WASM"
```
Solution: Serve files from a web server, not file:// protocol
```

## 💡 Best Practices

1. ✅ Always serve from a web server (not file://)
2. ✅ Enable gzip/brotli compression
3. ✅ Set proper cache headers for WASM files
4. ✅ Handle initialization errors gracefully
5. ✅ Show loading states during queries
6. ✅ Test in multiple browsers
7. ✅ Use TypeScript for type safety
8. ✅ Monitor bundle size in CI/CD

## 🔄 Development Workflow

```bash
# 1. Make changes to Rust code
vim src/lib.rs

# 2. Rebuild WASM
./scripts/build_wasm.sh

# 3. Test in browser
python -m http.server 8000

# 4. Iterate
```

## 📝 License

Same as the main project (MIT License)
