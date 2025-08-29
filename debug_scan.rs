// 调试 OpenGrep JSON 解析
use serde_json;

fn main() {
    let test_json = r#"{"version":"1.8.6","results":[{"check_id":"yaml.docker-compose.security.no-new-privileges.no-new-privileges","path":"../java-sec-code/docker-compose.yml","start":{"line":3,"col":5,"offset":28},"end":{"line":3,"col":8,"offset":31},"extra":{"metavars":{"$SERVICE":{"start":{"line":3,"col":5,"offset":28},"end":{"line":3,"col":8,"offset":31},"abstract_content":"jsc"}},"message":"Service 'jsc' allows for privilege escalation via setuid or setgid binaries. Add 'no-new-privileges:true' in 'security_opt' to prevent this.","metadata":{"cwe":["CWE-732: Incorrect Permission Assignment for Critical Resource"],"owasp":["A05:2021 - Security Misconfiguration","A06:2017 - Security Misconfiguration"]}},"severity":"WARNING","fingerprint":"8adc22399cd5ee11e3c7511fb80e8856039389d6e6b43bdcb510c960a1be1c2315ea524e258c3816c63cec0fb177fc6c82546fb9e9446fef4c877284a9d0d33c_0","lines":"    jsc:","is_ignored":false,"validation_state":"NO_VALIDATOR","engine_kind":"OSS"}]}"#;

    println!("🔍 解析测试JSON...");
    
    let v: serde_json::Value = serde_json::from_str(test_json).expect("JSON解析失败");
    println!("✅ JSON解析成功");
    
    println!("📄 JSON结构: {:?}", v);
    
    if let Some(results) = v.get("results").and_then(|r| r.as_array()) {
        println!("📋 找到 {} 个结果", results.len());
        
        for (i, item) in results.iter().enumerate() {
            println!("--- 结果 {} ---", i);
            
            let title = item["extra"]["message"].as_str().unwrap_or("Unknown issue");
            let file_path = item["path"].as_str().unwrap_or("");
            let line = item["start"]["line"].as_u64().unwrap_or(0);
            let rule_id = item["check_id"].as_str().unwrap_or("unknown");
            let severity_str = item["severity"].as_str().unwrap_or("WARNING");
            
            println!("  Title: {}", title);
            println!("  File: {}", file_path);
            println!("  Line: {}", line);
            println!("  Rule: {}", rule_id);
            println!("  Severity: {}", severity_str);
        }
    } else {
        println!("⚠️ 未找到 results 数组");
        println!("可用的顶级键: {:?}", v.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    }
}