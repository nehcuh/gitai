# 用户故事: 智能暂存支持

## 故事描述

**作为一个**开发者  
**我希望**使用 `gitai commit -a` 命令自动暂存修改的文件并提交  
**以便于**简化我的工作流程，无需手动执行 `git add` 就能直接进行智能提交

## 验收标准

### AC1: -a 参数支持
- **给定**我在一个有未暂存更改的 Git 仓库中
- **当**我执行 `gitai commit -a` 命令时
- **那么**系统应该自动暂存所有修改的文件
- **并且**然后执行正常的智能提交流程

### AC2: 智能文件过滤
- **给定**工作目录中有多种类型的文件变更
- **当**系统执行自动暂存时
- **那么**应该只暂存已跟踪的修改文件
- **并且**排除 .gitignore 中指定的文件
- **并且**不暂存新增的未跟踪文件

### AC3: 与其他参数的组合
- **给定**用户同时使用多个参数
- **当**执行 `gitai commit -a -t -m "message"` 时
- **那么**系统应该按正确顺序执行：
  - 自动暂存修改文件
  - 执行 Tree-sitter 分析
  - 生成增强的提交信息
  - 应用用户自定义消息

### AC4: 暂存状态检查
- **给定**部分文件已经暂存，部分文件未暂存
- **当**用户执行 `gitai commit -a` 时
- **那么**系统应该智能处理混合状态
- **并且**在分析时考虑所有相关的文件变更

### AC5: 安全确认机制
- **给定**系统将要暂存大量文件
- **当**检测到可能的误操作时
- **那么**应该显示将要暂存的文件列表
- **并且**要求用户确认后再继续

## 技术要求

### Git 操作集成
- 调用 `git add -u` 暂存已跟踪文件的修改
- 处理 `git status` 输出解析文件状态
- 正确处理文件重命名和删除操作
- 维护与标准 Git 行为的一致性

### 文件状态分析
- 区分已暂存、未暂存、未跟踪文件
- 处理文件权限和符号链接
- 检测二进制文件和大文件
- 应用 .gitignore 规则

### 智能过滤逻辑
- 排除临时文件和构建产物
- 检测敏感文件（密钥、配置等）
- 处理子模块和子目录
- 支持用户自定义过滤规则

## 实现细节

### 命令解析逻辑
```rust
// 检测 -a 参数
let auto_stage = args.iter().any(|arg| arg == "-a" || arg == "--all");

if auto_stage {
    // 执行智能暂存
    let staged_files = smart_stage_files().await?;
    tracing::info!("Auto-staged {} files", staged_files.len());
    
    // 继续正常的提交流程
    proceed_with_commit(args, &staged_files).await?;
}
```

### 智能暂存流程
1. 执行 `git status --porcelain` 获取文件状态
2. 解析输出，识别文件变更类型
3. 应用智能过滤规则
4. 显示将要暂存的文件列表
5. 执行 `git add` 操作
6. 验证暂存结果
7. 继续提交流程

### 文件状态解析
```rust
#[derive(Debug, PartialEq)]
enum FileStatus {
    Modified,        // M  - 已修改
    Added,          // A  - 新增
    Deleted,        // D  - 删除
    Renamed,        // R  - 重命名
    Copied,         // C  - 复制
    Untracked,      // ?? - 未跟踪
    Ignored,        // !! - 被忽略
}

struct FileChange {
    status: FileStatus,
    path: PathBuf,
    old_path: Option<PathBuf>, // for renames
    should_stage: bool,
}
```

### 智能过滤规则
```rust
fn should_stage_file(file: &FileChange) -> bool {
    match file.status {
        FileStatus::Modified | FileStatus::Deleted | FileStatus::Renamed => true,
        FileStatus::Untracked => false, // 需要用户明确添加
        FileStatus::Ignored => false,
        FileStatus::Added => {
            // 如果已经在暂存区，保持状态
            is_already_staged(&file.path)
        }
    }
}
```

### 安全检查机制
```rust
async fn safe_stage_files(files: &[FileChange]) -> Result<Vec<PathBuf>, GitError> {
    // 检查文件数量阈值
    if files.len() > MAX_FILES_WITHOUT_CONFIRMATION {
        show_staging_preview(files)?;
        if !confirm_staging()? {
            return Err(GitError::UserCancelled);
        }
    }
    
    // 检查敏感文件
    let sensitive_files = detect_sensitive_files(files);
    if !sensitive_files.is_empty() {
        warn_about_sensitive_files(&sensitive_files)?;
        if !confirm_sensitive_staging()? {
            return Err(GitError::SensitiveFilesDetected);
        }
    }
    
    execute_staging(files).await
}
```

### 用户输出格式
```
🔄 Auto-staging modified files...

Files to be staged:
  📝 src/auth.rs (modified)
  📝 src/config.rs (modified)  
  🗑️  old_file.rs (deleted)
  ↔️  src/utils.rs → src/helpers.rs (renamed)

⚠️  Detected 1 large file: data/sample.json (2.3MB)
⚠️  Sensitive file detected: .env.local

Continue with staging? [Y/n] y

✅ Successfully staged 4 files
🤖 Proceeding with AI commit generation...
```

## 配置选项

```toml
[auto_staging]
enabled = true
max_files_without_confirmation = 10
max_file_size_mb = 50
exclude_patterns = ["*.log", "*.tmp", "node_modules/*"]
include_sensitive_files = false

[staging_safety]
warn_on_large_files = true
warn_on_sensitive_files = true
require_confirmation_threshold = 20
auto_exclude_build_artifacts = true
```

## 错误处理

### 常见错误场景
- Git 仓库状态异常
- 文件权限问题
- 暂存操作失败
- 工作目录包含冲突文件

### 错误处理策略
- 提供详细的错误信息和解决建议
- 支持部分暂存成功的情况
- 在错误发生时恢复到原始状态
- 记录操作日志便于调试

## 完成定义

- [ ] 实现 `-a` 参数解析和处理逻辑
- [ ] 开发智能文件状态分析功能
- [ ] 实现安全的自动暂存机制
- [ ] 集成文件过滤和排除规则
- [ ] 添加用户确认和预览功能
- [ ] 实现与其他提交功能的集成
- [ ] 处理边界情况和错误恢复
- [ ] 编写文件暂存的单元测试
- [ ] 创建复杂场景的集成测试
- [ ] 测试与标准 Git 行为的兼容性
- [ ] 添加配置选项和文档

## 相关依赖

- 基础提交功能 (故事 01)
- Git 工具函数和状态解析
- 文件系统操作工具
- 用户交互和确认机制
- 配置系统
- 错误处理框架

## 优先级

**中** - 便利性功能，提高用户工作流效率

## 估算

**开发工作量**: 3-4 个开发日  
**测试工作量**: 2-3 个开发日

## 风险和缓解

### 主要风险
- 误暂存敏感或不需要的文件
- 与复杂 Git 状态的兼容性
- 性能问题（大型仓库）
- 用户期望与实际行为的差异

### 缓解策略
- 实现严格的安全检查机制
- 广泛测试各种 Git 仓库状态
- 优化文件扫描和处理性能
- 提供清晰的文档和使用指南