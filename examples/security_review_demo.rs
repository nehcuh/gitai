//! GitAI 安全评审演示
//!
//! 这个示例展示了AI时代开发安全的核心理念：
//! - 架构一致性检查
//! - 需求符合度验证
//! - 模式合规性分析
//! - 安全边界保护

use gitai::config::Config;
use gitai::security_review::{SecurityReviewResult, SecurityReviewer};
use gitai::tree_sitter::SupportedLanguage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 初始化日志
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    println!("🚀 GitAI 安全评审演示");
    println!("========================");

    // 加载配置
    let config = Config::load()?;

    // 创建安全评审器
    let reviewer = SecurityReviewer::new(config).await?;

    // 示例1: 分析一个包含架构问题的代码
    println!("\n📋 示例1: 架构一致性分析");
    println!("-------------------------");

    let problematic_code = r#"
// 这是一个典型的AI生成代码架构问题示例
// 职责过于分散，违反单一职责原则

class UserService {
    private Database db;
    private EmailService email;
    private PaymentProcessor payment;
    private Logger logger;
    private Cache cache;
    
    public UserService(Database db, EmailService email, 
                      PaymentProcessor payment, Logger logger, Cache cache) {
        this.db = db;
        this.email = email;
        this.payment = payment;
        this.logger = logger;
        this.cache = cache;
    }
    
    // 用户管理
    public User createUser(UserData data) {
        // 创建用户逻辑
        User user = new User(data);
        db.save(user);
        logger.log("User created: " + user.getId());
        return user;
    }
    
    // 邮件发送 - 应该在EmailService中
    public void sendWelcomeEmail(User user) {
        email.send(user.getEmail(), "Welcome!");
        logger.log("Welcome email sent to: " + user.getEmail());
    }
    
    // 支付处理 - 应该在PaymentService中
    public void processPayment(User user, double amount) {
        payment.process(user.getId(), amount);
        logger.log("Payment processed for user: " + user.getId());
        email.send(user.getEmail(), "Payment confirmation");
    }
    
    // 缓存管理 - 应该在CacheService中
    public void cacheUserData(User user) {
        cache.set("user:" + user.getId(), user);
        logger.log("User cached: " + user.getId());
    }
    
    // 报表生成 - 应该在ReportService中
    public String generateUserReport(User user) {
        String report = "User Report for " + user.getName();
        report += "\nEmail: " + user.getEmail();
        report += "\nCreated: " + user.getCreatedAt();
        logger.log("Report generated for user: " + user.getId());
        return report;
    }
    
    // 备份处理 - 应该在BackupService中
    public void backupUserData(User user) {
        String backup = db.export(user);
        logger.log("User backup created: " + user.getId());
        email.send(user.getEmail(), "Your data has been backed up");
    }
}
"#;

    let insights = reviewer
        .security_insights
        .analyze_code(
            problematic_code,
            SupportedLanguage::Java,
            "UserService.java",
            None,
        )
        .await?;

    println!("发现 {} 个架构问题:", insights.len());
    for insight in &insights {
        println!("  📌 {}: {}", insight.title, insight.description);
        println!("     💡 建议: {}", insight.suggestion);
        println!("     🔥 严重程度: {:?}\n", insight.severity);
    }

    // 示例2: 分析需求偏离问题
    println!("\n📋 示例2: 需求偏离分析");
    println!("-----------------------");

    let issue_context = r#"
Issue #123: 添加用户登录功能

需求：
1. 实现用户名密码登录
2. 添加JWT token生成
3. 支持登录失败次数限制
4. 记录登录日志

优先级：高
状态：开发中
"#;

    let deviated_code = r#"
// 代码实现了需求，但引入了额外的不相关功能
class AuthService {
    private UserRepository userRepo;
    private JwtService jwtService;
    private LoginAttemptTracker attemptTracker;
    
    // ✅ 符合需求：用户名密码登录
    public String login(String username, String password) {
        User user = userRepo.findByUsername(username);
        if (user == null || !user.checkPassword(password)) {
            attemptTracker.recordFailedAttempt(username);
            throw new AuthException("Invalid credentials");
        }
        
        String token = jwtService.generateToken(user);
        attemptTracker.reset(username);
        return token;
    }
    
    // ✅ 符合需求：JWT token生成
    public String refreshToken(String oldToken) {
        return jwtService.refreshToken(oldToken);
    }
    
    // ❌ 偏离需求：引入了不相关的社交登录功能
    public String socialLogin(String provider, String accessToken) {
        // 这个功能不在原始需求中，属于需求偏离
        SocialUser socialUser = fetchSocialUser(provider, accessToken);
        User user = userRepo.findBySocialId(provider, socialUser.getId());
        
        if (user == null) {
            user = registerSocialUser(socialUser);
        }
        
        return jwtService.generateToken(user);
    }
    
    // ❌ 偏离需求：引入了不相关的用户偏好设置
    public void updateUserPreferences(User user, Map<String, String> preferences) {
        user.setPreferences(preferences);
        userRepo.save(user);
    }
    
    // ❌ 偏离需求：引入了不相关的推荐算法
    public List<Recommendation> getRecommendations(User user) {
        RecommendationEngine engine = new RecommendationEngine();
        return engine.getRecommendations(user);
    }
}
"#;

    let deviation_insights = reviewer
        .security_insights
        .analyze_code(
            deviated_code,
            SupportedLanguage::Java,
            "AuthService.java",
            Some(issue_context),
        )
        .await?;

    println!("需求偏离分析结果:");
    for insight in &deviation_insights {
        if insight.category == gitai::security_insights::InsightCategory::RequirementDeviation {
            println!("  🚨 {}: {}", insight.title, insight.description);
            println!("     💡 建议: {}", insight.suggestion);
            println!("     🔥 严重程度: {:?}\n", insight.severity);
        }
    }

    // 示例3: 安全边界检查
    println!("\n📋 示例3: 安全边界检查");
    println!("-----------------------");

    let risky_code = r#"
import express from 'express';
import { exec } from 'child_process';
import fs from 'fs';

const app = express();

// 🚨 危险：直接使用用户输入进行eval
app.post('/calculate', (req, res) => {
    const expression = req.body.expression;
    const result = eval(expression); // 危险！
    res.json({ result });
});

// 🚨 危险：直接操作innerHTML
app.post('/update-content', (req, res) => {
    const content = req.body.content;
    document.getElementById('output').innerHTML = content; // XSS风险
    res.json({ success: true });
});

// 🚨 危险：使用exec执行系统命令
app.post('/run-command', (req, res) => {
    const command = req.body.command;
    exec(command, (error, stdout, stderr) => { // 命令注入风险
        if (error) {
            return res.status(500).json({ error: stderr });
        }
        res.json({ output: stdout });
    });
});

// 🚨 危险：直接写入用户提供的文件路径
app.post('/save-file', (req, res) => {
    const filename = req.body.filename;
    const content = req.body.content;
    fs.writeFileSync(filename, content); // 路径遍历风险
    res.json({ success: true });
});

app.listen(3000, () => {
    console.log('Server running on port 3000');
});
"#;

    let security_insights = reviewer
        .security_insights
        .analyze_code(risky_code, SupportedLanguage::JavaScript, "server.js", None)
        .await?;

    println!("安全边界检查结果:");
    for insight in &security_insights {
        if insight.category == gitai::security_insights::InsightCategory::BoundaryProtection {
            println!("  🛡️ {}: {}", insight.title, insight.description);
            println!("     💡 建议: {}", insight.suggestion);
            println!("     🔥 严重程度: {:?}\n", insight.severity);
        }
    }

    // 示例4: 完整的评审流程
    println!("\n📋 示例4: 完整评审流程");
    println!("-----------------------");

    let sample_diff = r#"
diff --git a/src/AuthService.java b/src/AuthService.java
index abc123..def456 100644
--- a/src/AuthService.java
+++ b/src/AuthService.java
@@ -1,5 +1,15 @@
 public class AuthService {
+    private UserRepository userRepo;
+    private JwtService jwtService;
+    
+    public String login(String username, String password) {
+        User user = userRepo.findByUsername(username);
+        if (user == null || !user.checkPassword(password)) {
+            throw new AuthException("Invalid credentials");
+        }
+        return jwtService.generateToken(user);
+    }
 }
    "#;

    let review_result = reviewer
        .review_changes(sample_diff, &["#123".to_string()], true)
        .await?;

    println!("📊 评审摘要:");
    println!("  总体评估: {}", review_result.summary.overall_assessment);
    println!(
        "  架构得分: {:.1}%",
        review_result.summary.architecture_score
    );
    println!(
        "  需求覆盖率: {:.1}%",
        review_result.summary.requirement_coverage
    );
    println!("  发现问题: {} 个", review_result.summary.total_insights);
    println!("  严重问题: {} 个", review_result.summary.critical_count);
    println!("  高风险问题: {} 个", review_result.summary.high_count);

    println!("\n🎯 主要建议:");
    for (i, recommendation) in review_result.recommendations.iter().enumerate() {
        println!("  {}. {}", i + 1, recommendation);
    }

    println!("\n✅ 安全评审演示完成！");
    println!("GitAI 为AI时代开发安全保驾护航");

    Ok(())
}
