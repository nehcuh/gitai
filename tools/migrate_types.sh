#!/bin/bash

# GitAI 类型系统迁移脚本
# 用于将旧的类型定义迁移到统一的 gitai-types crate

set -e

echo "🚀 开始 GitAI 类型系统迁移..."

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 备份当前代码
echo "📦 创建备份..."
if [ ! -d "backup" ]; then
    mkdir -p backup
    cp -r src backup/src_$(date +%Y%m%d_%H%M%S)
    echo -e "${GREEN}✓ 备份完成${NC}"
else
    echo -e "${YELLOW}⚠ 备份目录已存在，跳过备份${NC}"
fi

# 创建类型映射文件
cat > type_mappings.txt << 'EOF'
# 类型映射规则
# 格式: old_path::Type -> gitai_types::Type

# Severity 映射
src/scan.rs::Severity -> gitai_types::Severity
src/review.rs::Severity -> gitai_types::Severity
src/project_insights.rs::Severity -> gitai_types::Severity
src/metrics/mod.rs::Severity -> gitai_types::Severity
src/mcp/services/scan.rs::Severity -> gitai_types::Severity
src/mcp/services/review.rs::Severity -> gitai_types::Severity
src/security_insights.rs::Severity -> gitai_types::Severity

# RiskLevel 映射
src/architectural_impact/mod.rs::RiskLevel -> gitai_types::RiskLevel
src/project_insights.rs::RiskLevel -> gitai_types::RiskLevel

# Finding 映射
src/scan.rs::Finding -> gitai_types::Finding
src/review.rs::Finding -> gitai_types::Finding
src/mcp/services/scan.rs::Finding -> gitai_types::Finding
src/mcp/services/review.rs::Finding -> gitai_types::Finding

# BreakingChange 映射
src/architectural_impact/mod.rs::BreakingChange -> gitai_types::BreakingChange
src/project_insights.rs::BreakingChange -> gitai_types::BreakingChange
src/architectural_impact/cascade_detector.rs::BreakingChange -> gitai_types::BreakingChange

# ImpactLevel 映射
src/architectural_impact/mod.rs::ImpactLevel -> gitai_types::ImpactLevel
src/project_insights.rs::ImpactLevel -> gitai_types::ImpactLevel

# NodeType 映射
src/project_insights.rs::NodeType -> gitai_types::NodeType

# DependencyType 映射
src/project_insights.rs::DependencyType -> gitai_types::DependencyType

# BreakingChangeType 映射
src/architectural_impact/mod.rs::BreakingChangeType -> gitai_types::BreakingChangeType
src/project_insights.rs::BreakingChangeType -> gitai_types::BreakingChangeType
EOF

echo "📝 生成迁移计划..."

# 分析需要修改的文件
FILES_TO_MODIFY=(
    "src/scan.rs"
    "src/review.rs"
    "src/architectural_impact/mod.rs"
    "src/architectural_impact/cascade_detector.rs"
    "src/project_insights.rs"
    "src/metrics/mod.rs"
    "src/mcp/services/scan.rs"
    "src/mcp/services/review.rs"
    "src/security_insights.rs"
    "src/analysis.rs"
    "src/context.rs"
)

echo -e "${YELLOW}将要修改以下文件:${NC}"
for file in "${FILES_TO_MODIFY[@]}"; do
    if [ -f "$file" ]; then
        echo "  - $file"
    fi
done

# 创建迁移报告
echo "📊 创建迁移报告..."
cat > migration_report.md << 'EOF'
# GitAI 类型系统迁移报告

## 迁移时间
$(date)

## 迁移目标
将分散在各个模块中的类型定义统一迁移到 `gitai-types` crate

## 主要变更

### 1. 删除的类型定义
以下类型定义将从各自的模块中删除，改用 `gitai-types` 中的统一定义：

- `Severity` - 从 7 个文件中删除
- `RiskLevel` - 从 2 个文件中删除  
- `Finding` - 从 4 个文件中删除
- `BreakingChange` - 从 3 个文件中删除
- `ImpactLevel` - 从 2 个文件中删除
- `NodeType` - 从 1 个文件中删除
- `DependencyType` - 从 1 个文件中删除
- `BreakingChangeType` - 从 2 个文件中删除

### 2. 添加的导入
所有受影响的文件将添加：
```rust
use gitai_types::{
    Severity, RiskLevel, Finding, BreakingChange,
    ImpactLevel, NodeType, DependencyType, BreakingChangeType,
    // 其他需要的类型...
};
```

### 3. 需要手动处理的部分

#### 转换函数
某些模块可能有自定义的转换函数，需要手动检查和调整：
- [ ] scan.rs 中的 Severity 转换
- [ ] architectural_impact 中的 RiskLevel 计算
- [ ] project_insights 中的影响分析

#### 序列化/反序列化
检查所有 JSON 序列化是否兼容：
- [ ] API 响应格式
- [ ] 缓存文件格式
- [ ] 配置文件格式

#### 测试更新
- [ ] 单元测试
- [ ] 集成测试
- [ ] 端到端测试

## 回滚计划
如果迁移出现问题，可以从 backup/ 目录恢复原始代码：
```bash
cp -r backup/src_[timestamp]/* src/
```

## 验证步骤
1. 编译检查: `cargo check`
2. 运行测试: `cargo test`
3. 格式检查: `cargo fmt --check`
4. Lint 检查: `cargo clippy`

EOF

echo -e "${GREEN}✓ 迁移报告已生成: migration_report.md${NC}"

# 提示下一步操作
echo ""
echo "📌 下一步操作:"
echo "1. 查看 migration_report.md 了解迁移详情"
echo "2. 运行以下命令开始实际迁移:"
echo "   ./scripts/apply_type_migration.sh"
echo "3. 迁移后运行测试验证:"
echo "   cargo test --all-features"
echo ""
echo -e "${GREEN}✅ 迁移准备完成！${NC}"
