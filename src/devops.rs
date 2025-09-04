use crate::config::DevOpsConfig;
use serde::{Deserialize, Serialize};

/// Issue信息（统一结构，便于上层消费）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// 展示用ID：对于 Coding 使用 issue 编号（code），对于 GitHub 使用 issue number
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: Option<String>,
    pub assignee: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub labels: Vec<String>,
    pub url: String,
    /// AI 使用的上下文摘要（按平台定制，便于偏离度分析）
    pub ai_context: Option<String>,
}

/// DevOps客户端
pub struct DevOpsClient {
    config: DevOpsConfig,
    client: reqwest::Client,
}

impl DevOpsClient {
    /// Creates a new DevOpsClient using the provided configuration.
    ///
    /// The function builds an underlying `reqwest::Client` with its request timeout set
    /// from `config.timeout` (in seconds). If building the HTTP client fails, a
    /// default `reqwest::Client` is used as a fallback.
    ///
    /// # Examples
    ///
    /// ```
    /// // Assuming `DevOpsConfig` implements `Default` for examples/tests.
    /// let cfg = DevOpsConfig::default();
    /// let client = DevOpsClient::new(cfg);
    /// ```
    pub fn new(config: DevOpsConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout))
            .build()
            .unwrap_or_default();

        Self { config, client }
    }

    /// Fetch a single Issue using the client's configured platform and space.
    ///
    /// This is a convenience wrapper that delegates to `get_issue_with_space`, using the
    /// client's configured `space_id`. A leading `#` in `issue_id` (e.g. `#123`) is ignored.
    ///
    /// # Returns
    ///
    /// Returns the unified `Issue` representation for the requested issue, or an error if the
    /// underlying platform request fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn _example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = DevOpsClient::new(/* config */);
    /// let issue = client.get_issue("123").await?;
    /// println!("{}", issue.title);
    /// # Ok(()) }
    /// ```
    pub async fn get_issue(
        &self,
        issue_id: &str,
    ) -> Result<Issue, Box<dyn std::error::Error + Send + Sync>> {
        self.get_issue_with_space(issue_id, self.config.space_id)
            .await
    }

    /// Fetches a single issue by ID, optionally overriding the configured Coding space.
    ///
    /// The function normalizes an incoming `issue_id` by trimming a leading `#` and dispatches
    /// the request to the platform-specific fetcher based on the client's configured platform:
    /// - "coding": calls the Coding-specific loader and uses `space_override` if provided, otherwise the client's configured space_id.
    /// - "github": calls the GitHub-specific loader (ignores `space_override`).
    /// Returns an `Err` when the client's platform is not supported.
    ///
    /// # Parameters
    ///
    /// - `issue_id`: The issue identifier (may start with `#`).
    /// - `space_override`: Optional Coding space ID to use instead of the client's configured space.
    ///
    /// # Returns
    ///
    /// A `Result` containing the resolved `Issue` on success or a boxed error on failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn _example(client: &crate::DevOpsClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let issue = client.get_issue_with_space("#123", None).await?;
    /// println!("{}", issue.title);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_issue_with_space(
        &self,
        issue_id: &str,
        space_override: Option<u64>,
    ) -> Result<Issue, Box<dyn std::error::Error + Send + Sync>> {
        let clean_id = issue_id.trim_start_matches('#');
        match self.config.platform.to_lowercase().as_str() {
            "coding" => {
                self.get_coding_issue(clean_id, space_override.or(self.config.space_id))
                    .await
            }
            "github" => self.get_github_issue(clean_id).await,
            _ => Err(format!("Unsupported platform: {}", self.config.platform).into()),
        }
    }

    /// Fetches multiple issues by their IDs using the client's configured space.
    ///
    /// This is a convenience wrapper that delegates to `get_issues_with_space`, passing
    /// the client's configured `space_id`. It returns a vector of successfully
    /// retrieved `Issue` objects; individual fetch failures are handled by the
    /// underlying implementation (they may be logged and skipped).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn run_example(client: &crate::DevOpsClient) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// let ids = vec!["123".to_string(), "456".to_string()];
    /// let issues = client.get_issues(&ids).await?;
    /// assert!(issues.len() <= ids.len());
    /// # Ok(()) }
    /// ```
    pub async fn get_issues(
        &self,
        ids: &[String],
    ) -> Result<Vec<Issue>, Box<dyn std::error::Error + Send + Sync>> {
        self.get_issues_with_space(ids, self.config.space_id).await
    }

    /// Fetches multiple issues by their IDs, optionally overriding the configured Coding space ID.
    ///
    /// This attempts to fetch each issue using `get_issue_with_space` with the provided `space_override`.
    /// Successful fetches are collected and returned; individual failures are logged as warnings and do not
    /// abort the operation.
    ///
    /// # Parameters
    ///
    /// - `space_override`: when `Some`, that space ID is used for platforms that require a space (e.g., Coding);
    ///   when `None`, the client's configured space is used.
    ///
    /// # Returns
    ///
    /// A `Vec<Issue>` containing all successfully retrieved issues. The call itself only fails on unexpected
    /// errors that prevent performing the batch operation (individual issue errors are suppressed).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn doc() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// let client = DevOpsClient::new(/* config */);
    /// let ids = vec!["123".to_string(), "456".to_string()];
    /// let issues = client.get_issues_with_space(&ids, Some(42)).await?;
    /// assert!(issues.iter().all(|i| !i.id.is_empty()));
    /// # Ok(()) }
    /// ```
    pub async fn get_issues_with_space(
        &self,
        ids: &[String],
        space_override: Option<u64>,
    ) -> Result<Vec<Issue>, Box<dyn std::error::Error + Send + Sync>> {
        let mut issues = Vec::new();
        for id in ids {
            match self.get_issue_with_space(id, space_override).await {
                Ok(issue) => issues.push(issue),
                Err(e) => eprintln!("Warning: Failed to fetch issue {}: {}", id, e),
            }
        }
        Ok(issues)
    }

    /// Fetches a Coding-platform issue (using the external/collaboration API) and maps it into the unified `Issue` model.
    ///
    /// This performs an HTTP GET to the Coding external/collaboration endpoint for the given project (space) and issue code,
    /// validates the API response, extracts relevant fields, converts the creation timestamp to RFC3339 (when present),
    /// and synthesizes a human-readable AI context summary placed in `Issue.ai_context`.
    ///
    /// Errors:
    /// - Returns an error if `space_id` is `None`.
    /// - Returns an error for non-success HTTP responses or when the Coding API returns a non-zero `code`.
    /// - Returns an error on JSON deserialization or other request failures.
    ///
    /// Parameters:
    /// - `issue_code`: issue code/number as used by Coding (leading `#` should be removed beforehand if present).
    /// - `space_id`: Coding project (space) ID to query; required for this API.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// let client = DevOpsClient::new(/* config */);
    /// let issue = client.get_coding_issue("ISSUE-123", Some(42)).await?;
    /// println!("Title: {}", issue.title);
    /// # Ok(())
    /// # }
    /// ```
    async fn get_coding_issue(
        &self,
        issue_code: &str,
        space_id: Option<u64>,
    ) -> Result<Issue, Box<dyn std::error::Error + Send + Sync>> {
        let space_id = space_id.ok_or(
            "缺少 Coding 空间（项目）ID：请在配置 devops.space_id 设置，或通过 --space-id 指定",
        )?;
        let base = self.config.base_url.trim_end_matches('/');
        let url = format!(
            "{}/external/collaboration/api/project/{}/issues/{}",
            base, space_id, issue_code
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("Authorization", format!("token {}", self.config.token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to fetch issue: {}", response.status()).into());
        }

        let root: serde_json::Value = response.json().await?;
        if root["code"].as_i64().unwrap_or(-1) != 0 {
            return Err(format!("Coding API error: {:?}", root["msg"]).into());
        }
        let data = &root["data"];

        // 提取字段
        let title = data["name"].as_str().unwrap_or("").to_string();
        let description = data["description"].as_str().unwrap_or("").to_string();
        let status = data["issueStatusName"].as_str().unwrap_or("").to_string();
        let code = data["code"].as_i64().unwrap_or_default();
        let internal_id = data["id"].as_i64().unwrap_or_default();
        let project_id = data["projectId"].as_i64().unwrap_or_default();
        let creator = data["creator"]["name"].as_str().unwrap_or("").to_string();
        let plan_date = data["planDate"].as_str().unwrap_or("").to_string();
        let team_name = data["groupTeam"]["groupTeamName"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let team_leader = data["groupTeam"]["groupTeamLeaderName"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let tester = data["groupTeam"]["groupTeamTesterName"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let system_name = data["systemDTO"]["systemName"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let system_no = data["systemDTO"]["systemNo"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let created_at_ms = data["createdAt"].as_i64().unwrap_or(0);
        let created_at = if created_at_ms > 0 {
            // API 返回毫秒时间戳
            let secs = created_at_ms / 1000;
            let nanos = ((created_at_ms % 1000) * 1_000_000) as u32;
            let dt_utc = chrono::DateTime::<chrono::Utc>::from_timestamp(secs, nanos)
                .unwrap_or_else(|| chrono::DateTime::<chrono::Utc>::from_timestamp(0, 0).unwrap());
            dt_utc.to_rfc3339()
        } else {
            String::new()
        };

        // 构建 AI 上下文摘要
        let mut ai_lines = Vec::new();
        ai_lines.push("📋 需求详情".to_string());
        ai_lines.push(String::new());
        ai_lines.push(format!("标题: {}", title));
        let itype = data["issueTypeDetail"]["name"].as_str().unwrap_or("");
        let itype_code = data["type"].as_str().unwrap_or("");
        ai_lines.push(format!("类型: {} ({})", itype, itype_code));
        ai_lines.push(format!("状态: {}", status));
        ai_lines.push(String::new());
        if !creator.is_empty() {
            ai_lines.push(format!("创建者: {}", creator));
        }
        if !plan_date.is_empty() {
            ai_lines.push(format!("计划日期: {}", plan_date));
        }
        if !team_name.is_empty() {
            ai_lines.push(format!("所属团队: {}", team_name));
        }
        if !system_name.is_empty() {
            ai_lines.push(format!("关联系统: {} ({})", system_name, system_no));
        }
        ai_lines.push(String::new());
        if !description.trim().is_empty() {
            ai_lines.push("📝 需求描述".to_string());
            ai_lines.push(String::new());
            // 尝试按行输出任务列表
            for line in description.lines() {
                let l = line.trim();
                if !l.is_empty() {
                    ai_lines.push(l.to_string());
                }
            }
            ai_lines.push(String::new());
        }
        ai_lines.push("🔍 关键信息".to_string());
        ai_lines.push(format!("•  项目ID: {}", project_id));
        ai_lines.push(format!("•  问题ID: {}", internal_id));
        ai_lines.push(format!("•  问题编号: {}", code));
        if !created_at.is_empty() {
            ai_lines.push(format!("•  创建时间: {}", created_at));
        }
        if !team_leader.is_empty() {
            ai_lines.push(format!("•  团队负责人: {}", team_leader));
        }
        if !tester.is_empty() {
            ai_lines.push(format!("•  测试人员: {}", tester));
        }

        let ai_context = ai_lines.join("\n");

        Ok(Issue {
            id: code.to_string(),
            title,
            description,
            status,
            priority: None,
            assignee: None,
            created_at,
            updated_at: String::new(),
            labels: Vec::new(),
            url: url.replace("/external/collaboration/api", "/p"),
            ai_context: Some(ai_context),
        })
    }

    /// Fetches a GitHub issue by number and converts it into the unified `Issue` model.
    ///
    /// The function calls the GitHub Issues API for the repository configured in `self.config.project`
    /// (falls back to `"owner/repo"` if unset) and maps the response into an `Issue`. The returned
    /// `Issue` contains core fields (id, title, description, status, assignee, timestamps, labels, url).
    /// The `ai_context` field is left as `None` for GitHub issues.
    ///
    /// Errors are returned for non-success HTTP responses, request failures, or JSON deserialization errors.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn run_example(client: &crate::DevOpsClient) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    /// let issue = client.get_github_issue("123").await?;
    /// println!("Fetched issue {}: {}", issue.id, issue.title);
    /// # Ok(()) }
    /// ```
    async fn get_github_issue(
        &self,
        issue_id: &str,
    ) -> Result<Issue, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.github.com/repos/{}/issues/{}",
            self.config.project.as_deref().unwrap_or("owner/repo"),
            issue_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("token {}", self.config.token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Failed to fetch issue: {}", response.status()).into());
        }

        let issue_data: serde_json::Value = response.json().await?;

        Ok(Issue {
            id: issue_data["number"].as_u64().unwrap_or(0).to_string(),
            title: issue_data["title"].as_str().unwrap_or("").to_string(),
            description: issue_data["body"].as_str().unwrap_or("").to_string(),
            status: if issue_data["state"].as_str().unwrap_or("") == "open" {
                "open".to_string()
            } else {
                "closed".to_string()
            },
            priority: None,
            assignee: issue_data["assignee"]["login"]
                .as_str()
                .map(|s| s.to_string()),
            created_at: issue_data["created_at"].as_str().unwrap_or("").to_string(),
            updated_at: issue_data["updated_at"].as_str().unwrap_or("").to_string(),
            labels: issue_data["labels"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v["name"].as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            url: issue_data["html_url"].as_str().unwrap_or("").to_string(),
            ai_context: None,
        })
    }
}
