# 用户故事 01: 命令行参数扩展

## 故事概述
**作为一名开发者**
**我希望能够在 `gitai review` 命令中指定 DevOps 工作项 ID**
**这样我就能够将代码评审与具体的需求、任务或缺陷关联起来**

## 详细描述

### 用户角色
- 开发工程师
- 技术负责人
- QA 工程师

### 功能需求
扩展 `gitai review` 命令，支持以下新参数：

1. `--stories=[story_id_1,story_id_2,...]` - 用户故事 ID 列表
2. `--tasks=[task_id_1,task_id_2,...]` - 任务 ID 列表
3. `--defects=[defect_id_1,defect_id_2,...]` - 缺陷 ID 列表
4. `--space-id=space_id` - DevOps 空间/项目 ID（必需参数）

### 使用场景

#### 场景 1: 单个用户故事评审
```bash
gitai review --space-id=726226 --stories=99
```

#### 场景 2: 多个用户故事评审
```bash
gitai review --space-id=726226 --stories=99,100,101
```

#### 场景 3: 混合工作项类型评审
```bash
gitai review --space-id=726226 --stories=99 --tasks=200,201 --defects=301
```

#### 场景 4: 结合现有参数
```bash
gitai review --space-id=726226 --stories=99 --depth=deep --format=json --output=review.json
```

## 验收标准

### 功能验收
- [ ] 命令行能够正确解析 `--stories` 参数及其值列表
- [ ] 命令行能够正确解析 `--tasks` 参数及其值列表  
- [ ] 命令行能够正确解析 `--defects` 参数及其值列表
- [ ] 命令行能够正确解析 `--space-id` 参数
- [ ] 支持工作项 ID 的逗号分隔列表格式
- [ ] 当指定工作项参数时，`--space-id` 为必需参数
- [ ] 新参数与现有参数兼容，不影响原有功能

### 参数验证
- [ ] 当使用 `--stories`、`--tasks` 或 `--defects` 时，必须提供 `--space-id`
- [ ] 缺少 `--space-id` 时显示清晰的错误信息
- [ ] 工作项 ID 必须为数字格式
- [ ] 无效的工作项 ID 格式时显示错误提示
- [ ] 空的工作项列表时显示警告信息

### 错误处理
- [ ] 参数格式错误时提供友好的错误信息
- [ ] 显示参数使用示例
- [ ] 支持 `--help` 显示新参数的说明文档

### 向后兼容性
- [ ] 不影响现有的 `gitai review` 功能
- [ ] 现有参数组合继续正常工作
- [ ] 不破坏现有的配置文件格式

## 技术实现要求

### 代码结构
- 扩展 `ReviewArgs` 结构体，新增字段：
  - `stories: Option<Vec<u32>>`
  - `tasks: Option<Vec<u32>>`  
  - `defects: Option<Vec<u32>>`
  - `space_id: Option<u32>`

### 参数解析
- 使用 `clap` 库扩展参数定义
- 支持逗号分隔的数字列表解析
- 实现自定义验证逻辑

### 验证逻辑
```rust
// 伪代码示例
fn validate_args(args: &ReviewArgs) -> Result<(), ArgError> {
    let has_work_items = args.stories.is_some() 
        || args.tasks.is_some() 
        || args.defects.is_some();
    
    if has_work_items && args.space_id.is_none() {
        return Err(ArgError::MissingSpaceId);
    }
    
    Ok(())
}
```

## 优先级
**高优先级** - 这是 DevOps 集成功能的基础，后续功能都依赖于此。

## 估算工作量
- 开发时间：2-3 天
- 测试时间：1 天
- 文档更新：0.5 天

## 依赖关系
- 无前置依赖
- 后续故事依赖此功能完成

## 测试用例

### 正常场景测试
1. 测试单个故事 ID 参数解析
2. 测试多个故事 ID 参数解析
3. 测试混合工作项类型参数解析
4. 测试与现有参数的组合使用

### 异常场景测试
1. 测试缺少 space-id 的错误处理
2. 测试无效工作项 ID 格式的错误处理
3. 测试空工作项列表的处理
4. 测试参数冲突的处理

### 边界条件测试
1. 测试大量工作项 ID 的处理
2. 测试特殊字符在工作项 ID 中的处理
3. 测试超长参数值的处理

## 完成定义 (Definition of Done)
- [ ] 代码实现完成并通过代码评审
- [ ] 单元测试覆盖率达到 90% 以上
- [ ] 集成测试通过
- [ ] 文档更新完成
- [ ] 功能演示通过产品验收