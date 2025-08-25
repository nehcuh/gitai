// Tree-sitter功能测试文件
// 用于验证各语言解析器的基本功能

use gitai::tree_sitter::{TreeSitterManager, SupportedLanguage};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🌳 Tree-sitter 功能测试");
    println!("========================");
    
    // 创建测试用的Java代码
    let java_code = r#"
public class HelloWorld {
    private String message;
    
    /**
     * 构造函数
     */
    public HelloWorld(String msg) {
        this.message = msg;
    }
    
    /**
     * 获取消息
     * @return 消息字符串
     */
    public String getMessage() {
        return message;
    }
    
    public void setMessage(String newMessage) {
        this.message = newMessage;
    }
    
    public static void main(String[] args) {
        HelloWorld hello = new HelloWorld("Hello, World!");
        System.out.println(hello.getMessage());
    }
}
"#;

    // 创建测试用的Rust代码
    let rust_code = r#"
/// 一个简单的问候结构体
pub struct Greeter {
    name: String,
}

impl Greeter {
    /// 创建新的Greeter实例
    pub fn new(name: String) -> Self {
        Self { name }
    }
    
    /// 生成问候消息
    pub fn greet(&self) -> String {
        format!("Hello, {}!", self.name)
    }
    
    /// 设置名称
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

fn main() {
    let mut greeter = Greeter::new("World".to_string());
    println!("{}", greeter.greet());
    
    greeter.set_name("Rust".to_string());
    println!("{}", greeter.greet());
}
"#;

    // 创建Tree-sitter管理器
    let mut manager = TreeSitterManager::new().await?;
    
    // 测试Java解析
    println!("\n📄 测试Java代码解析：");
    test_language_parsing(&mut manager, SupportedLanguage::Java, java_code)?;
    
    // 测试Rust解析
    println!("\n📄 测试Rust代码解析：");
    test_language_parsing(&mut manager, SupportedLanguage::Rust, rust_code)?;
    
    println!("\n✅ Tree-sitter 测试完成！");
    Ok(())
}

fn test_language_parsing(
    manager: &mut TreeSitterManager,
    language: SupportedLanguage,
    code: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("  语言: {:?}", language);
    
    match manager.analyze_structure(code, language) {
        Ok(summary) => {
            println!("  ✅ 解析成功！");
            println!("    函数数量: {}", summary.functions.len());
            println!("    类数量: {}", summary.classes.len());
            println!("    注释数量: {}", summary.comments.len());
            
            // 显示函数信息
            if !summary.functions.is_empty() {
                println!("    函数列表:");
                for func in &summary.functions {
                    println!("      - {} (第{}行)", func.name, func.line_start);
                }
            }
            
            // 显示类信息
            if !summary.classes.is_empty() {
                println!("    类列表:");
                for class in &summary.classes {
                    println!("      - {} (第{}行)", class.name, class.line_start);
                }
            }
            
            // 显示复杂度提示
            if !summary.complexity_hints.is_empty() {
                println!("    复杂度提示:");
                for hint in &summary.complexity_hints {
                    println!("      - {}", hint);
                }
            }
        }
        Err(e) => {
            println!("  ❌ 解析失败: {}", e);
        }
    }
    
    Ok(())
}
