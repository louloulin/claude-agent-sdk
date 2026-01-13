---
name: deployment-automation
description: "Automated deployment pipeline and CI/CD workflow expert"
version: "2.0.0"
author: "DevOps Team <devops@example.com>"
tags:
  - deployment
  - cicd
  - automation
  - devops
dependencies:
  - docker-helper
  - git-workflow
---

# Deployment Automation Skill

You are a deployment automation expert. Help design and implement CI/CD pipelines.

## CI/CD Platforms

### GitHub Actions
```yaml
name: deployment-automation

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '20'
      - run: npm ci
      - run: npm test
      - run: npm run lint

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: docker build -t myapp:${{ github.sha }} .
      - run: docker tag myapp:${{ github.sha }} myapp:latest

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - run: docker push myapp:${{ github.sha }}
      - run: kubectl set image deployment/myapp myapp=myapp:${{ github.sha }}
```

### GitLab CI
```yaml
stages:
  - test
  - build
  - deploy

test:
  stage: test
  image: node:20
  script:
    - npm ci
    - npm test
    - npm run lint

build:
  stage: build
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker build -t myapp:$CI_COMMIT_SHA .
    - docker login -u $CI_REGISTRY_USER -p $CI_REGISTRY_PASSWORD $CI_REGISTRY
    - docker push myapp:$CI_COMMIT_SHA

deploy:staging:
  stage: deploy
  script:
    - kubectl set image deployment/myapp myapp=myapp:$CI_COMMIT_SHA
  environment:
    name: staging
  only:
    - develop

deploy:production:
  stage: deploy
  script:
    - kubectl set image deployment/myapp myapp=myapp:$CI_COMMIT_SHA
  environment:
    name: production
  when: manual
  only:
    - main
```

## Deployment Strategies

### 1. Blue-Green Deployment
- Run two identical environments (blue and green)
- Deploy to inactive environment
- Test thoroughly
- Switch traffic with DNS/load balancer
- Keep old environment for rollback

**Pros:**
- Zero downtime
- Instant rollback
- Easy testing

**Cons:**
- Double resource cost
- Complex setup

### 2. Rolling Deployment
- Deploy to subset of instances
- Gradually replace old version
- Monitor health
- Continue or rollback

**Pros:**
- Resource efficient
- Gradual rollout
- Easy to implement

**Cons:**
- Slower rollback
- Version coexistence during rollout

### 3. Canary Deployment
- Deploy to small percentage of users
- Monitor metrics carefully
- Gradually increase traffic
- Full rollout or rollback

**Pros:**
- Risk mitigation
- Real user testing
- Easy rollback

**Cons:**
- Complex monitoring
- Longer deployment time

### 4. Feature Flags
- Deploy code behind flags
- Enable features incrementally
- Kill switch for problems
- A/B testing support

**Pros:**
- Maximum control
- Instant rollback
- Testing flexibility

**Cons:**
- Code complexity
- Flag management overhead

## Pipeline Stages

### Stage 1: Build
```yaml
build:
  script:
    - npm ci
    - npm run build
  artifacts:
    paths:
      - dist/
    expire_in: 1 week
```

### Stage 2: Test
```yaml
test:unit:
  script:
    - npm run test:unit

test:integration:
  script:
    - npm run test:integration

test:e2e:
  script:
    - npm run test:e2e
```

### Stage 3: Security Scan
```yaml
security:
  script:
    - npm audit
    - trivy fs .
    - semgrep --config=auto .
```

### Stage 4: Deploy
```yaml
deploy:staging:
  environment:
    name: staging
    url: https://staging.example.com
  script:
    - ./deploy.sh staging
  only:
    - develop

deploy:production:
  environment:
    name: production
    url: https://example.com
  script:
    - ./deploy.sh production
  when: manual
  only:
    - main
```

## Quality Gates

### Automated Checks
- [ ] All tests pass
- [ ] Code coverage > 80%
- [ ] No security vulnerabilities
- [ ] No linting errors
- [ ] Performance benchmarks met
- [ ] Documentation updated

### Manual Approvals
- [ ] Code review approved
- [ ] Product owner approval
- [ ] Security team sign-off (for sensitive changes)
- [ ] Performance team review (for major changes)

## Monitoring & Rollback

### Deployment Metrics
- Deployment frequency
- Lead time for changes
- Change failure rate
- Mean time to recovery (MTTR)

### Health Checks
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 30
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /ready
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 5
```

### Rollback Triggers
- Error rate > 1%
- Response time > 2x baseline
- CPU/Memory > 90%
- Failed health checks
- Manual trigger

## Secrets Management

### Environment Variables
```yaml
deploy:
  variables:
    DATABASE_URL: $DATABASE_URL
    API_KEY: $API_KEY
  script:
    - ./deploy.sh
```

### Secure Files
```yaml
deploy:
  script:
    - openssl aes-256-cbc -d -in secrets.tar.enc -out secrets.tar -k $KEY
    - tar xvf secrets.tar
    - ./deploy.sh
```

## Best Practices

✅ **DO:**
- Automate everything possible
- Use infrastructure as code
- Test in production-like environment
- Monitor deployments closely
- Have rollback plans
- Use semantic versioning
- Document deployment process
- Keep secrets secure

❌ **DON'T:**
- Deploy without testing
- Skip staging environment
- Ignore failed deployments
- Store secrets in repo
- Deploy on Fridays (unless necessary)
- Skip monitoring
- Rollback without investigation
