# 质量度量 (Quality Metrics)

## 功能概述

GitAI 的质量度量功能提供全面的代码质量跟踪和分析，帮助团队监控项目健康状态、识别技术债务、跟踪质量趋势，并生成可视化报告。

## 核心特性

### 1. 多维度度量
- **代码复杂度**：圈复杂度、认知复杂度
- **代码覆盖率**：测试覆盖率、分支覆盖率
- **技术债务**：债务比率、修复时间估算
- **代码重复**：重复率、克隆块检测
- **依赖健康**：过时依赖、安全漏洞
- **架构质量**：模块耦合度、内聚性

### 2. 趋势分析
- 历史数据对比
- 质量趋势图表
- 预警和阈值
- 改进建议

### 3. 报告生成
- HTML 可视化报告
- JSON 数据导出
- CSV 表格导出
- Markdown 文档

### 4. 集成能力
- CI/CD 集成
- Git hooks
- IDE 插件
- Dashboard API

## 使用方法

### 基本用法

```bash
# 记录当前质量快照
gitai metrics record

# 分析质量趋势（最近30天）
gitai metrics analyze --days 30

# 生成质量报告
gitai metrics report

# 导出数据
gitai metrics export --format json

# 比较两个快照
gitai metrics compare --from 2024-01-01 --to 2024-01-31

# 列出所有快照
gitai metrics list
```

### 高级用法

```bash
# 记录特定类型的度量
gitai metrics record --type complexity,coverage,security

# 生成 HTML 报告
gitai metrics report --format html --output report.html

# 设置质量阈值
gitai metrics analyze --threshold-complexity 10 --threshold-coverage 80

# 清理旧数据
gitai metrics clean --older-than 90

# 导入外部度量数据
gitai metrics import --source sonarqube --file metrics.json
```

## 配置选项

在 `~/.config/gitai/config.toml` 中配置：

```toml
[metrics]
# 自动记录间隔（小时）
auto_record_interval = 24

# 数据保留期限（天）
retention_days = 90

# 默认分析天数
default_analysis_days = 30

[metrics.thresholds]
# 质量阈值
max_complexity = 10
min_coverage = 80
max_duplication = 5
max_tech_debt_ratio = 5

[metrics.types]
# 启用的度量类型
enabled = [
    "complexity",
    "coverage",
    "duplication",
    "security",
    "dependencies",
    "architecture"
]

[metrics.report]
# 报告配置
default_format = "html"
include_charts = true
include_recommendations = true
```

## 度量指标详解

### 1. 代码复杂度

**圈复杂度 (Cyclomatic Complexity)**
- 衡量代码中独立路径的数量
- 建议值：≤ 10
- 计算公式：M = E - N + 2P

**认知复杂度 (Cognitive Complexity)**
- 衡量代码理解难度
- 考虑嵌套深度和控制流
- 建议值：≤ 15

### 2. 测试覆盖率

**行覆盖率**
- 被测试执行的代码行百分比
- 目标值：≥ 80%

**分支覆盖率**
- 被测试的条件分支百分比
- 目标值：≥ 70%

### 3. 代码重复

**重复率**
- 重复代码占总代码的百分比
- 目标值：< 5%

**克隆类型**
- Type-1：完全相同
- Type-2：参数化相同
- Type-3：结构相似

### 4. 技术债务

**债务比率**
- 修复成本 / 开发成本
- 目标值：< 5%

**债务分类**
- 🔴 **阻塞**：必须立即修复
- 🟡 **关键**：短期内修复
- 🔵 **主要**：计划修复
- ⚪ **次要**：可选修复

## 工作流程

### 1. 数据采集
```
代码扫描 → 度量计算 → 数据聚合 → 快照保存
```

### 2. 趋势分析
```
历史数据加载 → 时间序列分析 → 趋势识别 → 预警生成
```

### 3. 报告生成
```
数据查询 → 图表生成 → 模板渲染 → 报告输出
```

## 示例场景

### 场景 1：每日质量跟踪

```bash
# 每天早上记录质量快照
gitai metrics record

# 输出示例：
📊 正在分析代码质量...
✅ 质量快照已记录

质量概览：
- 代码行数：45,678
- 圈复杂度：平均 6.5（良好）
- 测试覆盖率：82.3%（优秀）
- 代码重复率：3.2%（良好）
- 技术债务：2.1%（低）
- 安全问题：0 个高危，2 个中危

与昨天相比：
- 覆盖率 +1.2% ↑
- 复杂度 -0.3 ↓
- 技术债务 -0.1% ↓

整体质量：A 级（优秀）
```

### 场景 2：月度质量报告

```bash
gitai metrics report --format html --days 30

# 输出示例：
📈 生成月度质量报告...
✅ 报告已生成：quality-report-2024-01.html

报告摘要：
- 分析周期：2024-01-01 至 2024-01-31
- 质量趋势：稳步改善
- 关键成就：
  * 测试覆盖率提升 5%
  * 消除 3 个高复杂度模块
  * 修复 15 个安全问题
- 需要关注：
  * src/parser 模块复杂度偏高
  * 依赖包 lodash 存在安全漏洞
  * 代码重复率有上升趋势
```

### 场景 3：CI/CD 质量门禁

```yaml
# .github/workflows/quality.yml
name: Quality Gate

on: [push, pull_request]

jobs:
  quality-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install GitAI
        run: cargo install gitai
      
      - name: Record Metrics
        run: gitai metrics record
      
      - name: Check Quality Gates
        run: |
          gitai metrics analyze --threshold-complexity 10 \
                              --threshold-coverage 80 \
                              --threshold-duplication 5
      
      - name: Upload Report
        if: always()
        uses: actions/upload-artifact@v2
        with:
          name: quality-report
          path: metrics-report.html
```

## 可视化报告

### HTML 报告内容

1. **执行摘要**
   - 质量评级（A-F）
   - 关键指标仪表盘
   - 趋势箭头

2. **详细分析**
   - 复杂度热图
   - 覆盖率分布
   - 技术债务清单

3. **趋势图表**
   - 时间序列图
   - 对比柱状图
   - 饼图分布

4. **改进建议**
   - 优先级列表
   - 估算工作量
   - 最佳实践

### 示例报告结构

```html
<!DOCTYPE html>
<html>
<head>
    <title>GitAI 质量报告</title>
    <style>
        /* 美观的样式 */
    </style>
</head>
<body>
    <h1>项目质量报告</h1>
    
    <section class="summary">
        <div class="grade">A</div>
        <div class="metrics">
            <div class="metric">
                <span>复杂度</span>
                <span>6.5</span>
            </div>
            <!-- 更多指标 -->
        </div>
    </section>
    
    <section class="trends">
        <canvas id="trend-chart"></canvas>
    </section>
    
    <section class="recommendations">
        <ul>
            <li>降低 parser.rs 的复杂度</li>
            <li>增加 auth 模块的测试覆盖</li>
        </ul>
    </section>
</body>
</html>
```

## 数据导出格式

### JSON 格式

```json
{
  "timestamp": "2024-01-31T10:00:00Z",
  "project": "gitai",
  "metrics": {
    "complexity": {
      "average": 6.5,
      "max": 23,
      "distribution": {
        "low": 78,
        "medium": 18,
        "high": 4
      }
    },
    "coverage": {
      "line": 82.3,
      "branch": 75.6,
      "function": 88.9
    },
    "duplication": {
      "percentage": 3.2,
      "blocks": 15
    }
  },
  "trends": {
    "complexity": -0.3,
    "coverage": +1.2,
    "duplication": +0.1
  }
}
```

### CSV 格式

```csv
Date,Complexity,Coverage,Duplication,TechDebt,Security
2024-01-01,6.8,81.1,3.1,2.2,3
2024-01-02,6.7,81.5,3.1,2.1,2
2024-01-03,6.5,82.3,3.2,2.1,0
```

## 与其他功能集成

### 代码评审集成

```bash
# 评审时包含质量度量
gitai review --with-metrics
```

### 提交时记录

```bash
# 提交后自动记录度量
gitai commit && gitai metrics record
```

### 扫描集成

```bash
# 安全扫描后更新度量
gitai scan && gitai metrics record --type security
```

## 故障排除

### 问题：度量数据不准确

**解决方案：**
1. 确保代码已完全构建
2. 运行测试生成覆盖率数据
3. 检查配置的度量工具
4. 清理缓存重新计算

### 问题：报告生成失败

**解决方案：**
1. 检查数据目录权限
2. 确保有足够的历史数据
3. 验证模板文件完整
4. 查看错误日志

### 问题：趋势分析异常

**解决方案：**
1. 检查时间范围设置
2. 验证数据连续性
3. 排除异常数据点
4. 重新计算基线

## 最佳实践

### 1. 定期记录
- 每日自动记录
- 重要版本前后记录
- 代码审查后记录

### 2. 设置合理阈值
- 根据项目特点调整
- 逐步提高标准
- 区分关键和非关键代码

### 3. 行动跟进
- 定期审查报告
- 制定改进计划
- 跟踪改进效果

### 4. 团队协作
- 共享质量目标
- 定期质量回顾
- 庆祝质量改进

## 高级功能

### 自定义度量

```rust
// 实现自定义度量
pub struct CustomMetric {
    name: String,
    calculate: Box<dyn Fn(&CodeBase) -> f64>,
}

impl CustomMetric {
    pub fn new(name: &str, calculator: impl Fn(&CodeBase) -> f64 + 'static) -> Self {
        Self {
            name: name.to_string(),
            calculate: Box::new(calculator),
        }
    }
}
```

### 度量插件

```toml
# 配置插件
[metrics.plugins]
enabled = ["sonarqube", "codecov", "custom"]

[metrics.plugins.sonarqube]
url = "https://sonar.example.com"
token = "xxx"
```

## 未来展望

- [ ] 实时度量监控
- [ ] AI 驱动的质量预测
- [ ] 自动化质量改进建议
- [ ] 团队绩效仪表板
- [ ] 跨项目质量对比
