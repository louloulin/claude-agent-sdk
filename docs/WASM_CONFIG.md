# WASM Build Configuration

## Cargo Configuration for WASM

Add to `.cargo/config.toml`:

```toml
[build]
target = "wasm32-unknown-unknown"

[target.wasm32-unknown-unknown]
rustflags = [
    "-C", "link-args=--import-memory",
    "-C", "link-args=--export-table",
    "-C", "link-args=--export=__wbindgen_malloc__",
    "-C", "link-args=--export=__wbindgen_realloc__"
]

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1

[profile.release.package."*"]
opt-level = 3
```

## Package.json Scripts

```json
{
  "scripts": {
    "build:wasm": "wasm-pack build --target web --out-dir pkg",
    "build:wasm:release": "wasm-pack build --target web --out-dir pkg --release",
    "serve:wasm": "python -m http.server 8000",
    "test:wasm": "node test_wasm.js"
  }
}
```

## Vite Configuration (for React apps)

```javascript
// vite.config.js
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  optimizeDeps: {
    exclude: ['claude-agent-sdk-rs']
  },
  server: {
    headers: {
      'Cross-Origin-Embedder-Policy': 'require-corp',
      'Cross-Origin-Opener-Policy': 'same-origin'
    }
  },
  build: {
    target: 'esnext',
    rollupOptions: {
      output: {
        inlineDynamicImports: false
      }
    }
  }
});
```

## Webpack Configuration (alternative)

```javascript
// webpack.config.js
const path = require('path');

module.exports = {
  entry: './index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bundle.js',
  },
  module: {
    rules: [
      {
        test: /\.wasm$/,
        type: 'webassembly/async',
      },
      {
        test: /\.js$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader',
        },
      },
    ],
  },
  experiments: {
    asyncWebAssembly: true,
  },
};
```

## TypeScript Configuration

```json
// tsconfig.json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "lib": ["ESNext", "DOM"],
    "jsx": "react-jsx",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```
