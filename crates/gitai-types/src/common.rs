//! 通用领域实体和值对象

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// 文件路径值对象
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FilePath {
    path: String,
    absolute_path: String,
    extension: Option<String>,
    file_name: Option<String>,
    directory: String,
}

impl FilePath {
    /// 从字符串创建文件路径
    pub fn new(path: impl Into<String>) -> Result<Self, String> {
        let path_str = path.into();
        let path_buf = std::path::PathBuf::from(&path_str);

        // 验证路径格式
        if path_str.contains("..") {
            return Err("Path cannot contain '..'".to_string());
        }

        let absolute_path = path_buf
            .canonicalize()
            .map_err(|e| format!("Failed to canonicalize path: {e}"))?
            .to_string_lossy()
            .to_string();

        let extension = path_buf
            .extension()
            .map(|ext| ext.to_string_lossy().to_string());

        let file_name = path_buf
            .file_name()
            .map(|name| name.to_string_lossy().to_string());

        let directory = path_buf
            .parent()
            .map(|dir| dir.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string());

        Ok(Self {
            path: path_str,
            absolute_path,
            extension,
            file_name,
            directory,
        })
    }

    /// 获取原始路径
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 获取绝对路径
    pub fn absolute_path(&self) -> &str {
        &self.absolute_path
    }

    /// 获取文件扩展名
    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }

    /// 获取文件名
    pub fn file_name(&self) -> Option<&str> {
        self.file_name.as_deref()
    }

    /// 获取目录路径
    pub fn directory(&self) -> &str {
        &self.directory
    }

    /// 检查是否是代码文件
    pub fn is_code_file(&self) -> bool {
        matches!(
            self.extension.as_deref(),
            Some(
                "rs" | "java"
                    | "py"
                    | "js"
                    | "ts"
                    | "go"
                    | "c"
                    | "cpp"
                    | "h"
                    | "hpp"
                    | "scala"
                    | "kt"
                    | "swift"
                    | "php"
                    | "rb"
                    | "cs"
                    | "fs"
                    | "ml"
            )
        )
    }

    /// 检查是否是测试文件
    pub fn is_test_file(&self) -> bool {
        if let Some(file_name) = &self.file_name {
            file_name.contains("test") || file_name.contains("spec")
        } else {
            false
        }
    }

    /// 检查是否是配置文件
    pub fn is_config_file(&self) -> bool {
        matches!(
            self.extension.as_deref(),
            Some("toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "conf" | "properties")
        )
    }

    /// 转换为std::path::PathBuf
    pub fn to_path_buf(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(&self.absolute_path)
    }
}

impl fmt::Display for FilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl std::str::FromStr for FilePath {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// 代码语言枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgrammingLanguage {
    /// Rust 语言
    Rust,
    /// Java 语言
    Java,
    /// Python 语言
    Python,
    /// JavaScript 语言
    JavaScript,
    /// TypeScript 语言
    TypeScript,
    /// Go 语言
    Go,
    /// C 语言
    C,
    /// C++ 语言
    Cpp,
    /// C# 语言
    CSharp,
    /// Scala 语言
    Scala,
    /// Kotlin 语言
    Kotlin,
    /// Swift 语言
    Swift,
    /// PHP 语言
    Php,
    /// Ruby 语言
    Ruby,
    /// F# 语言
    FSharp,
    /// OCaml 语言
    OCaml,
    /// Haskell 语言
    Haskell,
    /// 未知语言
    Unknown,
}

impl ProgrammingLanguage {
    /// 从文件扩展名识别语言
    pub fn from_extension(extension: &str) -> Self {
        match extension.to_lowercase().as_str() {
            "rs" => Self::Rust,
            "java" => Self::Java,
            "py" | "pyx" | "pyi" => Self::Python,
            "js" | "jsx" => Self::JavaScript,
            "ts" | "tsx" => Self::TypeScript,
            "go" => Self::Go,
            "c" => Self::C,
            "cpp" | "cc" | "cxx" | "c++" => Self::Cpp,
            "cs" => Self::CSharp,
            "scala" => Self::Scala,
            "kt" | "kts" => Self::Kotlin,
            "swift" => Self::Swift,
            "php" => Self::Php,
            "rb" => Self::Ruby,
            "fs" | "fsx" | "fsi" => Self::FSharp,
            "ml" | "mli" => Self::OCaml,
            "hs" | "lhs" => Self::Haskell,
            _ => Self::Unknown,
        }
    }

    /// 从文件名识别语言
    pub fn from_file_name(file_name: &str) -> Self {
        if let Some(extension) = std::path::Path::new(file_name).extension() {
            Self::from_extension(&extension.to_string_lossy())
        } else {
            Self::Unknown
        }
    }

    /// 获取语言的常用扩展名
    pub fn common_extensions(&self) -> Vec<&'static str> {
        match self {
            Self::Rust => vec!["rs"],
            Self::Java => vec!["java"],
            Self::Python => vec!["py", "pyx", "pyi"],
            Self::JavaScript => vec!["js", "jsx"],
            Self::TypeScript => vec!["ts", "tsx"],
            Self::Go => vec!["go"],
            Self::C => vec!["c", "h"],
            Self::Cpp => vec!["cpp", "cc", "cxx", "c++", "hpp", "hh"],
            Self::CSharp => vec!["cs"],
            Self::Scala => vec!["scala"],
            Self::Kotlin => vec!["kt", "kts"],
            Self::Swift => vec!["swift"],
            Self::Php => vec!["php"],
            Self::Ruby => vec!["rb"],
            Self::FSharp => vec!["fs", "fsx", "fsi"],
            Self::OCaml => vec!["ml", "mli"],
            Self::Haskell => vec!["hs", "lhs"],
            Self::Unknown => vec![],
        }
    }

    /// 获取语言的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Java => "Java",
            Self::Python => "Python",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Go => "Go",
            Self::C => "C",
            Self::Cpp => "C++",
            Self::CSharp => "C#",
            Self::Scala => "Scala",
            Self::Kotlin => "Kotlin",
            Self::Swift => "Swift",
            Self::Php => "PHP",
            Self::Ruby => "Ruby",
            Self::FSharp => "F#",
            Self::OCaml => "OCaml",
            Self::Haskell => "Haskell",
            Self::Unknown => "Unknown",
        }
    }

    /// 检查是否支持Tree-sitter
    pub fn supports_tree_sitter(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

impl fmt::Display for ProgrammingLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// 代码行范围值对象
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineRange {
    start_line: u32,
    end_line: u32,
}

impl LineRange {
    /// 创建新的行范围
    pub fn new(start_line: u32, end_line: u32) -> Result<Self, String> {
        if start_line == 0 || end_line == 0 {
            return Err("Line numbers must be greater than 0".to_string());
        }

        if start_line > end_line {
            return Err("Start line must be less than or equal to end line".to_string());
        }

        if end_line - start_line > 10000 {
            return Err("Line range too large (max 10000 lines)".to_string());
        }

        Ok(Self {
            start_line,
            end_line,
        })
    }

    /// 获取起始行
    pub fn start_line(&self) -> u32 {
        self.start_line
    }

    /// 获取结束行
    pub fn end_line(&self) -> u32 {
        self.end_line
    }

    /// 获取行数
    pub fn line_count(&self) -> u32 {
        self.end_line - self.start_line + 1
    }

    /// 检查是否包含指定行
    pub fn contains_line(&self, line: u32) -> bool {
        line >= self.start_line && line <= self.end_line
    }

    /// 检查是否与另一个范围重叠
    pub fn overlaps(&self, other: &LineRange) -> bool {
        self.start_line <= other.end_line && self.end_line >= other.start_line
    }

    /// 合并两个重叠的范围
    pub fn merge(&self, other: &LineRange) -> Option<LineRange> {
        if self.overlaps(other) {
            let start_line = self.start_line.min(other.start_line);
            let end_line = self.end_line.max(other.end_line);
            Some(LineRange::new(start_line, end_line).ok()?)
        } else {
            None
        }
    }
}

impl fmt::Display for LineRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start_line == self.end_line {
            write!(f, "line {}", self.start_line)
        } else {
            write!(f, "lines {}-{}", self.start_line, self.end_line)
        }
    }
}

/// 代码行变更类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeType {
    /// 新增行
    Added,
    /// 删除行
    Removed,
    /// 修改行
    Modified,
    /// 上下文行（未变更）
    Context,
}

impl ChangeType {
    /// 获取变更类型的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Removed => "removed",
            Self::Modified => "modified",
            Self::Context => "context",
        }
    }

    /// 获取变更类型的符号表示
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Added => "+",
            Self::Removed => "-",
            Self::Modified => "~",
            Self::Context => " ",
        }
    }
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// 代码变更实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    /// 文件路径
    pub file_path: FilePath,
    /// 变更类型
    pub change_type: ChangeType,
    /// 变更的行范围
    pub line_range: LineRange,
    /// 变更前的内容
    pub old_content: Option<String>,
    /// 变更后的内容
    pub new_content: Option<String>,
}

impl CodeChange {
    /// 创建新的代码变更
    pub fn new(
        file_path: FilePath,
        change_type: ChangeType,
        line_range: LineRange,
        old_content: Option<String>,
        new_content: Option<String>,
    ) -> Self {
        Self {
            file_path,
            change_type,
            line_range,
            old_content,
            new_content,
        }
    }

    /// 获取变更的统计信息
    pub fn get_stats(&self) -> ChangeStats {
        let lines_changed = self.line_range.line_count();
        let content_size = self
            .new_content
            .as_ref()
            .or(self.old_content.as_ref())
            .map(|content| content.len())
            .unwrap_or(0);

        ChangeStats {
            lines_changed,
            content_size,
            change_type: self.change_type,
        }
    }
}

/// 变更统计信息
#[derive(Debug, Clone)]
pub struct ChangeStats {
    /// 变更的行数
    pub lines_changed: u32,
    /// 内容大小（字节数）
    pub content_size: usize,
    /// 变更类型
    pub change_type: ChangeType,
}

/// 代码质量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityMetrics {
    /// 代码行数
    pub lines_of_code: u32,
    /// 圈复杂度
    pub cyclomatic_complexity: Option<f64>,
    /// 代码重复率（百分比）
    pub duplication_percentage: Option<f64>,
    /// 测试覆盖率（百分比）
    pub test_coverage: Option<f64>,
    /// 技术债务评分
    pub technical_debt_score: Option<f64>,
    /// 代码异味数量
    pub code_smell_count: u32,
    /// 安全漏洞数量
    pub security_vulnerability_count: u32,
}

impl CodeQualityMetrics {
    /// 创建新的质量指标
    pub fn new(lines_of_code: u32) -> Self {
        Self {
            lines_of_code,
            cyclomatic_complexity: None,
            duplication_percentage: None,
            test_coverage: None,
            technical_debt_score: None,
            code_smell_count: 0,
            security_vulnerability_count: 0,
        }
    }

    /// 获取总体质量评分（0-100）
    pub fn overall_quality_score(&self) -> f64 {
        let mut score = 100.0;

        // 基于圈复杂度扣分
        if let Some(complexity) = self.cyclomatic_complexity {
            if complexity > 10.0 {
                score -= (complexity - 10.0) * 2.0;
            }
        }

        // 基于代码重复率扣分
        if let Some(duplication) = self.duplication_percentage {
            score -= duplication * 0.5;
        }

        // 基于测试覆盖率加分
        if let Some(coverage) = self.test_coverage {
            if coverage < 80.0 {
                score -= (80.0 - coverage) * 0.5;
            }
        }

        // 基于技术债务扣分
        if let Some(debt) = self.technical_debt_score {
            score -= debt * 10.0;
        }

        // 基于代码异味扣分
        score -= (self.code_smell_count as f64) * 2.0;

        // 基于安全漏洞扣分
        score -= (self.security_vulnerability_count as f64) * 10.0;

        score.clamp(0.0, 100.0)
    }
}

/// 时间戳值对象
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp {
    inner: DateTime<Utc>,
}

impl Timestamp {
    /// 创建当前时间戳
    pub fn now() -> Self {
        Self { inner: Utc::now() }
    }

    /// 从DateTime创建
    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self { inner: dt }
    }

    /// 转换为DateTime
    pub fn to_datetime(&self) -> DateTime<Utc> {
        self.inner
    }

    /// 获取Unix时间戳
    pub fn unix_timestamp(&self) -> i64 {
        self.inner.timestamp()
    }

    /// 格式化显示
    pub fn format(&self, format: &str) -> Result<String, chrono::format::ParseError> {
        Ok(self.inner.format(format).to_string())
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.to_rfc3339())
    }
}

/// 版本号值对象
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    pre_release: Option<String>,
    build_metadata: Option<String>,
}

impl Version {
    /// 创建新版本
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build_metadata: None,
        }
    }

    /// 从字符串解析版本
    pub fn parse(version_str: &str) -> Result<Self, String> {
        // 简化的版本解析，实际应该使用semver库
        let parts: Vec<&str> = version_str.split('.').collect();

        if parts.len() < 3 {
            return Err("Version must have at least 3 parts".to_string());
        }

        let major = parts[0].parse().map_err(|_| "Invalid major version")?;
        let minor = parts[1].parse().map_err(|_| "Invalid minor version")?;
        let patch = parts[2].parse().map_err(|_| "Invalid patch version")?;

        Ok(Self {
            major,
            minor,
            patch,
            pre_release: None,
            build_metadata: None,
        })
    }

    /// 获取主版本号
    pub fn major(&self) -> u32 {
        self.major
    }

    /// 获取次版本号
    pub fn minor(&self) -> u32 {
        self.minor
    }

    /// 获取补丁版本号
    pub fn patch(&self) -> u32 {
        self.patch
    }

    /// 检查版本兼容性（语义化版本规范）
    pub fn is_compatible_with(&self, other: &Version) -> bool {
        // 主版本号相同即为兼容
        self.major == other.major
    }

    /// 检查是否是预发布版本
    pub fn is_pre_release(&self) -> bool {
        self.pre_release.is_some()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if let Some(pre) = &self.pre_release {
            write!(f, "-{pre}")?;
        }

        if let Some(build) = &self.build_metadata {
            write!(f, "+{build}")?;
        }

        Ok(())
    }
}

/// 错误级别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    /// 信息级别
    Info,
    /// 警告级别
    Warning,
    /// 错误级别
    Error,
    /// 严重级别
    Critical,
}

impl ErrorSeverity {
    /// 获取显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }

    /// 获取图标表示
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Info => "ℹ️",
            Self::Warning => "⚠️",
            Self::Error => "❌",
            Self::Critical => "🚨",
        }
    }

    /// 检查是否需要阻断操作
    pub fn should_block(&self) -> bool {
        matches!(self, Self::Error | Self::Critical)
    }
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// 分页信息值对象
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pagination {
    page: u32,
    page_size: u32,
    total_count: Option<u64>,
}

impl Pagination {
    /// 创建新的分页信息
    pub fn new(page: u32, page_size: u32) -> Result<Self, String> {
        if page == 0 {
            return Err("Page number must be greater than 0".to_string());
        }

        if page_size == 0 || page_size > 1000 {
            return Err("Page size must be between 1 and 1000".to_string());
        }

        Ok(Self {
            page,
            page_size,
            total_count: None,
        })
    }

    /// 设置总记录数
    pub fn with_total_count(mut self, total: u64) -> Self {
        self.total_count = Some(total);
        self
    }

    /// 获取页码
    pub fn page(&self) -> u32 {
        self.page
    }

    /// 获取每页大小
    pub fn page_size(&self) -> u32 {
        self.page_size
    }

    /// 获取总记录数
    pub fn total_count(&self) -> Option<u64> {
        self.total_count
    }

    /// 获取总页数
    pub fn total_pages(&self) -> Option<u32> {
        self.total_count
            .map(|total| total.div_ceil(self.page_size as u64) as u32)
    }

    /// 获取偏移量
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.page_size
    }

    /// 是否有上一页
    pub fn has_previous(&self) -> bool {
        self.page > 1
    }

    /// 是否有下一页
    pub fn has_next(&self) -> bool {
        self.total_pages()
            .map(|total| self.page < total)
            .unwrap_or(true)
    }

    /// 转换为SQL LIMIT子句
    pub fn to_sql_limit(&self) -> String {
        format!("LIMIT {} OFFSET {}", self.page_size, self.offset())
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self::new(1, 20).expect("Default pagination should be valid")
    }
}

/// 排序信息值对象
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sort {
    field: String,
    direction: SortDirection,
}

impl Sort {
    /// 创建新的排序信息
    pub fn new(field: impl Into<String>, direction: SortDirection) -> Self {
        Self {
            field: field.into(),
            direction,
        }
    }

    /// 升序排序
    pub fn ascending(field: impl Into<String>) -> Self {
        Self::new(field, SortDirection::Ascending)
    }

    /// 降序排序
    pub fn descending(field: impl Into<String>) -> Self {
        Self::new(field, SortDirection::Descending)
    }

    /// 获取排序字段
    pub fn field(&self) -> &str {
        &self.field
    }

    /// 获取排序方向
    pub fn direction(&self) -> SortDirection {
        self.direction
    }

    /// 转换为SQL ORDER BY子句
    pub fn to_sql_order(&self) -> String {
        let direction = match self.direction {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        };
        format!("{} {}", self.field, direction)
    }
}

/// 排序方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// 升序
    Ascending,
    /// 降序
    Descending,
}

impl SortDirection {
    /// 获取SQL表示
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Ascending => "ASC",
            Self::Descending => "DESC",
        }
    }
}

impl fmt::Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ascending => write!(f, "ascending"),
            Self::Descending => write!(f, "descending"),
        }
    }
}

/// 查询条件值对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCriteria {
    filters: Vec<Filter>,
    sorts: Vec<Sort>,
    pagination: Pagination,
}

impl QueryCriteria {
    /// 创建新的查询条件
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            sorts: Vec::new(),
            pagination: Pagination::default(),
        }
    }

    /// 添加过滤条件
    pub fn with_filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// 添加排序条件
    pub fn with_sort(mut self, sort: Sort) -> Self {
        self.sorts.push(sort);
        self
    }

    /// 设置分页
    pub fn with_pagination(mut self, pagination: Pagination) -> Self {
        self.pagination = pagination;
        self
    }

    /// 获取过滤条件
    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }

    /// 获取排序条件
    pub fn sorts(&self) -> &[Sort] {
        &self.sorts
    }

    /// 获取分页信息
    pub fn pagination(&self) -> &Pagination {
        &self.pagination
    }

    /// 转换为SQL WHERE子句
    pub fn to_sql_where(&self) -> String {
        if self.filters.is_empty() {
            return String::new();
        }

        let conditions: Vec<String> = self
            .filters
            .iter()
            .map(|filter| filter.to_sql_condition())
            .collect();

        format!("WHERE {}", conditions.join(" AND "))
    }

    /// 转换为SQL ORDER BY子句
    pub fn to_sql_order(&self) -> String {
        if self.sorts.is_empty() {
            return String::new();
        }

        let order_by: Vec<String> = self.sorts.iter().map(|sort| sort.to_sql_order()).collect();

        format!("ORDER BY {}", order_by.join(", "))
    }
}

impl Default for QueryCriteria {
    fn default() -> Self {
        Self::new()
    }
}

/// 过滤条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    field: String,
    operator: FilterOperator,
    value: serde_json::Value,
}

impl Filter {
    /// 创建新的过滤条件
    pub fn new(
        field: impl Into<String>,
        operator: FilterOperator,
        value: serde_json::Value,
    ) -> Self {
        Self {
            field: field.into(),
            operator,
            value,
        }
    }

    /// 等于条件
    pub fn equals(
        field: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self::new(
            field,
            FilterOperator::Equals,
            serde_json::to_value(value)?,
        ))
    }

    /// 包含条件
    pub fn contains(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(
            field,
            FilterOperator::Contains,
            serde_json::Value::String(value.into()),
        )
    }

    /// 大于条件
    pub fn greater_than(
        field: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self::new(
            field,
            FilterOperator::GreaterThan,
            serde_json::to_value(value)?,
        ))
    }

    /// 小于条件
    pub fn less_than(
        field: impl Into<String>,
        value: impl Serialize,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self::new(
            field,
            FilterOperator::LessThan,
            serde_json::to_value(value)?,
        ))
    }

    /// 转换为SQL条件
    pub fn to_sql_condition(&self) -> String {
        let field = &self.field;

        match &self.operator {
            FilterOperator::Equals => format!("{} = {}", field, self.format_sql_value()),
            FilterOperator::NotEquals => format!("{} != {}", field, self.format_sql_value()),
            FilterOperator::Contains => {
                format!("{} LIKE '%{}%'", field, self.format_sql_like_value())
            }
            FilterOperator::GreaterThan => format!("{} > {}", field, self.format_sql_value()),
            FilterOperator::LessThan => format!("{} < {}", field, self.format_sql_value()),
            FilterOperator::GreaterThanOrEquals => {
                format!("{} >= {}", field, self.format_sql_value())
            }
            FilterOperator::LessThanOrEquals => format!("{} <= {}", field, self.format_sql_value()),
            FilterOperator::In => format!("{} IN ({})", field, self.format_sql_in_values()),
            FilterOperator::NotIn => format!("{} NOT IN ({})", field, self.format_sql_in_values()),
            FilterOperator::IsNull => format!("{field} IS NULL"),
            FilterOperator::IsNotNull => format!("{field} IS NOT NULL"),
        }
    }

    fn format_sql_value(&self) -> String {
        match &self.value {
            serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "NULL".to_string(),
            _ => "'{}'".to_string(), // 其他类型序列化为JSON
        }
    }

    fn format_sql_like_value(&self) -> String {
        if let serde_json::Value::String(s) = &self.value {
            s.replace('\'', "''")
                .replace('%', "\\%")
                .replace('_', "\\_")
        } else {
            String::new()
        }
    }

    fn format_sql_in_values(&self) -> String {
        if let serde_json::Value::Array(arr) = &self.value {
            arr.iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
                    serde_json::Value::Number(n) => n.to_string(),
                    _ => "NULL".to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            self.format_sql_value()
        }
    }
}

/// 过滤操作符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    /// 等于
    Equals,
    /// 不等于
    NotEquals,
    /// 包含
    Contains,
    /// 大于
    GreaterThan,
    /// 小于
    LessThan,
    /// 大于等于
    GreaterThanOrEquals,
    /// 小于等于
    LessThanOrEquals,
    /// 在列表中
    In,
    /// 不在列表中
    NotIn,
    /// 为空
    IsNull,
    /// 不为空
    IsNotNull,
}

/// 结果包装器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultWrapper<T> {
    /// 结果数据
    pub data: T,
    /// 结果元数据
    pub metadata: ResultMetadata,
}

impl<T> ResultWrapper<T> {
    /// 创建新的结果包装器
    pub fn new(data: T) -> Self {
        Self {
            data,
            metadata: ResultMetadata::default(),
        }
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: ResultMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// 结果元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResultMetadata {
    /// 时间戳
    pub timestamp: Option<DateTime<Utc>>,
    /// 版本信息
    pub version: Option<String>,
    /// 请求ID
    pub request_id: Option<String>,
    /// 处理时间（毫秒）
    pub processing_time_ms: Option<u64>,
    /// 缓存命中标志
    pub cache_hit: Option<bool>,
}

impl ResultMetadata {
    /// 创建新的元数据
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置时间戳
    pub fn with_timestamp(mut self) -> Self {
        self.timestamp = Some(Utc::now());
        self
    }

    /// 设置版本信息
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// 设置请求ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// 设置处理时间
    pub fn with_processing_time(mut self, duration: std::time::Duration) -> Self {
        self.processing_time_ms = Some(duration.as_millis() as u64);
        self
    }

    /// 设置缓存命中状态
    pub fn with_cache_hit(mut self, hit: bool) -> Self {
        self.cache_hit = Some(hit);
        self
    }
}

/// 审计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditInfo {
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 创建者
    pub created_by: Option<String>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 更新者
    pub updated_by: Option<String>,
    /// 版本号
    pub version: u64,
}

impl AuditInfo {
    /// 创建新的审计信息
    pub fn new(creator: Option<impl Into<String>>) -> Self {
        let now = Utc::now();
        let created_by = creator.map(|c| c.into());

        Self {
            created_at: now,
            created_by: created_by.clone(),
            updated_at: now,
            updated_by: created_by,
            version: 1,
        }
    }

    /// 更新审计信息
    pub fn update(&mut self, updater: Option<impl Into<String>>) {
        self.updated_at = Utc::now();
        self.updated_by = updater.map(|u| u.into());
        self.version += 1;
    }
}

impl Default for AuditInfo {
    fn default() -> Self {
        Self::new(None::<String>)
    }
}

/// 软删除信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionInfo {
    /// 是否已删除
    pub is_deleted: bool,
    /// 删除时间
    pub deleted_at: Option<DateTime<Utc>>,
    /// 删除者
    pub deleted_by: Option<String>,
}

impl DeletionInfo {
    /// 创建新的软删除信息
    pub fn new() -> Self {
        Self {
            is_deleted: false,
            deleted_at: None,
            deleted_by: None,
        }
    }

    /// 标记为删除
    pub fn delete(&mut self, deleter: Option<impl Into<String>>) {
        self.is_deleted = true;
        self.deleted_at = Some(Utc::now());
        self.deleted_by = deleter.map(|d| d.into());
    }

    /// 恢复删除
    pub fn restore(&mut self) {
        self.is_deleted = false;
        self.deleted_at = None;
        self.deleted_by = None;
    }
}

impl Default for DeletionInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// 标签集合
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Tags {
    tags: std::collections::HashSet<String>,
}

impl Tags {
    /// 创建空的标签集合
    pub fn new() -> Self {
        Self {
            tags: std::collections::HashSet::new(),
        }
    }

    /// 从向量创建标签集合
    pub fn from_vec(tags: Vec<impl Into<String>>) -> Self {
        Self {
            tags: tags.into_iter().map(|t| t.into()).collect(),
        }
    }

    /// 添加标签
    pub fn add(&mut self, tag: impl Into<String>) -> bool {
        self.tags.insert(tag.into())
    }

    /// 移除标签
    pub fn remove(&mut self, tag: &str) -> bool {
        self.tags.remove(tag)
    }

    /// 检查是否包含标签
    pub fn contains(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    /// 获取标签数量
    pub fn len(&self) -> usize {
        self.tags.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// 转换为向量
    pub fn to_vec(&self) -> Vec<String> {
        self.tags.iter().cloned().collect()
    }

    /// 获取迭代器
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.tags.iter()
    }
}

impl std::iter::FromIterator<String> for Tags {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self {
            tags: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for Tags {
    type Item = String;
    type IntoIter = std::collections::hash_set::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.tags.into_iter()
    }
}

/// 文件路径结果类型别名
pub type FilePathResult = Result<FilePath, String>;
/// 行范围结果类型别名
pub type LineRangeResult = Result<LineRange, String>;
/// 版本结果类型别名
pub type VersionResult = Result<Version, String>;

/// 项目实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// 项目ID
    pub id: String,
    /// 项目名称
    pub name: String,
    /// 项目描述
    pub description: Option<String>,
    /// 项目路径
    pub path: String,
    /// 项目语言
    pub language: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// 分析结果ID
    pub id: String,
    /// 项目ID
    pub project_id: String,
    /// 分析类型
    pub analysis_type: String,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: DateTime<Utc>,
    /// 分析状态
    pub status: AnalysisStatus,
    /// 发现的问题列表
    pub findings: Vec<Finding>,
    /// 分析摘要
    pub summary: AnalysisSummary,
}

/// 分析状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnalysisStatus {
    /// 等待中
    Pending,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
}

/// 发现问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// 问题ID
    pub id: String,
    /// 严重级别
    pub severity: FindingSeverity,
    /// 问题类别
    pub category: String,
    /// 问题描述
    pub message: String,
    /// 文件路径
    pub file_path: Option<String>,
    /// 行号
    pub line_number: Option<u32>,
    /// 建议
    pub recommendation: Option<String>,
}

/// 发现严重级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FindingSeverity {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重
    Critical,
}

/// 分析摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    /// 总文件数
    pub total_files: usize,
    /// 已分析文件数
    pub files_analyzed: usize,
    /// 总发现问题数
    pub total_findings: usize,
    /// 按严重级别分类的问题数量
    pub findings_by_severity: HashMap<FindingSeverity, usize>,
    /// 持续时间（毫秒）
    pub duration_ms: u64,
}

/// 配置实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    /// 配置ID
    pub id: String,
    /// 配置名称
    pub name: String,
    /// 配置版本
    pub version: String,
    /// 设置项
    pub settings: HashMap<String, serde_json::Value>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 工作流实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// 工作流ID
    pub id: String,
    /// 工作流名称
    pub name: String,
    /// 工作流描述
    pub description: Option<String>,
    /// 工作流步骤
    pub steps: Vec<WorkflowStep>,
    /// 工作流状态
    pub status: WorkflowStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 开始时间
    pub started_at: Option<DateTime<Utc>>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
}

/// 工作流步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// 步骤ID
    pub id: String,
    /// 步骤名称
    pub name: String,
    /// 步骤类型
    pub step_type: String,
    /// 参数
    pub parameters: HashMap<String, serde_json::Value>,
    /// 依赖的步骤ID
    pub depends_on: Vec<String>,
    /// 步骤状态
    pub status: StepStatus,
}

/// 工作流状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// 等待中
    Pending,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 步骤状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StepStatus {
    /// 等待中
    Pending,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已跳过
    Skipped,
}
