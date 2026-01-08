# WASM 优化指南

## 📊 性能目标

| 指标 | 目标值 | 说明 |
|------|--------|------|
| 初始加载时间 | < 2秒 | 在 3G 网络下 |
| 首次查询响应 | < 500ms | 不包括网络延迟 |
| WASM 包大小 | < 2MB | 压缩后 |
| 内存占用 | < 50MB | 典型使用场景 |

## 🔧 编译优化

### 1. Rust 代码优化

```rust
// 使用 #[wasm_bindgen] 时避免不必要的克隆
#[wasm_bindgen]
pub fn process_data(input: &str) -> String {
    // ❌ 避免
    // let owned = input.to_string();
    // owned.transform()

    // ✅ 推荐
    input.transform().to_string()
}

// 使用 JsValue 避免序列化开销
#[wasm_bindgen]
pub fn transfer_large_data(data: JsValue) -> JsValue {
    // 直接传递 JsValue，避免序列化
    process_js_value(data)
}
```

### 2. Cargo.toml 配置优化

```toml
[dependencies]
wee_alloc = "0.4.5"  # 更小的分配器

[dependencies.wee_alloc]
version = "0.4.5"
optional = true

[features]
default = ["wee_alloc"]

[profile.release]
opt-level = "z"        # 优化大小
lto = true             # 链接时优化
codegen-units = 1      # 单编译单元
panic = "abort"        # 减少二进制大小

[profile.release.package."*"]
opt-level = 3          # 依赖项全优化
```

### 3. wasm-pack 构建选项

```bash
# 基础优化构建
wasm-pack build --target web --release -- --no-threads

# 自定义优化
wasm-pack build \
  --target web \
  --release \
  -- --no-threads \
  --no-default-features \
  --features "wee_alloc"
```

## ⚡ wasm-opt 高级优化

### 安装 Binaryen

```bash
# macOS
brew install binaryen

# Linux
wget https://github.com/WebAssembly/binaryen/releases/download/version_116/binaryen-version_116-x86_64-linux.tar.gz
tar -xf binaryen-version_116-x86_64-linux.tar.gz
export PATH=$PATH:/path/to/binaryen/bin
```

### 优化级别

```bash
# Level 1: 快速优化（-O1）
wasm-opt pkg/claude_agent_sdk_rs_bg.wasm -O1 -o pkg/optimized.wasm

# Level 2: 标准优化（-O2，推荐）
wasm-opt pkg/claude_agent_sdk_rs_bg.wasm -O2 -o pkg/optimized.wasm

# Level 3: 激进优化（-O3，最慢但最优）
wasm-opt pkg/claude_agent_sdk_rs_bg.wasm -O3 -o pkg/optimized.wasm

# Level 4: 大小优化（-O4，最小体积）
wasm-opt pkg/claude_agent_sdk_rs_bg.wasm -O4 -o pkg/optimized.wasm

# Level Z: 极致大小优化（-Oz）
wasm-opt pkg/claude_agent_sdk_rs_bg.wasm -Oz -o pkg/optimized.wasm
```

### 特定优化选项

```bash
# 内联函数
wasm-opt input.wasm -O3 --inline-functions-with-calls -o output.wasm

# 死代码消除
wasm-opt input.wasm -O3 --dce -o output.wasm

# 循环优化
wasm-opt input.wasm -O3 --optimize-instructions -o output.wasm

# 合并页面
wasm-opt input.wasm -O3 --merge-parallel-load-store -o output.wasm

# 组合多个优化
wasm-opt input.wasm \
  -O3 \
  --inline-functions-with-calls \
  --dce \
  --optimize-instructions \
  --merge-parallel-load-store \
  -o output.wasm
```

## 📦 打包优化

### 1. Gzip 压缩

```nginx
# Nginx 配置
location ~ \.wasm$ {
    gzip on;
    gzip_types application/wasm;
    gzip_comp_level 9;
    gzip_min_length 1000;
    add_header Content-Encoding gzip;
}
```

```bash
# 手动压缩
gzip -9 pkg/claude_agent_sdk_rs_bg.wasm -c > pkg/claude_agent_sdk_rs_bg.wasm.gz
```

### 2. Brotli 压缩（更好压缩率）

```bash
# 安装 brotli
brew install brotli  # macOS
apt install brotli  # Linux

# 压缩
brotli -q 11 pkg/claude_agent_sdk_rs_bg.wasm -o pkg/claude_agent_sdk_rs_bg.wasm.br
```

```nginx
# Nginx 配置
location ~ \.wasm$ {
    brotli on;
    brotli_types application/wasm;
    brotli_comp_level 11;
    brotli_static on;
}
```

### 3. 压缩效果对比

| 格式 | 原始大小 | 压缩后 | 压缩率 | 传输时间* |
|------|---------|--------|--------|----------|
| 无压缩 | 4.5 MB | 4.5 MB | 0% | 8.3s |
| Gzip | 4.5 MB | 1.1 MB | 75% | 2.0s |
| Brotli | 4.5 MB | 850 KB | 81% | 1.5s |

*基于 4.4 Mbps (3G) 网络计算

## 🚀 运行时优化

### 1. 预加载和缓存

```javascript
// 预加载 WASM 模块
<link rel="preload" href="pkg/claude_agent_sdk_rs_bg.wasm" as="fetch" crossorigin>
<link rel="modulepreload" href="pkg/claude_agent_sdk_rs.js">

// Service Worker 缓存
self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open('wasm-cache-v1').then((cache) => {
            return cache.addAll([
                '/pkg/claude_agent_sdk_rs_bg.wasm',
                '/pkg/claude_agent_sdk_rs.js',
            ]);
        })
    );
});

self.addEventListener('fetch', (event) => {
    event.respondWith(
        caches.match(event.request).then((response) => {
            return response || fetch(event.request);
        })
    );
});
```

### 2. 延迟初始化

```javascript
// 不阻塞页面加载
async function initWasmWhenNeeded() {
    const { init } = await import('./pkg/claude_agent_sdk_rs.js');
    await init();
    return true;
}

// 用户交互时初始化
button.addEventListener('click', async () => {
    if (!wasmReady) {
        await initWasmWhenNeeded();
        wasmReady = true;
    }
    // 使用 WASM 功能
});
```

### 3. SharedArrayBuffer 优化

```javascript
// 需要 COOP/COEP 头
const sharedBuffer = new SharedArrayBuffer(4096);
const sharedArray = new Int32Array(sharedBuffer);

// 在 Rust 端使用
use wasm_bindgen::prelude::*;
use web_sys::js_sys::{SharedArrayBuffer, Int32Array};

#[wasm_bindgen]
pub fn use_shared_memory(buffer: SharedArrayBuffer) {
    let array = Int32Array::new(&buffer);
    // 高效的共享内存操作
}
```

## 🔍 性能分析

### 1. Chrome DevTools 分析

```javascript
// 在代码中添加性能标记
performance.mark('wasm-init-start');
await init();
performance.mark('wasm-init-end');
performance.measure('wasm-init', 'wasm-init-start', 'wasm-init-end');

// 查询性能标记
const measures = performance.getEntriesByType('measure');
console.table(measures);
```

### 2. WebAssembly 性能分析

```javascript
// 使用 WebAssembly.instantiateStreaming 的回调
const streamingResult = await WebAssembly.instantiateStreaming(
    fetch('pkg/claude_agent_sdk_rs_bg.wasm'),
    importObject
);

// 监控实例化时间
performance.mark('wasm-instantiate-start');
const instance = streamingResult.instance;
performance.mark('wasm-instantiate-end');
```

### 3. 内存分析

```javascript
// 检查 WASM 内存使用
const wasmMemory = instance.exports.memory;

function checkMemoryUsage() {
    const buffer = wasmMemory.buffer;
    const pages = wasmMemory.buffer.byteLength / 65536; // 64KB per page
    console.log(`WASM memory: ${pages} pages (${(buffer.byteLength / 1024 / 1024).toFixed(2)} MB)`);

    // 分析内存增长
    const used = new Uint8Array(buffer).filter(v => v !== 0).length;
    const usage = (used / buffer.byteLength) * 100;
    console.log(`Memory usage: ${usage.toFixed(2)}%`);
}

setInterval(checkMemoryUsage, 5000);
```

## 📏 性能基准测试

### 基准测试脚本

```javascript
// benchmarks/wasm_perf.js
import init, { query } from '../pkg/claude_agent_sdk_rs.js';

async function benchmark() {
    await init();

    const testCases = [
        { name: 'Simple query', query: 'What is 2 + 2?' },
        { name: 'Complex query', query: 'Explain quantum computing' },
        { name: 'Multi-turn', query: 'Create a todo list' }
    ];

    for (const test of testCases) {
        const times = [];
        const iterations = 10;

        for (let i = 0; i < iterations; i++) {
            const start = performance.now();
            await query(test.query, null);
            const end = performance.now();
            times.push(end - start);
        }

        const avg = times.reduce((a, b) => a + b) / times.length;
        const min = Math.min(...times);
        const max = Math.max(...times);

        console.log(`${test.name}:`);
        console.log(`  Average: ${avg.toFixed(2)}ms`);
        console.log(`  Min: ${min.toFixed(2)}ms`);
        console.log(`  Max: ${max.toFixed(2)}ms`);
    }
}

benchmark();
```

## 🎯 优化检查清单

### 编译时优化

- [ ] 启用 `wee_alloc` 替代默认分配器
- [ ] 设置 `opt-level = "z"` 优化大小
- [ ] 启用 `lto = true` 链接时优化
- [ ] 设置 `codegen-units = 1` 单编译单元
- [ ] 使用 `panic = "abort"` 减小二进制大小
- [ ] 移除不必要的依赖项
- [ ] 使用 `--no-default-features` 减少特性
- [ ] 运行 `wasm-opt -O3` 优化

### 打包优化

- [ ] 启用 Gzip 压缩
- [ ] 启用 Brotli 压缩
- [ ] 设置正确的 MIME 类型（application/wasm）
- [ ] 配置缓存头
- [ ] 使用预加载（<link rel="preload">）
- [ ] 实现 Service Worker 缓存

### 运行时优化

- [ ] 延迟初始化 WASM 模块
- [ ] 复用 WASM 实例
- [ ] 使用 SharedArrayBuffer 进行高效数据传输
- [ ] 实现内存监控
- [ ] 添加性能监控
- [ ] 使用 Web Workers 进行后台处理

## 🔧 常见问题

### Q: WASM 文件太大怎么办？

**A**: 使用以下优化组合：
1. `wasm-opt -O4` 优化
2. Brotli 压缩（~80% 压缩率）
3. 检查依赖项，移除不必要的 features
4. 使用 `#[wasm_bindgen(skip_typescript)]` 跳过类型生成

### Q: 初始化时间太长？

**A**:
1. 使用异步初始化
2. 预加载 WASM 文件
3. 延迟初始化到用户交互时
4. 使用 CDN 加速下载

### Q: 如何减少内存占用？

**A**:
1. 使用 `wee_alloc` 分配器（节省 ~40KB）
2. 及时释放大对象
3. 重用缓冲区
4. 监控内存使用

### Q: 多次初始化导致性能下降？

**A**:
```javascript
// 确保只初始化一次
let initPromise = null;

export async function ensureInitialized() {
    if (!initPromise) {
        initPromise = init();
    }
    return initPromise;
}
```

## 📚 参考资源

- [WebAssembly Optimization](https://webassembly.org/docs/optimizing-wasm/)
- [Binaryen Optimization Guide](https://github.com/WebAssembly/binaryen/blob/main/docs/Optimizations.md)
- [WASM Pack Optimization](https://rustwasm.github.io/wasm-pack/book/prerequisites/optimize-size.html)
- [Chrome DevTools WASM Profiling](https://developer.chrome.com/blog/webassembly-debugging-and-profiling-in-chrome-devtools/)

## 🎬 快速优化流程

```bash
# 1. 基础构建
./scripts/build_wasm.sh web --release

# 2. 高级优化
wasm-opt pkg/claude_agent_sdk_rs_bg.wasm -O3 -o pkg/claude_agent_sdk_rs_bg_opt.wasm
mv pkg/claude_agent_sdk_rs_bg_opt.wasm pkg/claude_agent_sdk_rs_bg.wasm

# 3. 压缩
brotli -q 11 pkg/claude_agent_sdk_rs_bg.wasm -o pkg/claude_agent_sdk_rs_bg.wasm.br

# 4. 验证
ls -lh pkg/*.wasm*
# 确保优化后大小合理

# 5. 测试
cd examples/wasm/react-app
npm run build
npm run preview
```
