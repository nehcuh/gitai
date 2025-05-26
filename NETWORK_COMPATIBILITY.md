# 网络兼容处理说明

## 问题背景

在使用 `cargo test` 或 `cargo build` 时，项目的 `build.rs` 脚本会尝试从 GitHub 下载 Tree-sitter 查询文件（highlights.scm、injections.scm、locals.scm）。这会导致以下问题：

1. **网络依赖**：每次构建都需要网络连接
2. **构建缓慢**：网络请求增加构建时间
3. **构建失败**：网络不可用时构建会失败
4. **重复下载**：即使本地已有文件也会重复请求

## 解决方案

我们在 `build.rs` 中实现了智能的网络兼容处理机制：

### 1. 本地文件优先策略
- 在尝试网络下载前，先检查本地文件是否已存在
- 如果文件存在，跳过下载并输出友好的提示信息
- 大大减少不必要的网络请求

### 2. 网络失败回退机制
- 当网络请求失败时，不会导致构建失败
- 自动创建包含基础查询的备用文件
- 确保项目能在离线环境下正常构建

### 3. 详细的构建日志
- 清晰地显示哪些文件被跳过
- 记录网络请求的成功/失败状态
- 帮助开发者了解构建过程

## 实现细节

### 文件检查逻辑
```rust
// 首先检查本地文件是否已存在
if dest.exists() {
    println!("cargo:warning=本地文件已存在，跳过下载: {}", dest.display());
    println!("cargo:rerun-if-changed={}", dest.display());
    continue;
}
```

### 网络失败处理
```rust
match get(&url) {
    Ok(resp) => {
        // 处理成功响应
    }
    Err(e) => {
        println!("cargo:warning=网络请求失败 {}: {}. 如果本地存在备用文件，构建将继续。", url, e);
        // 创建备用查询文件
        create_fallback_query(&dest, lang, file);
    }
}
```

### 备用查询内容
为每种语言提供基础的语法高亮查询：

- **Rust**: 函数、结构体、枚举、模块等基础语法
- **Java**: 方法、类、接口、包导入等基础语法  
- **Python**: 函数、类、导入语句等基础语法
- **其他语言**: 通用的标识符查询

## 使用效果

### 正常情况（本地文件存在）
```
warning: gitai@0.1.0: 本地文件已存在，跳过下载: /path/to/queries/rust/highlights.scm
warning: gitai@0.1.0: 本地文件已存在，跳过下载: /path/to/queries/rust/injections.scm
warning: gitai@0.1.0: Tree-sitter 查询文件准备完成
```

### 网络失败情况
```
warning: gitai@0.1.0: 网络请求失败 https://example.com/file: 连接超时
warning: gitai@0.1.0: 创建备用查询文件: /path/to/queries/rust/highlights.scm
warning: gitai@0.1.0: Tree-sitter 查询文件准备完成
```

## 优势

1. **离线友好**：无网络环境下也能正常构建
2. **构建加速**：跳过不必要的网络请求
3. **稳定性提升**：网络问题不会阻止构建
4. **开发体验改善**：减少等待时间，提高开发效率
5. **CI/CD 友好**：在受限网络环境下也能正常工作

## 维护说明

### 查询文件更新
如果需要更新 Tree-sitter 查询文件：
1. 删除对应的本地文件
2. 重新运行 `cargo build`
3. 系统会自动下载最新版本

### 添加新语言支持
1. 在 `build.rs` 的 `repos` 数组中添加新语言
2. 在 `create_fallback_query` 函数中添加对应的备用查询
3. 确保新语言的查询文件能正确下载和创建

### 调试构建过程
使用 `cargo build -v` 查看详细的构建日志，包括所有的警告和状态信息。

## 总结

这个网络兼容处理机制确保了 gitai 项目能够在各种网络环境下稳定构建，同时保持了对最新 Tree-sitter 查询文件的支持。它平衡了功能完整性和构建稳定性，为开发者提供了更好的使用体验。