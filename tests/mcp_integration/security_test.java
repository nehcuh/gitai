public class SecurityTest {
    public static void main(String[] args) {
        // SQL 注入漏洞
        String query = "SELECT * FROM users WHERE id = " + args[0];
        
        // 硬编码密码
        String password = "123456";
        
        // 路径遍历漏洞
        String file = "/etc/passwd";
        
        // XSS 漏洞
        String html = "<script>alert('xss')</script>";
        
        System.out.println(query);
        System.out.println(password);
        System.out.println(file);
        System.out.println(html);
    }
}