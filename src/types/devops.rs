use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct DevOpsResponse {
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<WorkItem>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct WorkItem {
    pub id: u32,
    pub name: String,
    pub description: String,
    #[serde(rename = "issueTypeDetail")]
    pub issue_type_detail: IssueTypeDetail,
    pub r#type: String, // Handles Rust keyword `type`
    #[serde(rename = "issueStatusName")]
    pub status_name: String,
    pub priority: u32,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct IssueTypeDetail {
    pub id: u32,
    pub name: String,
    #[serde(rename = "iconType")]
    pub icon_type: String,
    #[serde(rename = "issueType")]
    pub issue_type: String,
    // The user story shows 'type: "requirement"' in the example,
    // but it's not in the struct definition.
    // Let's stick to the struct definition from the user story.
}
