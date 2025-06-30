# 🌐 GitAI 翻译功能故障排除指南 / GitAI Translation Troubleshooting Guide

本文档提供了针对 GitAI 翻译功能的详细故障排除步骤和最佳实践，帮助您诊断和解决在使用多语言功能时可能遇到的问题。

This document provides detailed troubleshooting steps and best practices for GitAI translation features, helping you diagnose and resolve issues you may encounter when using multilingual functionality.

## 📋 目录 / Table of Contents

1. [常见问题 / Common Issues](#常见问题--common-issues)
2. [诊断工具 / Diagnostic Tools](#诊断工具--diagnostic-tools)
3. [配置问题 / Configuration Issues](#配置问题--configuration-issues)
4. [API 连接问题 / API Connection Issues](#api-连接问题--api-connection-issues)
5. [缓存管理 / Cache Management](#缓存管理--cache-management)
6. [性能优化 / Performance Optimization](#性能优化--performance-optimization)
7. [高级故障排除 / Advanced Troubleshooting](#高级故障排除--advanced-troubleshooting)

## 常见问题 / Common Issues

### 翻译功能未生效 / Translation Not Working

**症状 / Symptoms**:
- 命令输出仍然是默认语言（通常是英文）
- 设置 `--lang` 参数但没有效果
- 没有错误消息，但翻译未应用

**解决方案 / Solutions**:

1. **检查翻译功能是否启用 / Check if translation is enabled**
   ```bash
   # 查看配置文件 / Check configuration file
   grep "enabled" ~/.config/gitai/config.toml
   
   # 应该显示 / Should show: enabled = true
   ```

2. **验证语言参数 / Verify language parameter**
   ```bash
   # 明确指定语言 / Explicitly specify language
   gitai --lang=zh scan src/
   
   # 检查环境变量 / Check environment variable
   echo $GITAI_TRANSLATION_LANGUAGE
   ```

3. **重置翻译设置 / Reset translation settings**
   ```bash
   # 创建默认配置 / Create default configuration
   mkdir -p ~/.config/gitai
   cat > ~/.config/gitai/config.toml << EOF
   [translation]
   enabled = true
   default_language = "zh"
   cache_enabled = true
   provider = "openai"
   cache_dir = "~/.cache/gitai/translation"
   
   [translation.provider_settings]
   api_key = "your-api-key-here"
   model = "gpt-3.5-turbo"
   EOF
   ```

### 翻译质量问题 / Translation Quality Issues

**症状 / Symptoms**:
- 翻译结果不准确或不一致
- 专业术语翻译不正确
- 某些内容未被翻译

**解决方案 / Solutions**:

1. **使用更高质量的模型 / Use higher quality model**
   ```toml
   [translation.provider_settings]
   model = "gpt-4"  # 替换为更高质量的模型 / Replace with higher quality model
   ```

2. **清除翻译缓存 / Clear translation cache**
   ```bash
   rm -rf ~/.cache/gitai/translation/*
   ```

3. **使用自定义翻译提供商 / Use custom translation provider**
   ```toml
   [translation]
   provider = "custom"
   
   [translation.provider_settings]
   endpoint = "https://your-custom-translation-api.example.com/v1/translate"
   api_key = "your-custom-api-key"
   ```

## 诊断工具 / Diagnostic Tools

### 启用调试日志 / Enable Debug Logging

```bash
# 设置环境变量启用详细日志 / Set environment variables for verbose logging
export GITAI_DEBUG=true
export GITAI_TRANSLATION_DEBUG=true

# 运行命令查看详细日志 / Run command with verbose logging
gitai --lang=zh scan src/
```

### 追踪翻译请求 / Trace Translation Requests

```bash
# 显示翻译API请求和响应 / Show translation API requests and responses
gitai --lang=zh scan src/ --trace-translation

# 输出翻译性能数据 / Output translation performance data
gitai scan --lang=zh src/ --translation-perf-stats
```

### 检查翻译状态 / Check Translation Status

```bash
# 查看缓存统计 / View cache statistics
gitai scan --lang=zh src/ --translation-cache-info

# 检查支持的语言 / Check supported languages
gitai translation-status
```

## 配置问题 / Configuration Issues

### 配置文件位置 / Configuration File Location

默认情况下，GitAI 配置文件位于：
By default, GitAI configuration file is located at:
- `~/.config/gitai/config.toml`

如果使用自定义配置文件，可以通过环境变量指定：
If using a custom configuration file, you can specify via environment variable:
```bash
export GITAI_CONFIG_PATH=/path/to/custom/config.toml
```

### 常见配置错误 / Common Configuration Errors

1. **缺少 API 密钥 / Missing API Key**
   ```toml
   [translation.provider_settings]
   # 错误：缺少 api_key / Error: missing api_key
   model = "gpt-3.5-turbo"
   ```
   
   **修复 / Fix**:
   ```toml
   [translation.provider_settings]
   api_key = "your-api-key-here"  # 添加 API 密钥 / Add API key
   model = "gpt-3.5-turbo"
   ```

2. **无效的默认语言 / Invalid Default Language**
   ```toml
   [translation]
   default_language = "fr"  # 错误：不支持的语言 / Error: unsupported language
   ```
   
   **修复 / Fix**:
   ```toml
   [translation]
   default_language = "zh"  # 修改为支持的语言：zh|en|auto / Change to supported language: zh|en|auto
   ```

3. **无效的缓存目录 / Invalid Cache Directory**
   ```toml
   [translation]
   cache_dir = "/root/cache"  # 错误：可能没有写入权限 / Error: might not have write permission
   ```
   
   **修复 / Fix**:
   ```toml
   [translation]
   cache_dir = "~/.cache/gitai/translation"  # 修改为用户可写目录 / Change to user-writable directory
   ```

## API 连接问题 / API Connection Issues

### 连接超时 / Connection Timeout

**症状 / Symptoms**:
- 命令执行缓慢
- 出现超时错误消息
- 翻译功能间歇性失效

**解决方案 / Solutions**:

1. **设置更长的超时时间 / Set longer timeout**
   ```toml
   [translation.provider_settings]
   timeout_seconds = 30  # 增加超时时间 / Increase timeout
   ```

2. **检查网络连接 / Check network connection**
   ```bash
   # 测试与API服务器的连接 / Test connection to API server
   curl -I https://api.openai.com
   ```

3. **使用代理 / Use proxy**
   ```bash
   # 设置代理环境变量 / Set proxy environment variables
   export HTTP_PROXY=http://proxy.example.com:8080
   export HTTPS_PROXY=http://proxy.example.com:8080
   ```

### API 密钥问题 / API Key Issues

**症状 / Symptoms**:
- 出现 "Invalid API key" 或类似错误
- 翻译请求被拒绝
- 收到授权失败消息

**解决方案 / Solutions**:

1. **验证 API 密钥 / Verify API key**
   ```bash
   # 使用 curl 测试 API 密钥 / Test API key using curl
   curl -s -H "Authorization: Bearer YOUR_API_KEY" https://api.openai.com/v1/models
   ```

2. **检查环境变量 / Check environment variable**
   ```bash
   echo $GITAI_TRANSLATION_API_KEY
   ```

3. **检查 API 密钥权限和限制 / Check API key permissions and limits**
   - 确认 API 密钥有足够的使用配额
   - 确认 API 密钥有权限访问所需的模型
   - 检查 API 密钥是否已过期

## 缓存管理 / Cache Management

### 清理缓存 / Clean Cache

```bash
# 清除所有翻译缓存 / Clear all translation cache
rm -rf ~/.cache/gitai/translation/*

# 创建缓存目录（如果不存在）/ Create cache directory (if not exists)
mkdir -p ~/.cache/gitai/translation
```

### 缓存问题诊断 / Cache Issue Diagnosis

**症状 / Symptoms**:
- 翻译结果不一致
- 更新后的内容仍显示旧翻译
- 缓存相关错误消息

**解决方案 / Solutions**:

1. **禁用缓存进行测试 / Disable cache for testing**
   ```toml
   [translation]
   cache_enabled = false
   ```

2. **强制刷新翻译 / Force refresh translations**
   ```bash
   gitai scan --lang=zh src/ --force-scan
   ```

3. **检查缓存目录权限 / Check cache directory permissions**
   ```bash
   ls -la ~/.cache/gitai/
   # 确保目录存在且用户有读写权限
   # Ensure directory exists and user has read/write permissions
   ```

## 性能优化 / Performance Optimization

### 减少翻译延迟 / Reduce Translation Latency

1. **优化缓存配置 / Optimize cache configuration**
   ```toml
   [translation]
   cache_enabled = true
   cache_max_age_days = 30
   ```

2. **使用更快的翻译模型 / Use faster translation model**
   ```toml
   [translation.provider_settings]
   model = "gpt-3.5-turbo"  # 通常比 GPT-4 更快 / Usually faster than GPT-4
   ```

3. **预热翻译缓存 / Warm up translation cache**
   ```bash
   # 在后台预热缓存 / Warm up cache in background
   gitai scan --lang=zh src/ --quiet &
   gitai scan --lang=en src/ --quiet &
   ```

### 减少 API 调用 / Reduce API Calls

1. **批量处理翻译 / Batch process translations**
   ```bash
   # 一次性处理多个文件而不是单独处理 / Process multiple files at once instead of individually
   gitai scan --lang=zh src/ --use-cache
   ```

2. **限制翻译范围 / Limit translation scope**
   ```bash
   # 只翻译关键结果 / Only translate key results
   gitai scan --lang=zh src/ --max-issues=10 --only-critical
   ```

## 高级故障排除 / Advanced Troubleshooting

### 自定义翻译提供商集成 / Custom Translation Provider Integration

如果您使用自定义翻译 API，确保您的 API 符合 GitAI 预期的格式：
If you're using a custom translation API, ensure your API conforms to the format GitAI expects:

```json
// 请求格式 / Request format
{
  "source_text": "Text to translate",
  "source_language": "en",
  "target_language": "zh"
}

// 期望的响应格式 / Expected response format
{
  "translated_text": "翻译后的文本",
  "status": "success"
}
```

### 调试翻译质量 / Debug Translation Quality

对于特定术语的翻译问题，可以创建术语表：
For translation issues with specific terms, you can create a glossary:

```toml
[translation.glossary]
"code review" = "代码评审"
"performance issue" = "性能问题"
"security vulnerability" = "安全漏洞"
```

### 创建翻译问题报告 / Create Translation Issue Report

如果遇到无法解决的翻译问题，请收集以下信息并创建详细的问题报告：
If you encounter translation issues you cannot resolve, collect the following information and create a detailed issue report:

1. GitAI 版本 / GitAI version
2. 操作系统和环境 / OS and environment
3. 完整配置文件（删除敏感信息）/ Complete configuration file (redact sensitive information)
4. 调试日志输出 / Debug log output
5. 重现步骤 / Steps to reproduce
6. 实际与预期结果 / Actual vs expected results

---

## 快速参考 / Quick Reference

### 常用故障排除命令 / Common Troubleshooting Commands

```bash
# 诊断翻译问题 / Diagnose translation issues
export GITAI_DEBUG=true
gitai --lang=zh scan src/ --trace-translation

# 检查翻译配置 / Check translation configuration
cat ~/.config/gitai/config.toml | grep -A 10 "translation"

# 强制刷新翻译 / Force refresh translations
gitai scan --lang=zh src/ --force-scan

# 清理翻译缓存 / Clean translation cache
rm -rf ~/.cache/gitai/translation/*
mkdir -p ~/.cache/gitai/translation

# 测试API连接 / Test API connection
curl -s -H "Authorization: Bearer $GITAI_TRANSLATION_API_KEY" https://api.openai.com/v1/models
```

### 环境变量快速参考 / Environment Variables Quick Reference

- `GITAI_TRANSLATION_ENABLED`: 启用/禁用翻译 (true/false)
- `GITAI_TRANSLATION_LANGUAGE`: 默认语言 (zh/en/auto)
- `GITAI_TRANSLATION_API_KEY`: 翻译 API 密钥
- `GITAI_TRANSLATION_CACHE_DIR`: 翻译缓存目录
- `GITAI_DEBUG`: 启用调试模式 (true/false)
- `GITAI_TRANSLATION_DEBUG`: 启用翻译调试 (true/false)
- `GITAI_CONFIG_PATH`: 自定义配置文件路径

---

如果您在解决翻译问题后有任何改进建议，欢迎提交反馈或贡献更新到本故障排除指南。

If you have any suggestions for improvements after resolving translation issues, please submit feedback or contribute updates to this troubleshooting guide.