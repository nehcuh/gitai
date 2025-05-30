# Security Scanning with GitAI

GitAI provides comprehensive security scanning capabilities using both local AST-based analysis and external tools.

## Overview

GitAI's security scanning uses a two-tier approach:

1. **Primary**: Tree-sitter based local AST analysis (no external dependencies)
2. **Fallback**: Semgrep community scanner (when tree-sitter finds no issues)

## Local Tree-sitter Security Scanner

The tree-sitter security scanner provides local, privacy-focused security analysis without sending code to external servers.

### Supported Languages

- **Rust**: Unsafe code blocks, unwrap() usage, panic patterns
- **JavaScript/TypeScript**: XSS vulnerabilities, eval usage, DOM manipulation
- **Python**: SQL injection, command injection, hardcoded secrets
- **Java**: SQL injection, hardcoded passwords, unsafe deserialization
- **Go**: SQL injection, command injection, hardcoded credentials
- **C/C++**: Buffer overflows, format string vulnerabilities, unsafe functions

### Security Rules

#### Cross-Language Rules
- **Hardcoded Secrets**: Detects API keys, passwords, tokens in source code
- **SQL Injection**: Identifies unsafe SQL query construction
- **Command Injection**: Finds unsafe system command execution
- **XSS Vulnerabilities**: Detects unsafe HTML/DOM manipulation

#### Language-Specific Rules
- **Rust**: `unsafe` blocks, `.unwrap()` calls, `panic!()` usage
- **JavaScript**: `eval()`, `innerHTML`, `document.write()`
- **Python**: `exec()`, `eval()`, `os.system()`, hardcoded credentials
- **Java**: `Runtime.exec()`, hardcoded passwords, unsafe deserialization
- **Go**: `exec.Command()`, SQL string concatenation
- **C/C++**: `strcpy()`, `sprintf()`, `gets()`, buffer operations

## Usage

### Basic Security Scan

```bash
# Scan entire repository
gitai scan

# Scan with detailed output
gitai scan --detailed

# Scan specific file
gitai scan --file src/main.rs

# Scan with severity filtering
gitai scan --severity HIGH
gitai scan --severity MEDIUM
gitai scan --severity LOW
```

### Advanced Options

```bash
# Exclude files/patterns
gitai scan --exclude "test_*.rs" --exclude "*.test.js"

# Output to JSON file
gitai scan --output security_report.json

# Enable AI analysis (requires AI configuration)
gitai scan --ai-analysis

# Combine options
gitai scan --detailed --severity HIGH --exclude "tests/" --output report.json
```

### Output Formats

#### Console Output
```
📊 Security Scan Results
==================================================

Summary:
  📁 Files scanned: 32
  🔍 Total findings: 8
  🟡 Medium: 8

🟡 MEDIUM 8 Issues:
----------------------------------------

1. Unsafe Rust Code (./src/config.rs:1016)
   Unsafe blocks bypass Rust's safety guarantees
   CWE: CWE-119
   Recommendation: Ensure unsafe code is necessary and properly reviewed
   Code: unsafe { std::env::set_var("HOME", home_path.to_str().unwrap()) }
```

#### JSON Output
```json
{
  "findings": [
    {
      "id": "rust-unsafe-1",
      "title": "Unsafe Rust Code",
      "description": "Unsafe blocks bypass Rust's safety guarantees",
      "severity": "Medium",
      "file_path": "./src/config.rs",
      "line_start": 1016,
      "line_end": 1016,
      "column_start": 9,
      "column_end": 74,
      "code_snippet": "unsafe { std::env::set_var(\"HOME\", home_path.to_str().unwrap()) }",
      "recommendation": "Ensure unsafe code is necessary and properly reviewed",
      "cwe_id": "CWE-119",
      "owasp_category": null
    }
  ]
}
```

## Severity Levels

- **🔴 CRITICAL**: Immediate security risks requiring urgent attention
- **🟠 HIGH**: Significant security vulnerabilities
- **🟡 MEDIUM**: Moderate security concerns
- **🔵 LOW**: Minor security improvements

## Security Rule Categories

### CWE (Common Weakness Enumeration) Coverage

- **CWE-79**: Cross-site Scripting (XSS)
- **CWE-89**: SQL Injection
- **CWE-94**: Code Injection
- **CWE-119**: Buffer Overflow
- **CWE-200**: Information Exposure
- **CWE-259**: Hard-coded Password
- **CWE-327**: Weak Cryptography
- **CWE-502**: Unsafe Deserialization

### OWASP Top 10 Alignment

The security rules are designed to detect vulnerabilities from the OWASP Top 10:

1. **A01 - Broken Access Control**
2. **A02 - Cryptographic Failures**
3. **A03 - Injection**
4. **A04 - Insecure Design**
5. **A05 - Security Misconfiguration**
6. **A06 - Vulnerable Components**
7. **A07 - Authentication Failures**
8. **A08 - Software Integrity Failures**
9. **A09 - Logging Failures**
10. **A10 - Server-Side Request Forgery**

## Privacy and Security

### Local Analysis Benefits

- **No Data Transmission**: Code never leaves your machine
- **Offline Capability**: Works without internet connection
- **Fast Analysis**: Local AST parsing is extremely fast
- **No Rate Limits**: Unlimited scans without API restrictions

### Semgrep Fallback

When tree-sitter finds no issues, GitAI optionally runs Semgrep as a fallback:

- Uses Semgrep community rules
- Provides additional coverage for edge cases
- Can be disabled if privacy is a concern

## Configuration

### Disabling Semgrep Fallback

To use only local tree-sitter analysis:

```bash
# Set environment variable
export GITAI_DISABLE_SEMGREP=true

# Or modify your config.toml
[security]
disable_semgrep_fallback = true
```

### Custom Security Rules

You can extend the security scanner with custom rules by modifying the `SecurityRule` definitions in `src/tree_sitter_analyzer/security.rs`.

## Integration with CI/CD

### GitHub Actions

```yaml
name: Security Scan
on: [push, pull_request]

jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install GitAI
        run: cargo install --git https://github.com/nehcuh/gitai.git
      - name: Run Security Scan
        run: gitai scan --output security-report.json
      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: security-report
          path: security-report.json
```

### Exit Codes

- `0`: No security issues found
- `1`: Security issues found (check output for details)
- `2`: Scan failed due to error

## Troubleshooting

### Common Issues

1. **Language Not Supported**: Ensure the file extension is recognized
2. **Parse Errors**: Check for syntax errors in source files
3. **False Positives**: Use `--exclude` to skip problematic files

### Debug Mode

```bash
# Enable verbose logging
RUST_LOG=debug gitai scan --detailed
```

## Contributing

To add new security rules:

1. Define the rule in `src/tree_sitter_analyzer/security.rs`
2. Add tree-sitter query patterns for the target language
3. Test with sample vulnerable code
4. Update documentation

## Examples

### Detecting SQL Injection in Python

```python
# This would be detected
query = "SELECT * FROM users WHERE id = " + user_id
cursor.execute(query)

# This is safe
cursor.execute("SELECT * FROM users WHERE id = ?", (user_id,))
```

### Detecting XSS in JavaScript

```javascript
// This would be detected
element.innerHTML = userInput;

// This is safer
element.textContent = userInput;
```

### Detecting Unsafe Rust Code

```rust
// This would be detected
unsafe { 
    std::env::set_var("HOME", path.to_str().unwrap()) 
}

// Consider safer alternatives
std::env::set_var("HOME", path.to_str().unwrap_or_default());
```

## Performance

- **Tree-sitter Analysis**: ~100-1000 files/second depending on file size
- **Memory Usage**: Minimal, processes files individually
- **Disk Usage**: No temporary files created
- **Network**: No network requests for local analysis

## Comparison with Other Tools

| Feature | GitAI Tree-sitter | Semgrep | CodeQL | SonarQube |
|---------|------------------|---------|---------|-----------|
| Local Analysis | ✅ | ✅ | ❌ | ❌ |
| No Data Upload | ✅ | ✅ | ❌ | ❌ |
| Speed | ⚡ Fast | 🚀 Very Fast | 🐌 Slow | 🐌 Slow |
| Language Support | 7+ | 20+ | 10+ | 25+ |
| Custom Rules | ✅ | ✅ | ✅ | ✅ |
| CI/CD Integration | ✅ | ✅ | ✅ | ✅ |
| Free Tier | ✅ Unlimited | ✅ Limited | ❌ | ✅ Limited |

GitAI's tree-sitter scanner provides the best balance of privacy, speed, and effectiveness for local security analysis.