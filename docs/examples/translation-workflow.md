# 🌐 GitAI 翻译工作流示例 / GitAI Translation Workflow Examples

本文档提供了在不同场景下使用 GitAI 翻译功能的工作流示例，帮助您有效地集成多语言支持到开发流程中。

This document provides workflow examples for using GitAI translation features in different scenarios, helping you effectively integrate multilingual support into your development process.

## 1. 个人开发者工作流 / Individual Developer Workflow

### 日常开发工作流 / Daily Development Workflow

```bash
# 1. 开始工作日 - 检查代码状态（使用系统语言）
# 1. Start workday - Check code status (using system language)
gitai --lang=auto git status

# 2. 执行代码扫描（使用首选语言）
# 2. Perform code scan (using preferred language)
gitai --lang=zh scan src/

# 3. 修复问题
# 3. Fix issues
vim src/problematic_file.js

# 4. 验证修复（快速扫描）
# 4. Verify fix (quick scan)
gitai --lang=zh scan src/problematic_file.js

# 5. 提交修复（生成中文提交信息）
# 5. Commit fix (generate Chinese commit message)
gitai --lang=zh commit -a
```

### 代码评审准备 / Code Review Preparation

```bash
# 1. 在提交前运行全面评审
# 1. Run comprehensive review before submitting
gitai --lang=zh review --format=markdown > 我的评审.md

# 2. 生成英文版本的评审（用于国际同事）
# 2. Generate English version of review (for international colleagues)
gitai --lang=en review --format=markdown > my-review.md

# 3. 生成多语言扫描报告
# 3. Generate multilingual scan reports
gitai --lang=zh scan . --format=json --output=scan-zh.json
gitai --lang=en scan . --format=json --output=scan-en.json
```

## 2. 团队协作工作流 / Team Collaboration Workflow

### 设置团队翻译配置 / Setting Up Team Translation Configuration

**config.toml 示例 / config.toml Example:**

```toml
# ~/.config/gitai/config.toml
[translation]
enabled = true
default_language = "auto"  # 自动检测每个团队成员的语言偏好
cache_enabled = true
provider = "openai"
cache_dir = "${XDG_CACHE_HOME:-$HOME/.cache}/gitai/translation"

[translation.provider_settings]
api_key = "${GITAI_TRANSLATION_API_KEY}"  # 使用环境变量
model = "gpt-3.5-turbo"
```

### 多语言团队工作流 / Multilingual Team Workflow

```bash
# 创建翻译设置脚本 / Create translation setup script
cat > setup-translation.sh << 'EOF'
#!/bin/bash
# 根据用户区域设置配置GitAI语言
# Configure GitAI language based on user locale

# 检测系统语言 / Detect system language
SYSTEM_LANG=$(locale | grep LANG | cut -d= -f2 | cut -d_ -f1)

# 设置GitAI语言环境变量 / Set GitAI language environment variable
if [[ "$SYSTEM_LANG" == "zh" ]]; then
  echo "export GITAI_TRANSLATION_LANGUAGE=zh" >> ~/.bashrc
  echo "中文环境设置完成"
else
  echo "export GITAI_TRANSLATION_LANGUAGE=en" >> ~/.bashrc
  echo "English environment setup complete"
fi

# 重新加载配置 / Reload configuration
source ~/.bashrc
EOF

chmod +x setup-translation.sh
```

### 双语PR工作流 / Bilingual PR Workflow

```bash
# 1. 创建PR前的代码检查脚本 / Create pre-PR code check script
cat > pre-pr-check.sh << 'EOF'
#!/bin/bash
# 运行双语代码分析，确保团队所有成员都能理解

echo "🔍 Running bilingual code analysis..."

# 创建输出目录
mkdir -p pr-review

# 中文扫描和评审
echo "生成中文报告..."
gitai --lang=zh scan . --format=json --output=pr-review/scan-zh.json
gitai --lang=zh review --format=markdown > pr-review/review-zh.md

# 英文扫描和评审
echo "Generating English reports..."
gitai --lang=en scan . --format=json --output=pr-review/scan-en.json
gitai --lang=en review --format=markdown > pr-review/review-en.md

echo "✅ Bilingual analysis complete. Reports available in pr-review/ directory"
EOF

chmod +x pre-pr-check.sh
```

## 3. CI/CD 集成工作流 / CI/CD Integration Workflow

### GitHub Actions 工作流 / GitHub Actions Workflow

**workflow 文件示例 / workflow file example:**

```yaml
# .github/workflows/gitai-analysis.yml
name: GitAI Multilingual Analysis

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  analyze:
    runs-on: ubuntu-latest
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v3
      with:
        fetch-depth: 0
    
    - name: Set up GitAI
      run: |
        curl -sSL https://example.com/install-gitai.sh | bash
        echo "GITAI_TRANSLATION_API_KEY=${{ secrets.TRANSLATION_API_KEY }}" >> $GITHUB_ENV
    
    - name: Generate Chinese Analysis
      run: |
        mkdir -p reports
        gitai --lang=zh scan . --format=json --output=reports/scan-zh.json
        gitai --lang=zh review --format=markdown > reports/review-zh.md
    
    - name: Generate English Analysis
      run: |
        gitai --lang=en scan . --format=json --output=reports/scan-en.json
        gitai --lang=en review --format=markdown > reports/review-en.md
    
    - name: Upload Reports
      uses: actions/upload-artifact@v3
      with:
        name: gitai-multilingual-reports
        path: reports/
    
    - name: Comment on PR (if PR)
      if: github.event_name == 'pull_request'
      uses: actions/github-script@v6
      with:
        github-token: ${{ secrets.GITHUB_TOKEN }}
        script: |
          const fs = require('fs');
          const enReview = fs.readFileSync('reports/review-en.md', 'utf8');
          const zhReview = fs.readFileSync('reports/review-zh.md', 'utf8');
          
          // Create multilingual comment
          const comment = `## GitAI Code Analysis
          
          <details>
          <summary>English Report</summary>
          
          ${enReview}
          
          </details>
          
          <details>
          <summary>中文报告</summary>
          
          ${zhReview}
          
          </details>
          
          [Download full reports](${process.env.GITHUB_SERVER_URL}/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID})`;
          
          github.rest.issues.createComment({
            issue_number: context.issue.number,
            owner: context.repo.owner,
            repo: context.repo.repo,
            body: comment
          });
```

### Jenkins 流水线示例 / Jenkins Pipeline Example

```groovy
// Jenkinsfile
pipeline {
    agent any
    
    environment {
        GITAI_TRANSLATION_API_KEY = credentials('gitai-translation-api-key')
    }
    
    stages {
        stage('Setup GitAI') {
            steps {
                sh 'curl -sSL https://example.com/install-gitai.sh | bash'
            }
        }
        
        stage('Multilingual Analysis') {
            parallel {
                stage('Chinese Analysis') {
                    steps {
                        sh '''
                            mkdir -p reports
                            gitai --lang=zh scan . --format=json --output=reports/scan-zh.json
                            gitai --lang=zh review --format=markdown > reports/review-zh.md
                        '''
                    }
                }
                
                stage('English Analysis') {
                    steps {
                        sh '''
                            mkdir -p reports
                            gitai --lang=en scan . --format=json --output=reports/scan-en.json
                            gitai --lang=en review --format=markdown > reports/review-en.md
                        '''
                    }
                }
            }
        }
        
        stage('Archive Reports') {
            steps {
                archiveArtifacts artifacts: 'reports/**', fingerprint: true
            }
        }
    }
    
    post {
        always {
            // Notification with links to both language reports
            echo "Analysis complete. Chinese and English reports are available in the artifacts."
        }
    }
}
```

## 4. 高级工作流实例 / Advanced Workflow Examples

### 自动化多语言文档生成 / Automated Multilingual Documentation Generation

```bash
#!/bin/bash
# generate-multilingual-docs.sh
# 为项目生成多语言代码分析文档

# 设置语言 / Set languages
LANGUAGES=("zh" "en")

# 设置输出目录 / Set output directory
OUTPUT_DIR="docs/code-analysis"
mkdir -p "$OUTPUT_DIR"

# 设置日期格式 / Set date format
DATE_FORMAT=$(date +"%Y-%m-%d")

# 对每种语言生成文档 / Generate docs for each language
for lang in "${LANGUAGES[@]}"; do
  echo "Generating $lang documentation..."
  
  # 创建语言特定目录 / Create language-specific directory
  LANG_DIR="$OUTPUT_DIR/$lang"
  mkdir -p "$LANG_DIR"
  
  # 扫描代码 / Scan code
  gitai --lang=$lang scan . --format=json --output="$LANG_DIR/scan-$DATE_FORMAT.json"
  
  # 生成全面评审 / Generate comprehensive review
  gitai --lang=$lang review --format=markdown > "$LANG_DIR/review-$DATE_FORMAT.md"
  
  # 生成针对安全的评审 / Generate security-focused review
  gitai --lang=$lang review --focus="security" --format=markdown > "$LANG_DIR/security-review-$DATE_FORMAT.md"
  
  # 生成针对性能的评审 / Generate performance-focused review
  gitai --lang=$lang review --focus="performance" --format=markdown > "$LANG_DIR/performance-review-$DATE_FORMAT.md"
  
  # 生成索引页面 / Generate index page
  if [ "$lang" == "zh" ]; then
    INDEX_TITLE="代码分析报告 ($DATE_FORMAT)"
    SCAN_LINK="[代码扫描结果](scan-$DATE_FORMAT.json)"
    REVIEW_LINK="[全面代码评审](review-$DATE_FORMAT.md)"
    SECURITY_LINK="[安全评审](security-review-$DATE_FORMAT.md)"
    PERFORMANCE_LINK="[性能评审](performance-review-$DATE_FORMAT.md)"
  else
    INDEX_TITLE="Code Analysis Reports ($DATE_FORMAT)"
    SCAN_LINK="[Code Scan Results](scan-$DATE_FORMAT.json)"
    REVIEW_LINK="[Comprehensive Code Review](review-$DATE_FORMAT.md)"
    SECURITY_LINK="[Security Review](security-review-$DATE_FORMAT.md)"
    PERFORMANCE_LINK="[Performance Review](performance-review-$DATE_FORMAT.md)"
  fi
  
  cat > "$LANG_DIR/index.md" << EOF
# $INDEX_TITLE

$SCAN_LINK
$REVIEW_LINK
$SECURITY_LINK
$PERFORMANCE_LINK
EOF

done

echo "Documentation generation complete. Reports available in $OUTPUT_DIR directory"
```

### 翻译缓存预热脚本 / Translation Cache Warming Script

```bash
#!/bin/bash
# warm-translation-cache.sh
# 通过预先运行常见命令预热翻译缓存

# 设置语言 / Set languages
LANGUAGES=("zh" "en")

echo "🔄 Warming translation cache..."

# 为每种语言预热缓存 / Warm cache for each language
for lang in "${LANGUAGES[@]}"; do
  echo "Warming $lang cache..."
  
  # 静默模式运行常见命令 / Run common commands in quiet mode
  gitai --lang=$lang scan . --quiet
  gitai --lang=$lang git status --quiet
  gitai --lang=$lang review --focus="common" --quiet
done

echo "✅ Translation cache warming complete"
```

## 5. 配置和优化建议 / Configuration and Optimization Tips

### 最佳实践 / Best Practices

1. **在配置文件中设置默认语言** / **Set default language in config file**
   - 团队成员可以通过 `~/.config/gitai/config.toml` 设置自己的默认语言偏好
   - Team members can set their default language preference via `~/.config/gitai/config.toml`

2. **缓存管理** / **Cache Management**
   - 每周清理一次缓存以避免过时翻译: `rm -rf ~/.cache/gitai/translation/*`
   - Clean cache weekly to avoid stale translations: `rm -rf ~/.cache/gitai/translation/*`

3. **CI/CD 中的翻译优化** / **Translation Optimization in CI/CD**
   - 使用 `--use-cache` 减少 API 调用
   - Use `--use-cache` to reduce API calls
   - 定期更新翻译缓存而不是每次都重新翻译
   - Periodically update translation cache instead of retranslating every time

4. **多语言团队的工作流** / **Workflow for Multilingual Teams**
   - 在代码评审过程中同时提供中英文报告
   - Provide both Chinese and English reports in code review process
   - 使用拉取请求模板包含多语言分析链接
   - Use pull request templates with links to multilingual analysis

### 性能优化 / Performance Optimization

- 翻译 36 个文件的扫描结果仅增加约 5ms 的处理时间
- Translating scan results for 36 files adds only about 5ms processing time
- 使用缓存可以进一步减少翻译开销
- Using cache can further reduce translation overhead
- 定期预热缓存可以提高命令响应速度
- Regularly warming the cache can improve command response time

---

这些工作流示例展示了如何将 GitAI 的翻译功能无缝集成到不同的开发场景中，从个人开发到团队协作再到自动化 CI/CD 流程。通过这些模式，您可以充分利用多语言支持来提高团队效率和沟通效果。

These workflow examples demonstrate how to seamlessly integrate GitAI's translation features into different development scenarios, from individual development to team collaboration to automated CI/CD processes. Using these patterns, you can fully leverage multilingual support to enhance team efficiency and communication effectiveness.