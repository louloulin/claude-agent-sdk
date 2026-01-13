# Security Policy

## 🔒 Security Best Practices

This document outlines security best practices for using and contributing to the Claude Agent SDK for Rust.

### API Key Management

**Critical Rules**:
1. **NEVER commit API keys** to version control
2. **ALWAYS use environment variables** for sensitive data
3. **Rotate keys regularly** (at least every 90 days)
4. **Monitor usage** in the Anthropic dashboard

### Getting Your API Key Safely

1. Visit [https://console.anthropic.com/](https://console.anthropic.com/)
2. Generate a new API key
3. Store it securely:
   ```bash
   # Option 1: Environment variable (recommended)
   export ANTHROPIC_API_KEY="your_key_here"

   # Option 2: .env file (NEVER commit this!)
   echo 'ANTHROPIC_API_KEY=sk-ant-...' > .env
   ```

### Environment Setup

The repository provides `.env.example` as a template:

```bash
# Copy the template
cp .env.example .env

# Edit with your actual key
nano .env
```

**⚠️ IMPORTANT**: The `.gitignore` file prevents `.env` from being committed. Verify with `git status`.

### Git Security

#### Checking for Committed Secrets

Before pushing, run:

```bash
# Check for API keys in git history
git log --all --full-history --source -- "*secret*" "*key*" "*.env"

# Search for Anthropic keys
git grep "sk-ant-"

# Use git-secrets for automated detection
git secrets --install
git secrets --register-aws
git secrets --add 'sk-ant-[a-zA-Z0-9\-_]{36}'
git secrets --scan
```

#### Removing Accidentally Committed Keys

If you accidentally commit a key:

```bash
# 1. Remove the file from git
git rm --cached .env

# 2. Commit the removal
git commit -m "Remove accidentally committed .env"

# 3. Rotate the compromised key immediately at https://console.anthropic.com/

# 4. For complete history removal (advanced)
# WARNING: This rewrites git history
git filter-branch --force --index-filter \
  "git rm --cached --ignore-unmatch .env" \
  --prune-empty --tag-name-filter cat -- --all

# 5. Force push (only if necessary!)
git push origin --force --all
```

### Security Checklist

Before committing or pushing:

- [ ] No `.env` files in `git status`
- [ ] No hardcoded keys (`git grep "sk-ant-"` returns only examples)
- [ ] `.env.example` updated with new variables
- [ ] All secrets use environment variables
- [ ] No sensitive data in logs or error messages
- [ ] Dependencies are up-to-date (`cargo update`)

### Production Deployment

For production environments:

1. **Use secret management tools**:
   - AWS Secrets Manager
   - HashiCorp Vault
   - Azure Key Vault
   - Google Secret Manager

2. **Environment-specific configuration**:
   ```rust
   // Load from environment only
   let api_key = std::env::var("ANTHROPIC_API_KEY")
       .expect("ANTHROPIC_API_KEY must be set");
   ```

3. **Rate limiting & monitoring**:
   - Implement request rate limits
   - Monitor API usage for anomalies
   - Set up alerts for unusual activity

4. **Key rotation strategy**:
   - Rotate keys every 90 days
   - Have a rotation procedure documented
   - Test rotation process in staging

### Reporting Security Issues

**If you discover a security vulnerability**:

1. **DO NOT** open a public issue
2. **DO** email: security@example.com (replace with actual email)
3. **DO** include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

4. **Response timeline**:
   - Initial response: Within 48 hours
   - Remediation: As soon as possible
   - Public disclosure: After fix is deployed

### Dependency Security

Keep dependencies updated:

```bash
# Check for security advisories
cargo audit

# Update dependencies
cargo update

# Check for outdated crates
cargo outdated
```

### Code Security Practices

When contributing:

1. **Input validation**:
   ```rust
   // Validate user input
   if input.len() > MAX_INPUT_SIZE {
       return Err(Error::InputTooLarge);
   }
   ```

2. **Error handling**:
   ```rust
   // Don't expose sensitive data in errors
   Err(Error::ApiError("Request failed".to_string()))
   // NOT: Err(Error::ApiError(format!("Failed with key: {}", key)))
   ```

3. **Logging security**:
   ```rust
   // Use tracing for secure logging
   use tracing::{info, warn, error};

   info!("API request initiated");
   // NOT: info!("API request with key: {}", api_key)
   ```

### Additional Resources

- [Anthropic API Security](https://docs.anthropic.com/claude/docs/security)
- [OWASP Rust Security](https://owasp.org/www-project-rust-security-guide/)
- [Rust Security Best Practices](https://doc.rust-lang.org/book/ch12-00-an-io-project.html)

---

**Remember**: Security is everyone's responsibility. When in doubt, ask!
