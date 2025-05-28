# 用户故事 02: 配置管理集成

## 故事概述
**作为一名开发者**
**我希望能够在配置文件中管理 DevOps 平台的认证信息**
**这样我就能够安全地存储和使用 API 访问凭据，而无需在命令行中暴露敏感信息**

## 详细描述

### 用户角色
- 开发工程师
- 技术负责人
- DevOps 工程师

### 功能需求
扩展 gitai 配置管理系统，支持 DevOps 平台集成配置：

1. 在配置文件中新增 `account` section
2. 支持多种 DevOps 平台配置
3. 安全存储认证 token
4. 支持环境变量覆盖配置
5. 提供配置验证机制

### 配置文件格式

#### 基本配置结构
```toml
[account]
# DevOps 平台配置
devops_platform = "coding"  # 支持 coding, jira, azure-devops 等
base_url = "https://codingcorp.devops.xxx.com.cn"
token = "c4e9e27573a4437d6ecad5119b2ebe026f5fdbc8"

# 可选：备用配置
[account.backup]
devops_platform = "jira"
base_url = "https://company.atlassian.net"
token = "ATATT3xFfGF0T..."
```

#### 多环境配置支持
```toml
[account.development]
devops_platform = "coding"
base_url = "https://dev.devops.company.com"
token = "dev_token_here"

[account.production]
devops_platform = "coding"
base_url = "https://prod.devops.company.com"
token = "prod_token_here"
```

### 使用场景

#### 场景 1: 首次配置
```bash
# 用户编辑配置文件
~/.config/gitai/config.toml

# 或使用命令初始化配置
gitai config set account.devops_platform coding
gitai config set account.base_url "https://company.devops.com"
gitai config set account.token "your_token_here"
```

#### 场景 2: 环境变量覆盖
```bash
# 临时使用不同的配置
export GITAI_DEVOPS_TOKEN="temporary_token"
export GITAI_DEVOPS_BASE_URL="https://test.devops.com"
gitai review --space-id=726226 --stories=99
```

#### 场景 3: 配置验证
```bash
# 验证配置有效性
gitai config validate

# 测试 DevOps 连接
gitai config test-connection
```

## 验收标准

### 配置文件管理
- [ ] 支持在配置文件中定义 `account` section
- [ ] 正确解析 DevOps 平台配置参数
- [ ] 支持必需配置项验证
- [ ] 支持可选配置项的默认值
- [ ] 配置文件格式验证

### 认证管理
- [ ] 安全存储 API token
- [ ] 支持 token 有效性验证
- [ ] 支持多种认证方式（token, OAuth, API key）
- [ ] 不在日志中暴露敏感信息
- [ ] 支持 token 过期检测

### 环境变量支持
- [ ] 支持通过环境变量覆盖配置
- [ ] 环境变量优先级高于配置文件
- [ ] 支持的环境变量：
  - `GITAI_DEVOPS_PLATFORM`
  - `GITAI_DEVOPS_BASE_URL`
  - `GITAI_DEVOPS_TOKEN`

### 多平台支持
- [ ] 抽象 DevOps 平台接口
- [ ] 支持 Coding 平台配置
- [ ] 为 JIRA、Azure DevOps 等平台预留扩展点
- [ ] 平台特定配置参数支持

### 错误处理
- [ ] 配置文件不存在时的友好提示
- [ ] 配置格式错误时的详细错误信息
- [ ] token 无效时的错误提示
- [ ] 网络连接失败时的错误处理

## 技术实现要求

### 配置结构定义
```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct AccountConfig {
    pub devops_platform: String,
    pub base_url: String,
    pub token: String,
    pub timeout: Option<u64>,
    pub retry_count: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    // 现有配置...
    pub account: Option<AccountConfig>,
}
```

### 环境变量处理
```rust
impl AccountConfig {
    pub fn from_env_or_config(config: Option<AccountConfig>) -> Result<AccountConfig, ConfigError> {
        let platform = env::var("GITAI_DEVOPS_PLATFORM")
            .or_else(|_| config.as_ref().map(|c| c.devops_platform.clone()).ok_or_else(|| ConfigError::Missing("devops_platform".to_string())))?;
        
        let base_url = env::var("GITAI_DEVOPS_BASE_URL")
            .or_else(|_| config.as_ref().map(|c| c.base_url.clone()).ok_or_else(|| ConfigError::Missing("base_url".to_string())))?;
        
        let token = env::var("GITAI_DEVOPS_TOKEN")
            .or_else(|_| config.as_ref().map(|c| c.token.clone()).ok_or_else(|| ConfigError::Missing("token".to_string())))?;
        
        Ok(AccountConfig {
            devops_platform: platform,
            base_url,
            token,
            timeout: config.as_ref().and_then(|c| c.timeout),
            retry_count: config.as_ref().and_then(|c| c.retry_count),
        })
    }
}
```

### 配置验证
```rust
impl AccountConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // 验证平台支持
        match self.devops_platform.as_str() {
            "coding" | "jira" | "azure-devops" => {},
            _ => return Err(ConfigError::UnsupportedPlatform(self.devops_platform.clone())),
        }
        
        // 验证 URL 格式
        if !self.base_url.starts_with("http") {
            return Err(ConfigError::InvalidUrl(self.base_url.clone()));
        }
        
        // 验证 token 格式
        if self.token.is_empty() {
            return Err(ConfigError::EmptyToken);
        }
        
        Ok(())
    }
}
```

## 安全要求

### 敏感信息保护
- [ ] token 不出现在日志输出中
- [ ] 配置文件权限限制（600）
- [ ] 支持加密存储敏感配置
- [ ] 内存中敏感数据及时清理

### 最佳实践
- [ ] 提供配置示例和文档
- [ ] 警告用户不要将 token 提交到代码仓库
- [ ] 支持从外部密钥管理系统读取 token
- [ ] 提供安全配置检查工具

## 优先级
**高优先级** - DevOps API 调用的基础配置，必须在 API 集成之前完成。

## 估算工作量
- 配置结构设计：1 天
- 配置读取和验证：2 天
- 环境变量支持：1 天
- 错误处理和测试：1 天
- 文档编写：0.5 天

## 依赖关系
- 依赖：用户故事 01 (命令行参数扩展)
- 被依赖：用户故事 03 (DevOps API 集成)

## 测试用例

### 配置文件测试
1. 测试完整配置文件解析
2. 测试部分配置文件解析
3. 测试配置文件格式错误处理
4. 测试缺失配置项错误处理

### 环境变量测试
1. 测试环境变量覆盖配置文件
2. 测试部分环境变量覆盖
3. 测试环境变量格式验证
4. 测试环境变量优先级

### 验证测试
1. 测试支持的平台验证
2. 测试 URL 格式验证
3. 测试 token 格式验证
4. 测试网络连接验证

### 安全测试
1. 测试敏感信息不泄露到日志
2. 测试配置文件权限
3. 测试内存敏感数据清理
4. 测试配置文件加密存储

## 完成定义 (Definition of Done)
- [ ] 代码实现完成并通过代码评审
- [ ] 单元测试覆盖率达到 90% 以上
- [ ] 集成测试通过
- [ ] 安全测试通过
- [ ] 配置文档更新完成
- [ ] 功能演示通过产品验收