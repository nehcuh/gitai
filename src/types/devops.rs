use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DevOpsResponse {
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<WorkItem>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct WorkItem {
    /// DevOps work item unique identifier
    /// Source: DevOps API response `data.id`
    /// Example: 1255140 (用户故事), 1455437 (缺陷), 1323273 (任务)
    pub id: u32,
    
    /// DevOps work item code/number used for reference  
    /// Source: DevOps API response `data.code`
    /// Example: 99 (用户故事), 833118 (缺陷), 655911 (任务)
    pub code: Option<u32>,
    
    /// Work item title/summary - the main subject of the work item
    /// Source: DevOps API response `data.name`
    /// Example: "封装 requests 函数到用户自定义函数" (用户故事)
    pub name: String,
    
    /// Detailed work item description - contains requirements, acceptance criteria, etc.
    /// Source: DevOps API response `data.description`
    /// This is the primary content AI will use to understand requirements and compare against code
    /// May contain markdown formatting, images, and structured content
    pub description: String,
    
    /// Project/Product context name for AI to understand business domain
    /// Source: DevOps API response `data.program.display_name`
    /// Example: "金科中心代码扫描引擎项目预研" (用户故事)
    ///          "T7.6券结(含券结ETF)融资行权业务回归及单客户上线" (缺陷)
    ///          null or missing (任务 - handle gracefully)
    #[serde(rename = "program")]
    pub project_name: Option<Program>,
    
    #[serde(rename = "issueTypeDetail")]
    pub issue_type_detail: IssueTypeDetail,
    pub r#type: String, // Handles Rust keyword `type`
    #[serde(rename = "issueStatusName")]
    pub status_name: String,
    pub priority: u32,
}

/// Program/Project information extracted from DevOps API response
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Program {
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct IssueTypeDetail {
    pub id: u32,
    /// Human-readable work item type for AI analysis strategy differentiation
    /// Source: DevOps API response `data.issueTypeDetail.name`
    /// Expected values: "用户故事", "缺陷", "任务"
    /// This field helps AI apply different analysis approaches for different work item types
    pub name: String,
    #[serde(rename = "iconType")]
    pub icon_type: String,
    #[serde(rename = "issueType")]
    pub issue_type: String,
}

/// Analysis-specific WorkItem representation
/// This struct is optimized for AI analysis by extracting and organizing
/// the most relevant fields from the DevOps API response
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnalysisWorkItem {
    /// DevOps work item unique identifier
    pub id: Option<u32>,
    
    /// DevOps work item code/number used for reference
    pub code: Option<u32>,
    
    /// Project/Product context name for AI to understand business domain
    pub project_name: Option<String>,
    
    /// Human-readable work item type for AI analysis strategy differentiation
    pub item_type_name: Option<String>,
    
    /// Work item title/summary - the main subject of the work item
    pub title: Option<String>,
    
    /// Detailed work item description - contains requirements, acceptance criteria, etc.
    pub description: Option<String>,
}

impl From<&WorkItem> for AnalysisWorkItem {
    fn from(work_item: &WorkItem) -> Self {
        Self {
            id: Some(work_item.id),
            code: work_item.code,
            project_name: work_item.project_name
                .as_ref()
                .and_then(|p| p.display_name.clone()),
            item_type_name: Some(work_item.issue_type_detail.name.clone()),
            title: Some(work_item.name.clone()),
            description: Some(work_item.description.clone()),
        }
    }
}
