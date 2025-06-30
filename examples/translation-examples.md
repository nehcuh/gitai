# 🌐 GitAI 翻译功能完全指南 / Translation Feature Complete Guide

## 概述 / Overview

GitAI 提供了强大的多语言翻译支持，使开发者和团队能够以自己偏好的语言查看代码分析结果。这些功能无缝集成到所有命令中，支持中英文双语输出。

GitAI offers powerful multilingual translation support, allowing developers and teams to view code analysis results in their preferred language. These features are seamlessly integrated into all commands, supporting both Chinese and English output.

## 基础配置 / Basic Configuration

### 配置文件 / Configuration File

```toml
# ~/.config/gitai/config.toml

[translation]
enabled = true                        # 启用翻译功能 / Enable translation features
default_language = "zh"               # 默认语言: zh|en|auto / Default language: zh|en|auto
cache_enabled = true                  # 启用缓存以提高性能 / Enable caching for better performance
provider = "openai"                   # 翻译服务提供商 / Translation service provider
cache_dir = "~/.cache/gitai/translation" # 缓存目录 / Cache directory location

[translation.provider_settings]
api_key = "your-translation-api-key"  # API密钥 / API key for the provider
model = "gpt-3.5-turbo"               # 使用的模型 / Model to use for translation
```

### 环境变量 / Environment Variables

```bash
# 设置默认语言 / Set default language
export GITAI_TRANSLATION_LANGUAGE=zh

# 启用或禁用翻译 / Enable or disable translation
export GITAI_TRANSLATION_ENABLED=true

# 设置翻译API密钥 / Set translation API key
export GITAI_TRANSLATION_API_KEY=your-api-key

# 设置翻译缓存目录 / Set translation cache directory
export GITAI_TRANSLATION_CACHE_DIR=~/.cache/gitai/custom-translation-cache
```

## 使用示例 / Usage Examples

### 命令行语言选项 / Command Line Language Options

```bash
# 全局语言设置 / Global language setting
gitai --lang=zh <command>              # 中文输出 / Chinese output
gitai --lang=en <command>              # 英文输出 / English output
gitai --lang=auto <command>            # 自动检测系统语言 / Auto-detect system language

# 子命令特定语言设置 / Command-specific language setting
gitai scan --lang=zh src/              # 中文扫描结果 / Chinese scan results
gitai review --lang=en --commit-id=HEAD # 英文代码评审 / English code review
```

### 扫描命令示例 / Scan Command Examples

```bash
# 基本扫描 / Basic scanning
gitai scan --lang=zh src/               # 中文结果 / Chinese results
gitai scan --lang=en src/               # 英文结果 / English results

# 详细输出 / Verbose output
gitai scan --lang=zh src/ --verbose     # 中文详细输出 / Chinese verbose output
gitai scan --lang=en src/ --verbose     # 英文详细输出 / English verbose output

# 格式化输出 / Formatted output
gitai scan --lang=zh src/ --format=json --output=scan-zh.json  # 中文JSON输出 / Chinese JSON output
gitai scan --lang=en src/ --format=json --output=scan-en.json  # 英文JSON输出 / English JSON output
```

### 评审命令示例 / Review Command Examples

```bash
# 基本代码评审 / Basic code review
gitai review --lang=zh                  # 中文评审 / Chinese review
gitai review --lang=en                  # 英文评审 / English review

# 特定重点评审 / Focused review
gitai review --lang=zh --focus="性能问题,安全漏洞"    # 中文性能和安全重点 / Chinese performance and security focus
gitai review --lang=en --focus="performance,security" # 英文性能和安全重点 / English performance and security focus

# 评审格式输出 / Review format output
gitai review --lang=zh --format=markdown > review-zh.md  # 中文Markdown输出 / Chinese Markdown output
gitai review --lang=en --format=json > review-en.json    # 英文JSON输出 / English JSON output
```

## 高级应用场景 / Advanced Usage Scenarios

### 多语言团队协作 / Multilingual Team Collaboration

**场景描述 / Scenario Description**: 
国际团队成员使用不同的语言进行协作，需要以各自偏好的语言查看和评审代码。

International team members collaborating using different languages need to view and review code in their preferred language.

**解决方案 / Solution**:

```bash
# 中文成员使用 / Chinese team members
gitai --lang=zh review --format=markdown > review-zh.md

# 英文成员使用 / English team members
gitai --lang=en review --format=markdown > review-en.md

# 合并评审结果 / Merge review results
# 使用脚本合并不同语言的评审结果 / Use a script to merge reviews in different languages
```

**配置示例 / Configuration Example**:

```toml
# 团队配置文件 / Team configuration file
[translation]
enabled = true
default_language = "auto"  # 自动检测成员语言偏好 / Auto-detect team member's language preference
cache_enabled = true
provider = "openai"
cache_dir = "~/.cache/gitai/translation"

[translation.provider_settings]
api_key = "${GITAI_TRANSLATION_API_KEY}"  # 使用环境变量 / Use environment variable
model = "gpt-3.5-turbo"
```

### CI/CD 集成 / CI/CD Integration

**场景描述 / Scenario Description**:
在CI/CD流程中自动执行代码扫描和评审，并以多种语言生成报告。

Automatically perform code scanning and reviews in CI/CD pipelines and generate reports in multiple languages.

**GitHub Actions 示例 / GitHub Actions Example**:

```yaml
name: GitAI Code Analysis

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  analyze:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
      with:
        fetch-depth: 0  # 获取完整历史以进行分析 / Fetch complete history for analysis
        
    - name: Set up GitAI
      run: |
        # 安装GitAI / Install GitAI
        curl -sSL https://example.com/install-gitai.sh | bash
        
    - name: Run Chinese Analysis
      env:
        GITAI_TRANSLATION_API_KEY: ${{ secrets.TRANSLATION_API_KEY }}
      run: |
        gitai --lang=zh scan . --format=json --output=scan-zh.json
        gitai --lang=zh review --format=markdown > review-zh.md
        
    - name: Run English Analysis
      env:
        GITAI_TRANSLATION_API_KEY: ${{ secrets.TRANSLATION_API_KEY }}
      run: |
        gitai --lang=en scan . --format=json --output=scan-en.json
        gitai --lang=en review --format=markdown > review-en.md
        
    - name: Upload Analysis Results
      uses: actions/upload-artifact@v3
      with:
        name: code-analysis-reports
        path: |
          scan-zh.json
          scan-en.json
          review-zh.md
          review-en.md
```

### 批处理翻译 / Batch Translation

**场景描述 / Scenario Description**:
批量处理多个项目或目录，生成双语报告。

Process multiple projects or directories in batch, generating bilingual reports.

**脚本示例 / Script Example**:

```bash
#!/bin/bash
# batch-translation.sh - 批量处理多个项目的翻译报告

# 设置语言 / Set languages
LANGUAGES=("zh" "en")

# 设置项目列表 / Set project list
PROJECTS=("project1" "project2" "project3")

# 创建输出目录 / Create output directory
mkdir -p reports

# 循环处理每个项目 / Process each project
for project in "${PROJECTS[@]}"; do
  echo "Processing $project..."
  
  # 为每种语言生成报告 / Generate reports for each language
  for lang in "${LANGUAGES[@]}"; do
    echo "  - Generating $lang report..."
    
    # 扫描代码 / Scan code
    gitai --lang=$lang scan $project --format=json --output=reports/$project-scan-$lang.json
    
    # 评审代码 / Review code
    gitai --lang=$lang review --repo=$project --format=markdown > reports/$project-review-$lang.md
  done
  
  echo "Completed $project"
done

echo "All projects processed. Reports available in the 'reports' directory."
```

## 性能优化技巧 / Performance Optimization Tips

### 翻译缓存管理 / Translation Cache Management

```bash
# 使用缓存加速翻译 / Use cache to speed up translation
gitai scan --lang=zh src/ --use-cache

# 强制刷新翻译 / Force refresh translations
gitai scan --lang=zh src/ --force-scan

# 清理过期缓存 / Clean expired cache
rm -rf ~/.cache/gitai/translation/*

# 预热翻译缓存 / Warm up translation cache
gitai scan --lang=zh src/ --quiet  # 静默模式预热缓存 / Quiet mode to warm up cache
gitai scan --lang=en src/ --quiet
```

### 最小化翻译开销 / Minimize Translation Overhead

```bash
# 仅翻译输出结果 / Only translate output results
gitai scan --lang=zh src/ --translate-results-only

# 限制翻译范围 / Limit translation scope
gitai scan --lang=zh src/ --max-issues=10 --only-critical

# 翻译详细程度控制 / Control translation verbosity
gitai scan --lang=zh src/ --translation-level=basic  # 基本翻译 / Basic translation
gitai scan --lang=zh src/ --translation-level=full   # 完整翻译 / Full translation
```

## 故障排除 / Troubleshooting

### 常见问题 / Common Issues

1. **翻译未生效 / Translation Not Working**

   **症状 / Symptom**: 输出仍然是默认语言（英文）/ Output remains in default language (English)
   
   **解决方案 / Solution**:
   - 检查配置文件中的 `translation.enabled = true` / Check `translation.enabled = true` in config
   - 确认 `--lang` 参数设置正确 / Ensure `--lang` parameter is set correctly
   - 验证API密钥有效 / Verify API key is valid
   
   ```bash
   # 诊断命令 / Diagnostic command
   gitai --lang=zh scan src/ --verbose --debug-translation
   ```

2. **翻译速度慢 / Slow Translation**

   **症状 / Symptom**: 命令执行时间明显增加 / Command execution time significantly increased
   
   **解决方案 / Solution**:
   - 启用缓存 `translation.cache_enabled = true` / Enable caching
   - 使用更快的翻译提供商 / Use a faster translation provider
   - 限制翻译范围 / Limit translation scope
   
   ```bash
   # 性能测试命令 / Performance test command
   time gitai --lang=zh scan src/ --use-cache
   time gitai --lang=zh scan src/ --force-scan  # 对比强制重新翻译的时间 / Compare time for forced retranslation
   ```

3. **缓存问题 / Cache Issues**

   **症状 / Symptom**: 翻译结果不更新或不一致 / Translation results not updating or inconsistent
   
   **解决方案 / Solution**:
   - 清理缓存目录 / Clean cache directory
   - 使用 `--force-scan` 强制刷新 / Force refresh with `--force-scan`
   - 检查缓存目录权限 / Check cache directory permissions
   
   ```bash
   # 重置缓存命令 / Reset cache command
   rm -rf ~/.cache/gitai/translation
   mkdir -p ~/.cache/gitai/translation
   ```

### 启用调试模式 / Enable Debug Mode

```bash
# 详细的翻译调试信息 / Verbose translation debugging
export GITAI_DEBUG=true
export GITAI_TRANSLATION_DEBUG=true
gitai --lang=zh scan src/

# 输出翻译请求和响应 / Output translation requests and responses
gitai --lang=zh scan src/ --trace-translation

# 查看缓存状态 / View cache status
gitai translation-status
```

## 最佳实践 / Best Practices

1. **保持缓存启用 / Keep Cache Enabled**
   - 减少API调用，提高性能 / Reduce API calls, improve performance
   - 定期清理缓存以避免过时 / Periodically clean cache to avoid staleness

2. **智能使用语言设置 / Smart Language Settings**
   - 设置 `default_language = "auto"` 在团队环境中 / Set `default_language = "auto"` in team environments
   - 为CI/CD明确指定语言 / Explicitly specify language for CI/CD

3. **翻译API密钥管理 / Translation API Key Management**
   - 使用环境变量而非硬编码 / Use environment variables instead of hardcoding
   - 在CI/CD系统中使用密钥管理 / Use secrets management in CI/CD systems

4. **性能优化 / Performance Optimization**
   - 先用英文进行开发调试 / Use English for development and debugging
   - 在最终报告生成时使用翻译 / Use translation for final report generation
   - 合理设置缓存大小和清理策略 / Set reasonable cache size and cleaning policy

5. **多语言工作流 / Multilingual Workflow**
   - 建立团队翻译术语表 / Establish team translation glossary
   - 在PR流程中包含多语言报告 / Include multilingual reports in PR process
   - 为翻译内容维护版本控制 / Maintain version control for translated content

---

## 扩展示例 / Extended Examples

### Docker 环境中的翻译 / Translation in Docker Environment

```dockerfile
FROM ubuntu:22.04

# 安装GitAI / Install GitAI
RUN curl -sSL https://example.com/install-gitai.sh | bash

# 设置翻译配置 / Set up translation configuration
COPY config.toml /root/.config/gitai/config.toml

# 设置环境变量 / Set environment variables
ENV GITAI_TRANSLATION_ENABLED=true
ENV GITAI_TRANSLATION_LANGUAGE=zh

# 创建工作目录 / Create working directory
WORKDIR /code

# 入口命令 / Entry command
ENTRYPOINT ["gitai"]
CMD ["--help"]
```

### 自动语言切换脚本 / Automatic Language Switching Script

```bash
#!/bin/bash
# auto-language.sh - 根据用户区域设置自动切换GitAI语言

# 检测系统语言 / Detect system language
SYSTEM_LANG=$(locale | grep LANG | cut -d= -f2 | cut -d_ -f1)

# 设置GitAI语言 / Set GitAI language
if [[ "$SYSTEM_LANG" == "zh" ]]; then
  GITAI_LANG="zh"
else
  GITAI_LANG="en"
fi

# 输出诊断信息 / Output diagnostic information
echo "System language: $SYSTEM_LANG"
echo "Selected GitAI language: $GITAI_LANG"

# 运行GitAI命令 / Run GitAI command
gitai --lang=$GITAI_LANG "$@"
```

使用方法 / Usage:
```bash
# 代替直接调用gitai / Instead of calling gitai directly
./auto-language.sh scan src/
./auto-language.sh review
```
