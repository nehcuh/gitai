# Security Scanning with Semgrep

GitAI integrates [Semgrep](https://semgrep.dev) for comprehensive security scanning capabilities. This document explains how to use the security scanning features both locally and in CI/CD pipelines.

## Overview

The `gitai scan` command provides:
- **Static security analysis** using Semgrep's extensive rule database
- **Custom rule support** for project-specific security requirements
- **AI-powered analysis** of scan results for actionable insights
- **Integration with CI/CD** for automated security checks
- **Multiple output formats** for different use cases

## Quick Start

### Basic Usage

```bash
# Scan current directory with default rules
gitai scan

# Scan with detailed output
gitai scan --detailed

# Scan with AI analysis of results
gitai scan --detailed --ai

# Scan specific directory
gitai scan /path/to/code

# Scan with custom severity filter
gitai scan --severity=ERROR
```

### Advanced Usage

```bash
# Use custom Semgrep rules
gitai scan --rules="p/security-audit,p/secrets"

# Exclude specific patterns
gitai scan --exclude="tests,docs,target"

# Save results to file
gitai scan --output=security-report.json

# Show all findings including low severity
gitai scan --detailed --show-low

# Combine multiple options
gitai scan --detailed --ai --severity=WARNING --exclude="tests" --output=scan-results.json
```

## Command Options

| Option | Short | Description |
|--------|-------|-------------|
| `--path` | `-p` | Path to scan (default: current directory) |
| `--rules` | `-r` | Custom Semgrep rules/config to use |
| `--severity` | `-s` | Severity filter (ERROR, WARNING, INFO) |
| `--exclude` | `-e` | Patterns to exclude (comma-separated) |
| `--detailed` | `-d` | Show detailed findings |
| `--show-low` | | Show low severity issues |
| `--ai` | `-a` | Enable AI analysis of results |
| `--output` | `-o` | Output file for results |

## Semgrep Rules

### Default Rules

GitAI uses Semgrep's `auto` configuration by default, which includes:
- Language-specific security rules
- Common vulnerability patterns
- Best practice violations

### Custom Rules

The project includes custom rules in `.semgrep.yml`:

1. **Hardcoded Secrets Detection**
   - Detects potential secrets in Rust code
   - Severity: ERROR

2. **Unsafe Block Review**
   - Flags unsafe Rust blocks for review
   - Severity: WARNING

3. **Command Injection Prevention**
   - Identifies potential command injection vulnerabilities
   - Severity: WARNING

4. **Error Handling Improvements**
   - Suggests better error handling patterns
   - Severity: INFO

5. **Code Quality Checks**
   - Finds TODO/FIXME comments
   - Severity: INFO

### Adding Custom Rules

To add project-specific rules, edit `.semgrep.yml`:

```yaml
rules:
  - id: my-custom-rule
    pattern: |
      dangerous_function($ARG)
    message: "Avoid using dangerous_function"
    languages: [rust]
    severity: ERROR
```

## AI Analysis

When using the `--ai` flag, GitAI provides:

1. **Security Risk Assessment**
   - Prioritization of findings by business impact
   - Risk scoring and categorization

2. **Remediation Suggestions**
   - Specific fix recommendations
   - Code examples for secure alternatives

3. **Pattern Analysis**
   - Identification of recurring security issues
   - Architectural security recommendations

4. **Compliance Insights**
   - Mapping to security frameworks (OWASP, CWE)
   - Regulatory compliance considerations

## CI/CD Integration

### GitHub Actions

The project includes a GitHub Actions workflow (`.github/workflows/semgrep.yml`) that:

- Runs on every push and pull request
- Uses comprehensive security rulesets
- Uploads results to GitHub Security tab
- Comments on PRs with scan summaries
- Runs weekly scheduled scans

### Configuration

```yaml
# Enable/disable CI scanning
on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 2 * * 0'  # Weekly on Sundays
```

### Custom CI Setup

For other CI systems, use:

```bash
# Install Semgrep
pip install semgrep

# Run scan
semgrep --config=auto --json --output=results.json .

# Use GitAI for analysis
gitai scan --ai --output=analysis.txt
```

## Output Formats

### Console Output

```
🔍 Starting Semgrep code scan...
Running: semgrep --config=auto --json --verbose .

📊 Scan Results
==================================================

Summary:
  🔴 Critical: 2
  🟠 High: 5
  🟡 Medium: 8
  🔵 Low: 3

🔴 CRITICAL 2 Issues:
----------------------------------------

1. hardcoded-secret (src/config.rs:45)
   Hardcoded secret detected in configuration
   Fix: Use environment variables for sensitive data
```

### JSON Output

```json
{
  "results": [
    {
      "check_id": "hardcoded-secret",
      "path": "src/config.rs",
      "start": {"line": 45, "col": 8},
      "end": {"line": 45, "col": 32},
      "extra": {
        "message": "Hardcoded secret detected",
        "severity": "ERROR"
      }
    }
  ]
}
```

## Best Practices

### Development Workflow

1. **Pre-commit Scanning**
   ```bash
   # Add to git hooks
   gitai scan --severity=ERROR
   ```

2. **Regular Security Reviews**
   ```bash
   # Weekly comprehensive scan
   gitai scan --detailed --ai --output=weekly-scan.json
   ```

3. **Feature Branch Scanning**
   ```bash
   # Before creating PR
   gitai scan --detailed --exclude="tests"
   ```

### Rule Management

1. **Start Conservative**
   - Begin with ERROR-level rules only
   - Gradually add WARNING and INFO rules

2. **Customize for Your Project**
   - Add domain-specific security rules
   - Exclude false positives appropriately

3. **Regular Rule Updates**
   - Keep Semgrep rules updated
   - Review and update custom rules

### Performance Optimization

1. **Exclude Unnecessary Files**
   ```yaml
   exclude:
     - "target/**"
     - "tests/**"
     - "docs/**"
   ```

2. **Use Targeted Scans**
   ```bash
   # Scan only changed files
   gitai scan --path="$(git diff --name-only HEAD~1)"
   ```

3. **Parallel Scanning**
   ```bash
   # Semgrep automatically uses multiple cores
   semgrep --jobs=4 --config=auto .
   ```

## Troubleshooting

### Common Issues

1. **Semgrep Not Installed**
   ```bash
   # Install Semgrep
   pip install semgrep
   
   # Or use GitAI's auto-install
   gitai scan  # Will prompt to install if missing
   ```

2. **High False Positive Rate**
   - Review and update `.semgrep.yml` exclude patterns
   - Use more specific rules
   - Add suppressions for known false positives

3. **Performance Issues**
   - Exclude large directories (target/, node_modules/)
   - Use targeted scanning for specific paths
   - Consider running scans on smaller changesets

4. **AI Analysis Failures**
   - Check AI configuration in GitAI config
   - Verify network connectivity
   - Review API rate limits

### Getting Help

1. **GitAI Help**
   ```bash
   gitai scan --help
   gitai --help
   ```

2. **Semgrep Documentation**
   - [Official Semgrep Docs](https://semgrep.dev/docs/)
   - [Rule Writing Guide](https://semgrep.dev/docs/writing-rules/overview/)

3. **Community Resources**
   - [Semgrep Community Rules](https://semgrep.dev/explore)
   - [Security Best Practices](https://semgrep.dev/docs/semgrep-ci/overview/)

## Security Considerations

1. **Sensitive Data**
   - Never commit scan results containing sensitive information
   - Use secure storage for scan reports
   - Review output before sharing

2. **Rule Validation**
   - Test custom rules thoroughly
   - Validate rule accuracy before deployment
   - Monitor for false positives

3. **Access Control**
   - Limit access to detailed scan results
   - Use appropriate CI/CD permissions
   - Secure API tokens and credentials

## Integration Examples

### Pre-commit Hook

```bash
#!/bin/sh
# .git/hooks/pre-commit
gitai scan --severity=ERROR
if [ $? -ne 0 ]; then
    echo "Security scan failed. Please fix issues before committing."
    exit 1
fi
```

### IDE Integration

```bash
# VS Code task
{
    "label": "GitAI Security Scan",
    "type": "shell",
    "command": "gitai scan --detailed --ai",
    "group": "build"
}
```

### Makefile Integration

```makefile
.PHONY: security-scan
security-scan:
	gitai scan --detailed --output=security-report.json

.PHONY: security-check
security-check:
	gitai scan --severity=ERROR
```

This comprehensive security scanning integration makes GitAI a powerful tool for maintaining code security throughout the development lifecycle.