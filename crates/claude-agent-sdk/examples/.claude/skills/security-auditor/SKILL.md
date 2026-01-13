---
name: security-auditor
description: "Security vulnerability detection and remediation expert for applications and infrastructure"
version: "1.4.0"
author: "Security Team <security@example.com>"
tags:
  - security
  - audit
  - vulnerability
  - compliance
dependencies:
  - code-reviewer
  - docker-helper
---

# Security Auditor Skill

You are a security expert. Identify vulnerabilities and recommend security improvements.

## Security Categories

### 1. Application Security

#### OWASP Top 10
- **A01:2021 - Broken Access Control**
  - Verify user permissions for all actions
  - Implement proper authentication
  - Check for IDOR vulnerabilities
  - Validate API access controls

- **A02:2021 - Cryptographic Failures**
  - Encrypt sensitive data at rest
  - Use TLS in transit
  - Implement proper key management
  - Avoid weak algorithms

- **A03:2021 - Injection**
  - SQL Injection: Use parameterized queries
  - XSS: Sanitize and escape user input
  - Command Injection: Avoid shell commands
  - NoSQL Injection: Validate queries

- **A04:2021 - Insecure Design**
  - Implement threat modeling
  - Design secure by default
  - Implement defense in depth
  - Plan security controls

- **A05:2021 - Security Misconfiguration**
  - Remove default credentials
  - Disable unnecessary features
  - Keep software updated
  - Secure configuration management

#### Code Review Checklist
```rust
// ❌ Bad: SQL Injection vulnerability
let query = format!("SELECT * FROM users WHERE id = {}", user_input);

// ✅ Good: Parameterized query
let query = "SELECT * FROM users WHERE id = $1";
client.execute(query, &[&user_id]);

// ❌ Bad: Hardcoded secrets
const API_KEY = "sk_live_abc123";

// ✅ Good: Environment variables
let api_key = std::env::var("API_KEY")?;

// ❌ Bad: Unsafe deserialization
let obj = serde_json::from_str::<User>(user_input)?;

// ✅ Good: Validate before deserialization
let validated = validate_and_sanitize(user_input)?;
let obj = serde_json::from_str::<User>(&validated)?;
```

### 2. Infrastructure Security

#### Docker Security
```dockerfile
# ✅ Use specific version tags
FROM python:3.11-slim

# ✅ Run as non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser
USER appuser

# ✅ Use minimal base images
FROM gcr.io/distroless/python3-debian11

# ❌ Avoid: Running as root
USER root

# ❌ Avoid: Using latest
FROM python:latest
```

#### Kubernetes Security
```yaml
# ✅ Security context
apiVersion: v1
kind: Pod
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    fsGroup: 1000
    seccompProfile:
      type: RuntimeDefault
  containers:
  - name: app
    securityContext:
      allowPrivilegeEscalation: false
      capabilities:
        drop:
        - ALL
      readOnlyRootFilesystem: true
```

#### Network Security
```yaml
# Network policies
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: deny-all
spec:
  podSelector: {}
  policyTypes:
  - Ingress
  - Egress
```

### 3. Secrets Management

#### Best Practices
- Never commit secrets to version control
- Use secret management systems (Vault, AWS Secrets Manager)
- Rotate credentials regularly
- Implement principle of least privilege
- Audit secret access

#### Tools
```bash
# Detect secrets in code
git-secrets scan
trufflehog git https://github.com/org/repo
gitleaks detect --source .

# Environment variables
export DATABASE_URL="postgresql://..."
export API_KEY=$(vault kv get -field=key secret/api)
```

### 4. Authentication & Authorization

#### JWT Best Practices
```rust
// ✅ Use strong algorithms
let algorithm = Algorithm::HS512;

// ✅ Set reasonable expiration
let expiration = Utc::now() + Duration::hours(24);

// ✅ Validate all claims
jwt.decode::<Claims>(token, &secret, &Validation::new(&algorithm))?;

// ❌ Avoid: Weak algorithms
let algorithm = Algorithm::HS256; // Use HS512 or RS512

// ❌ Avoid: No expiration
let expiration = None;
```

#### OAuth2 / OIDC
```rust
// Validate all OAuth2 parameters
- state parameter (CSRF protection)
- redirect_uri (must match registered)
- response_type (code for server-side)
- scope (principle of least privilege)
- nonce (for OpenID Connect)
```

### 5. Dependency Security

#### Supply Chain Security
```bash
# Scan for vulnerabilities
npm audit
cargo audit
safety check
pip-audit

# SBOM generation
syft ./app -o sbom > sbom.json

# Check for malicious packages
osv-scanner --sbom=sbom.json
```

#### Dependency Updates
```
Prioritize updates based on:
- Critical vulnerabilities (patch immediately)
- High vulnerabilities (patch within 7 days)
- Medium vulnerabilities (patch within 30 days)
- Low vulnerabilities (patch in next release)
```

### 6. Logging & Monitoring

#### Security Events to Log
```
Authentication:
- Failed login attempts
- Successful logins
- Password changes
- Permission changes

Authorization:
- Access denied
- Privilege escalation attempts
- Resource access violations

Data:
- Sensitive data access
- Data export/download
- Configuration changes

System:
- Security control failures
- System errors
- Anomalous behavior
```

#### Security Metrics
- Mean Time to Detect (MTTD)
- Mean Time to Respond (MTTR)
- Number of vulnerabilities
- Patch compliance rate
- Security incident frequency

### 7. Compliance

#### GDPR Requirements
- Data minimization
- Consent management
- Right to be forgotten
- Data portability
- Breach notification

#### SOC 2 Controls
- Access control
- Encryption
- Monitoring
- Change management
- Incident response

#### PCI DSS
- Network security
- Data protection
- Vulnerability management
- Access control
- Monitoring and testing

## Security Review Process

### Phase 1: Automated Scanning
```bash
# Static Application Security Testing (SAST)
semgrep --config=auto .
sonar-scanner

# Software Composition Analysis (SCA)
npm audit
cargo audit

# Container scanning
trivy image myapp:latest
docker scout cves myapp:latest

# Infrastructure as Code scanning
tfsec .
checkov -d .
```

### Phase 2: Manual Review
- Review authentication flows
- Check authorization logic
- Validate input sanitization
- Review error handling
- Check logging practices

### Phase 3: Penetration Testing
- Black-box testing
- White-box testing
- API security testing
- Network penetration testing

### Phase 4: Remediation
- Prioritize findings
- Implement fixes
- Verify remediation
- Update documentation

## Common Vulnerabilities

### Web Application Security
```javascript
// ❌ XSS Vulnerability
div.innerHTML = userInput;

// ✅ Safe alternative
div.textContent = userInput;
// or
div.innerHTML = DOMPurify.sanitize(userInput);

// ❌ SQL Injection
db.query(`SELECT * FROM users WHERE id = ${id}`);

// ✅ Parameterized query
db.query('SELECT * FROM users WHERE id = ?', [id]);
```

### API Security
```
Common Issues:
- Missing rate limiting
- No authentication on sensitive endpoints
- Exposing internal IDs (IDOR)
- Missing CORS validation
- Inadequate input validation
- Verbose error messages
```

### Infrastructure Security
```
Common Issues:
- Open ports/firewalls
- Default credentials
- Unpatched systems
- Weak encryption
- Missing network segmentation
- Exposed management interfaces
```

## Security Testing Tools

### SAST Tools
- **Semgrep** - Customizable rule engine
- **SonarQube** - Code quality and security
- **CodeQL** - Semantic code analysis
- **Bandit** - Python security linter

### DAST Tools
- **OWASP ZAP** - Web application scanner
- **Burp Suite** - Security testing platform
- **Nuclei** - Vulnerability scanner

### Container Security
- **Trivy** - Vulnerability scanner
- **Clair** - Container analysis
- **Docker Scout** - Docker security

### Dependency Scanning
- **Snyk** - Dependency vulnerability scanner
- **Dependabot** - Automated dependency updates
- **OWASP Dependency-Check**

## Security Best Practices

✅ **DO:**
- Implement defense in depth
- Use security headers (CSP, HSTS, X-Frame-Options)
- Enable HTTPS everywhere
- Implement rate limiting
- Validate and sanitize all input
- Use parameterized queries
- Implement proper authentication
- Follow principle of least privilege
- Keep dependencies updated
- Log security events
- Implement security monitoring
- Have incident response plan

❌ **DON'T:**
- Trust user input
- Roll your own crypto
- Store passwords in plain text
- Use weak encryption
- Ignore security warnings
- Disable security controls
- Expose sensitive data
- Use hardcoded credentials
- Forget to audit access
- Skip security reviews
- Ignore vulnerabilities
- Disable logging

## Incident Response

### Preparation
1. Establish incident response team
2. Define communication channels
3. Prepare response playbooks
4. Set up monitoring and alerting
5. Conduct regular drills

### Detection & Analysis
1. Identify security incident
2. Classify severity level
3. Contain the threat
4. Preserve evidence
5. Determine root cause

### Containment
1. Isolate affected systems
2. Block malicious activity
3. Change compromised credentials
4. Implement temporary controls

### Eradication
1. Remove malware/vulnerability
2. Patch security flaws
3. Update security controls
4. Validate remediation

### Recovery
1. Restore from clean backups
2. Monitor for recurrence
3. Document lessons learned
4. Update security practices

## Resources

### Documentation
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [Security Headers](https://securityheaders.com/)

### Tools
- [OWASP ZAP](https://www.zaproxy.org/)
- [Trivy](https://aquasecurity.github.io/trivy/)
- [Semgrep](https://semgrep.dev/)

### Communities
- [OWASP](https://owasp.org/)
- [r/netsec on Reddit](https://reddit.com/r/netsec)
- [Bug Bounty Platforms](https://hackerone.com/)
