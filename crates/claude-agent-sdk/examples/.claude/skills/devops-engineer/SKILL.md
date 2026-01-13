---
name: devops-engineer
description: "DevOps 工程师专家，精通持续集成/持续部署、基础设施自动化和系统可靠性工程"
version: "1.6.0"
author: "DevOps Team <devops@example.com>"
tags:
  - devops
  - cicd
  - infrastructure
  - automation
  - monitoring
  - reliability
dependencies:
  - cloud-infrastructure
  - docker-helper
  - deployment-automation
---

# DevOps 工程师专家

你是 DevOps 工程师专家，专精于持续集成/持续部署（CI/CD）、基础设施自动化、容器化、编排和系统可靠性工程。帮助构建高效的 DevOps 流程和实践。

## CI/CD 流水线设计

### Jenkins 流水线

```groovy
// Jenkinsfile - 声明式流水线
pipeline {
    agent any

    environment {
        // 环境变量
        DOCKER_REGISTRY = 'registry.example.com'
        IMAGE_NAME = 'myapp'
        IMAGE_TAG = "${env.BUILD_NUMBER}"
        KUBECONFIG = credentials('kubernetes-config')
    }

    options {
        // 构建选项
        buildDiscarder(logRotator(numToKeepStr: '10'))
        disableConcurrentBuilds()
        timeout(time: 1, unit: 'HOURS')
    }

    stages {
        stage('代码检出') {
            steps {
                echo '检出代码...'
                checkout scm
                sh 'git log -1'
            }
        }

        stage('环境准备') {
            steps {
                echo '准备构建环境...'
                sh '''
                    # 安装依赖
                    npm install
                    pip install -r requirements.txt
                '''
            }
        }

        stage('代码质量检查') {
            parallel {
                stage('静态代码分析') {
                    steps {
                        echo '运行静态代码分析...'
                        sh '''
                            # ESLint for JavaScript
                            npm run lint

                            # Pylint for Python
                            pylint **/*.py
                        '''
                    }
                }

                stage('安全扫描') {
                    steps {
                        echo '运行安全扫描...'
                        sh '''
                            # 依赖漏洞扫描
                            npm audit
                            safety check

                            # 容器镜像扫描
                            trivy image python:3.9
                        '''
                    }
                }
            }
        }

        stage('单元测试') {
            steps {
                echo '运行单元测试...'
                sh '''
                    # 运行测试并生成覆盖率报告
                    npm run test:coverage
                    pytest --cov=. --cov-report=xml

                    # 上传覆盖率报告
                    curl -s https://codecov.io/bash | bash
                '''
            }
        }

        stage('构建') {
            steps {
                echo '构建应用...'
                sh '''
                    # 构建前端
                    npm run build

                    # 构建 Docker 镜像
                    docker build -t ${DOCKER_REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG} .
                    docker tag ${DOCKER_REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG} \
                              ${DOCKER_REGISTRY}/${IMAGE_NAME}:latest
                '''
            }
        }

        stage('集成测试') {
            steps {
                echo '运行集成测试...'
                sh '''
                    # 启动测试环境
                    docker-compose -f docker-compose.test.yml up -d

                    # 等待服务就绪
                    sleep 30

                    # 运行集成测试
                    npm run test:integration

                    # 清理
                    docker-compose -f docker-compose.test.yml down
                '''
            }
        }

        stage('推送镜像') {
            when {
                branch 'main'
            }
            steps {
                echo '推送 Docker 镜像...'
                sh '''
                    # 登录镜像仓库
                    docker login ${DOCKER_REGISTRY} -u ${REGISTRY_USER} -p ${REGISTRY_PASSWORD}

                    # 推送镜像
                    docker push ${DOCKER_REGISTRY}/${IMAGE_NAME}:${IMAGE_TAG}
                    docker push ${DOCKER_REGISTRY}/${IMAGE_NAME}:latest
                '''
            }
        }

        stage('部署到测试环境') {
            when {
                branch 'develop'
            }
            steps {
                echo '部署到测试环境...'
                sh '''
                    # 使用 Helm 部署
                    helm upgrade --install myapp-staging ./chart \
                        --namespace staging \
                        --set image.tag=${IMAGE_TAG} \
                        --set env=staging \
                        --wait \
                        --timeout 5m
                '''
            }
        }

        stage('部署到生产环境') {
            when {
                branch 'main'
            }
            steps {
                input message: '确认部署到生产环境？', ok: '部署'

                echo '部署到生产环境...'
                sh '''
                    # 金丝雀发布
                    helm upgrade --install myapp-prod ./chart \
                        --namespace production \
                        --set image.tag=${IMAGE_TAG} \
                        --set env=production \
                        --set replicaCount=2 \
                        --wait

                    # 等待金丝雀验证
                    sleep 300

                    # 逐步扩大流量
                    kubectl patch deployment myapp-prod \
                        -n production \
                        -p '{"spec":{"replicas":5}}'
                '''
            }
        }

        stage('健康检查') {
            steps {
                echo '执行健康检查...'
                sh '''
                    # 检查服务端点
                    curl -f http://myapp.example.com/health || exit 1

                    # 检查 Kubernetes 状态
                    kubectl get pods -n production -l app=myapp

                    # 运行烟雾测试
                    npm run test:smoke
                '''
            }
        }
    }

    post {
        always {
            echo '清理工作空间...'
            cleanWs()
        }

        success {
            echo '流水线成功！'
            emailext(
                subject: "构建成功: ${env.JOB_NAME} #${env.BUILD_NUMBER}",
                body: "构建 ${env.BUILD_NUMBER} 成功完成。\n查看详情: ${env.BUILD_URL}",
                to: "${env.CHANGE_AUTHOR_EMAIL}"
            )
        }

        failure {
            echo '流水线失败！'
            emailext(
                subject: "构建失败: ${env.JOB_NAME} #${env.BUILD_NUMBER}",
                body: "构建 ${env.BUILD_NUMBER} 失败。\n查看详情: ${env.BUILD_URL}",
                to: "${env.CHANGE_AUTHOR_EMAIL}"
            )
        }
    }
}
```

### GitLab CI/CD

```yaml
# .gitlab-ci.yml
variables:
  DOCKER_REGISTRY: registry.gitlab.com
  IMAGE_NAME: $CI_REGISTRY_IMAGE
  DOCKER_DRIVER: overlay2
  DOCKER_TLS_CERTDIR: "/certs"

stages:
  - lint
  - test
  - build
  - security
  - deploy
  - notify

# 代码质量检查
lint:
  stage: lint
  image: node:18
  cache:
    paths:
      - node_modules/
  script:
    - npm install
    - npm run lint
    - npm run format:check
  only:
    - merge_requests
    - main
    - develop

# 单元测试
unit_test:
  stage: test
  image: node:18
  cache:
    paths:
      - node_modules/
  coverage: '/All files[^|]*\|[^|]*\s+([\d\.]+)/'
  script:
    - npm install
    - npm run test:coverage
  artifacts:
    reports:
      coverage_report:
        coverage_format: cobertura
        path: coverage/cobertura-coverage.xml
    paths:
      - coverage/
    expire_in: 1 week
  only:
    - merge_requests
    - main
    - develop

# 构建镜像
build_image:
  stage: build
  image: docker:24
  services:
    - docker:24-dind
  before_script:
    - echo "$CI_REGISTRY_PASSWORD" | docker login -u "$CI_REGISTRY_USER" --password-stdin $CI_REGISTRY
  script:
    - docker build -t $IMAGE_NAME:$CI_COMMIT_SHA .
    - docker push $IMAGE_NAME:$CI_COMMIT_SHA
    - docker tag $IMAGE_NAME:$CI_COMMIT_SHA $IMAGE_NAME:latest
    - docker push $IMAGE_NAME:latest
  only:
    - main
    - develop

# 容器镜像扫描
container_scanning:
  stage: security
  image: aquasec/trivy:latest
  script:
    - trivy image --exit-code 0 --no-progress --format json $IMAGE_NAME:$CI_COMMIT_SHA > scan-report.json
    - trivy image --exit-code 1 --severity HIGH,CRITICAL $IMAGE_NAME:$CI_COMMIT_SHA
  artifacts:
    reports:
      container_scanning: scan-report.json
    paths:
      - scan-report.json
    expire_in: 1 month
  allow_failure: true
  only:
    - main
    - develop

# SAST 扫描
sast:
  stage: security
  include:
    - template: Security/SAST.gitlab-ci.yml
  only:
    - merge_requests
    - main
    - develop

# 依赖扫描
dependency_scanning:
  stage: security
  include:
    - template: Security/Dependency-Scanning.gitlab-ci.yml
  only:
    - merge_requests
    - main
    - develop

# 部署到测试环境
deploy_staging:
  stage: deploy
  image: alpine/helm:3.12
  environment:
    name: staging
    url: https://staging.example.com
  before_script:
    - apk add --no-cache curl
    - helm version
  script:
    - helm upgrade --install myapp ./helm-chart
        --namespace staging
        --create-namespace
        --set image.tag=$CI_COMMIT_SHA
        --set env=staging
        --set replicaCount=2
        --wait
        --timeout 5m
    - ./scripts/health-check.sh https://staging.example.com
  only:
    - develop

# 部署到生产环境
deploy_production:
  stage: deploy
  image: alpine/helm:3.12
  environment:
    name: production
    url: https://example.com
  when: manual
  before_script:
    - apk add --no-cache curl
  script:
    - helm upgrade --install myapp ./helm-chart
        --namespace production
        --set image.tag=$CI_COMMIT_SHA
        --set env=production
        --set replicaCount=5
        --set resources.requests.memory="512Mi"
        --set resources.requests.cpu="500m"
        --set resources.limits.memory="1Gi"
        --set resources.limits.cpu="1000m"
        --wait
        --timeout 10m
    - ./scripts/health-check.sh https://example.com
    - ./scripts/smoke-test.sh
  only:
    - main

# 通知
notify_success:
  stage: notify
  image: alpine:3.18
  script:
    - apk add --no-cache curl
    - |
      curl -X POST "$SLACK_WEBHOOK" \
        -H 'Content-Type: application/json' \
        -d '{
          "text": "部署成功: '$CI_PROJECT_NAME' '$CI_COMMIT_REF_NAME'",
          "blocks": [
            {
              "type": "section",
              "text": {
                "type": "mrkdwn",
                "text": "*部署成功*\n\n项目: '$CI_PROJECT_NAME'\n分支: '$CI_COMMIT_REF_NAME'\n提交: '$CI_COMMIT_SHORT_SHA'\n作者: '$GITLAB_USER_NAME'\n\n查看: <'$CI_PIPELINE_URL'|流水线>"
              }
            }
          ]
        }'
  when: on_success
  only:
    - main
    - develop

notify_failure:
  stage: notify
  image: alpine:3.18
  script:
    - apk add --no-cache curl
    - |
      curl -X POST "$SLACK_WEBHOOK" \
        -H 'Content-Type: application/json' \
        -d '{
          "text": "部署失败: '$CI_PROJECT_NAME' '$CI_COMMIT_REF_NAME'",
          "blocks": [
            {
              "type": "section",
              "text": {
                "type": "mrkdwn",
                "text": "*部署失败*\n\n项目: '$CI_PROJECT_NAME'\n分支: '$CI_COMMIT_REF_NAME'\n提交: '$CI_COMMIT_SHORT_SHA'\n作者: '$GITLAB_USER_NAME'\n\n查看: <'$CI_PIPELINE_URL'|流水线>"
              }
            }
          ]
        }'
  when: on_failure
  only:
    - main
    - develop
```

### GitHub Actions

```yaml
# .github/workflows/ci-cd.yml
name: devops-engineer

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]
  release:
    types: [ created ]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  # 代码质量检查
  lint:
    name: 代码质量检查
    runs-on: ubuntu-latest
    steps:
      - name: 检出代码
        uses: actions/checkout@v4

      - name: 设置 Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '18'
          cache: 'npm'

      - name: 安装依赖
        run: npm ci

      - name: 运行 ESLint
        run: npm run lint

      - name: 检查代码格式
        run: npm run format:check

  # 单元测试
  test:
    name: 单元测试
    runs-on: ubuntu-latest
    steps:
      - name: 检出代码
        uses: actions/checkout@v4

      - name: 设置 Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '18'
          cache: 'npm'

      - name: 安装依赖
        run: npm ci

      - name: 运行测试
        run: npm run test:coverage

      - name: 上传覆盖率报告
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/lcov.info
          flags: unittests
          name: codecov-umbrella

  # 安全扫描
  security:
    name: 安全扫描
    runs-on: ubuntu-latest
    steps:
      - name: 检出代码
        uses: actions/checkout@v4

      - name: 运行 Trivy 漏洞扫描
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: '.'
          format: 'sarif'
          output: 'trivy-results.sarif'

      - name: 上传扫描结果到 GitHub Security
        uses: github/codeql-action/upload-sarif@v2
        if: always()
        with:
          sarif_file: 'trivy-results.sarif'

  # 构建镜像
  build:
    name: 构建 Docker 镜像
    runs-on: ubuntu-latest
    needs: [lint, test]
    if: github.event_name != 'pull_request'
    permissions:
      contents: read
      packages: write
    steps:
      - name: 检出代码
        uses: actions/checkout@v4

      - name: 设置 Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: 登录到容器镜像仓库
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: 提取元数据
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=sha,prefix={{branch}}-
            type=raw,value=latest,enable={{is_default_branch}}

      - name: 构建并推送镜像
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  # 部署到测试环境
  deploy_staging:
    name: 部署到测试环境
    runs-on: ubuntu-latest
    needs: build
    if: github.ref == 'refs/heads/develop'
    environment:
      name: staging
      url: https://staging.example.com
    steps:
      - name: 检出代码
        uses: actions/checkout@v4

      - name: 配置 AWS 凭证
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: us-east-1

      - name: 部署到 EKS
        run: |
          aws eks update-kubeconfig --name staging-cluster --region us-east-1
          kubectl set image deployment/myapp myapp=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }} -n staging
          kubectl rollout status deployment/myapp -n staging

      - name: 健康检查
        run: |
          ./scripts/health-check.sh https://staging.example.com

  # 部署到生产环境
  deploy_production:
    name: 部署到生产环境
    runs-on: ubuntu-latest
    needs: build
    if: github.ref == 'refs/heads/main'
    environment:
      name: production
      url: https://example.com
    steps:
      - name: 检出代码
        uses: actions/checkout@v4

      - name: 配置 AWS 凭证
        uses: aws-actions/configure-aws-credentials@v4
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: us-east-1

      - name: 部署到 EKS
        run: |
          aws eks update-kubeconfig --name production-cluster --region us-east-1

          # 金丝雀发布
          kubectl apply -f k8s/production/canary.yaml

          # 等待验证
          sleep 300

          # 全量发布
          kubectl apply -f k8s/production/deployment.yaml
          kubectl delete -f k8s/production/canary.yaml

      - name: 健康检查
        run: |
          ./scripts/health-check.sh https://example.com
          ./scripts/smoke-test.sh

  # 通知
  notify:
    name: 发送通知
    runs-on: ubuntu-latest
    needs: [deploy_staging, deploy_production]
    if: always()
    steps:
      - name: 发送 Slack 通知
        uses: slackapi/slack-github-action@v1.24.0
        with:
          payload: |
            {
              "text": "部署状态: ${{ job.status }}",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*部署 ${{ job.status }}*\n\n仓库: ${{ github.repository }}\n分支: ${{ github.ref_name }}\n提交: ${{ github.sha }}\n作者: ${{ github.actor }}\n\n查看: <${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}|流水线>"
                  }
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
```

## 基础设施即代码

### Terraform 多环境配置

```hcl
# terraform/environments/production/main.tf
terraform {
  required_version = ">= 1.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }

    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }

    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.11"
    }
  }

  backend "s3" {
    bucket         = "terraform-state-prod"
    key            = "production/terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "terraform-locks-prod"
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = {
      Environment = "production"
      ManagedBy   = "Terraform"
      Project     = var.project_name
    }
  }
}

# VPC
module "vpc" {
  source = "../../../modules/vpc"

  name               = "${var.project_name}-vpc"
  cidr               = var.vpc_cidr
  availability_zones = var.availability_zones

  enable_nat_gateway   = true
  enable_vpn_gateway   = false
  enable_dns_hostnames = true
  enable_dns_support   = true

  public_subnet_tags  = { Type = "public" }
  private_subnet_tags = { Type = "private" }
}

# EKS 集群
module "eks" {
  source = "terraform-aws-modules/eks/aws"

  cluster_name    = "${var.project_name}-cluster"
  cluster_version = "1.28"

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  eks_managed_node_groups = {
    general = {
      desired_size = 3
      min_size     = 2
      max_size     = 10

      instance_types = ["t3.large"]
      capacity_type  = "ON_DEMAND"

      labels = {
        Environment = "production"
        NodeGroup   = "general"
      }

      taints = []
    }

    compute_intensive = {
      desired_size = 2
      min_size     = 1
      max_size     = 5

      instance_types = ["c5.xlarge"]
      capacity_type  = "SPOT"

      labels = {
        Environment = "production"
        NodeGroup   = "compute"
      }

      taints = [{
        key    = "workload"
        value  = "compute"
        effect = "NO_SCHEDULE"
      }]
    }
  }

  cluster_addons = {
    coredns = {
      most_recent = true
    }
    kube-proxy = {
      most_recent = true
    }
    vpc-cni = {
      most_recent = true
    }
    aws-ebs-csi-driver = {
      most_recent = true
    }
  }

  cluster_endpoint_public_access  = true
  cluster_endpoint_private_access = true

  create_cluster_security_group = false
  create_node_security_group    = false

  manage_aws_auth_configmap = true

  tags = {
    Environment = "production"
  }
}

# RDS 数据库
module "database" {
  source = "../../../modules/rds"

  identifier     = "${var.project_name}-db"
  engine         = "postgres"
  engine_version = "15.4"
  instance_class = "db.r6g.xlarge"

  allocated_storage     = 100
  max_allocated_storage = 1000
  storage_encrypted     = true
  storage_type          = "io1"
  iops                  = 3000

  db_name  = "production"
  username = var.db_username
  password = var.db_password

  vpc_id            = module.vpc.vpc_id
  subnet_ids        = module.vpc.private_subnets
  security_group_ids = [module.eks.cluster_security_group_id]

  multi_az               = true
  backup_retention_period = 30
  backup_window          = "03:00-04:00"

  maintenance_window = "Mon:04:00-Mon:05:00"

  performance_insights_enabled = true
  monitoring_interval         = 60
  monitoring_role_arn         = aws_iam_role.rds_monitoring.arn

  tags = {
    Environment = "production"
  }
}

# ElastiCache Redis
module "redis" {
  source = "../../../modules/elasticache"

  cluster_id           = "${var.project_name}-redis"
  engine               = "redis"
  engine_version       = "7.0"
  node_type            = "cache.r6g.large"
  num_cache_nodes      = 3
  parameter_group_name = "default.redis7"
  port                 = 6379

  vpc_id            = module.vpc.vpc_id
  subnet_ids        = module.vpc.private_subnets
  security_group_ids = [module.eks.cluster_security_group_id]

  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
  auth_token                = var.redis_auth_token

  automatic_failover_enabled = true
  multi_az_enabled          = true

  snapshot_retention_limit = 7
  snapshot_window         = "03:00-05:00"

  tags = {
    Environment = "production"
  }
}

# ALB Ingress Controller
module "alb" {
  source = "../../../modules/alb"

  name       = "${var.project_name}-alb"
  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.public_subnets

  certificate_arn = var.acm_certificate_arn

  security_group_ids = [module.eks.cluster_security_group_id]

  tags = {
    Environment = "production"
  }
}

# S3 存储桶
module "s3" {
  source = "../../../modules/s3"

  bucket_prefix = "${var.project_name}-"

  buckets = {
    logs = {
      versioning = true
      lifecycle_rule = [
        {
          enabled      = true
          transition   = { days = 30, storage_class = "STANDARD_IA" }
          expiration   = { days = 90 }
        }
      ]
    }

    backups = {
      versioning = true
      lifecycle_rule = [
        {
          enabled    = true
          expiration = { days = 90 }
        }
      ]
    }

    assets = {
      versioning = false
      cdn_enabled = true
    }
  }

  tags = {
    Environment = "production"
  }
}
```

### Kubernetes 部署清单

```yaml
# k8s/production/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: myapp
  namespace: production
  labels:
    app: myapp
    environment: production
spec:
  replicas: 5
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: myapp
  template:
    metadata:
      labels:
        app: myapp
        environment: production
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      serviceAccountName: myapp

      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        fsGroup: 1000

      containers:
      - name: myapp
        image: registry.example.com/myapp:latest
        imagePullPolicy: Always

        ports:
        - name: http
          containerPort: 8080
          protocol: TCP

        env:
        - name: ENVIRONMENT
          value: "production"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: myapp-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: myapp-secrets
              key: redis-url

        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"

        livenessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3

        readinessProbe:
          httpGet:
            path: /ready
            port: http
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3

        volumeMounts:
        - name: config
          mountPath: /etc/config
          readOnly: true

      volumes:
      - name: config
        configMap:
          name: myapp-config

      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - myapp
              topologyKey: kubernetes.io/hostname

      nodeSelector:
        workload: general

      tolerations:
      - key: "workload"
        operator: "Equal"
        value: "compute"
        effect: "NoSchedule"

---
apiVersion: v1
kind: Service
metadata:
  name: myapp-service
  namespace: production
  labels:
    app: myapp
spec:
  type: ClusterIP
  selector:
    app: myapp
  ports:
  - name: http
    port: 80
    targetPort: http
    protocol: TCP

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: myapp-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: myapp
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60

---
# 网络策略
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: myapp-network-policy
  namespace: production
spec:
  podSelector:
    matchLabels:
      app: myapp
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
  - from:
    - podSelector:
        matchLabels:
          app: monitoring
    ports:
    - protocol: TCP
      port: 9090
  egress:
  - to:
    - namespaceSelector:
        matchLabels:
          name: kube-system
    ports:
    - protocol: TCP
      port: 53
  - to:
    - podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
```

## 监控与告警

### Prometheus 配置

```yaml
# prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'production'
    env: 'production'

# 告警规则文件
rule_files:
  - '/etc/prometheus/rules/*.yml'

# 告警管理
alerting:
  alertmanagers:
  - static_configs:
    - targets:
      - alertmanager:9093

# 抓取配置
scrape_configs:
  # Kubernetes API Server
  - job_name: 'kubernetes-apiservers'
    kubernetes_sd_configs:
    - role: endpoints
    scheme: https
    tls_config:
      ca_file: /var/run/secrets/kubernetes.io/serviceaccount/ca.crt
    bearer_token_file: /var/run/secrets/kubernetes.io/serviceaccount/token
    relabel_configs:
    - source_labels: [__meta_kubernetes_namespace, __meta_kubernetes_service_name, __meta_kubernetes_endpoint_port_name]
      action: keep
      regex: default;kubernetes;https

  # Node Exporter
  - job_name: 'kubernetes-nodes'
    kubernetes_sd_configs:
    - role: node
    relabel_configs:
    - action: labelmap
      regex: __meta_kubernetes_node_label_(.+)

  # Kubelet
  - job_name: 'kubernetes-kubelet'
    kubernetes_sd_configs:
    - role: node
    scheme: https
    tls_config:
      ca_file: /var/run/secrets/kubernetes.io/serviceaccount/ca.crt
      insecure_skip_verify: true
    relabel_configs:
    - action: labelmap
      regex: __meta_kubernetes_node_label_(.+)

  # cAdvisor
  - job_name: 'kubernetes-cadvisor'
    kubernetes_sd_configs:
    - role: node
    relabel_configs:
    - action: labelmap
      regex: __meta_kubernetes_node_label_(.+)
    - source_labels: [__meta_kubernetes_node_name]
      regex: (.+)
      target_label: __address__
      replacement: ${1}:4194

  # 应用指标
  - job_name: 'myapp'
    kubernetes_sd_configs:
    - role: pod
    relabel_configs:
    - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
      action: keep
      regex: true
    - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
      action: replace
      target_label: __metrics_path__
      regex: (.+)
    - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
      action: replace
      regex: ([^:]+)(?::\d+)?;(\d+)
      replacement: $1:$2
      target_label: __address__
    - action: labelmap
      regex: __meta_kubernetes_pod_label_(.+)
    - source_labels: [__meta_kubernetes_namespace]
      action: replace
      target_label: kubernetes_namespace
    - source_labels: [__meta_kubernetes_pod_name]
      action: replace
      target_label: kubernetes_pod_name
```

### 告警规则

```yaml
# prometheus/rules/alerts.yml
groups:
- name: application_alerts
  interval: 30s
  rules:
  # 应用可用性
  - alert: ApplicationDown
    expr: up{job="myapp"} == 0
    for: 1m
    labels:
      severity: critical
      team: platform
    annotations:
      summary: "应用不可用 (instance {{ $labels.instance }})"
      description: "应用 {{ $labels.job }} 在 {{ $labels.instance }} 已经宕机超过 1 分钟"

  # 高错误率
  - alert: HighErrorRate
    expr: |
      (
        sum(rate(http_requests_total{status=~"5..", job="myapp"}[5m]))
        /
        sum(rate(http_requests_total{job="myapp"}[5m]))
      ) > 0.05
    for: 5m
    labels:
      severity: warning
      team: platform
    annotations:
      summary: "高错误率 ({{ $labels.instance }})"
      description: "错误率超过 5% (当前值: {{ $value | humanizePercentage }})"

  # 高延迟
  - alert: HighLatency
    expr: |
      histogram_quantile(0.99,
        sum(rate(http_request_duration_seconds_bucket{job="myapp"}[5m])) by (le)
      ) > 1
    for: 10m
    labels:
      severity: warning
      team: platform
    annotations:
      summary: "高延迟 ({{ $labels.instance }})"
      description: "P99 延迟超过 1s (当前值: {{ $value }}s)"

  # Pod 崩溃
  - alert: PodCrashLooping
    expr: |
      rate(kube_pod_container_status_restarts_total{namespace="production"}[15m]) > 0
    for: 5m
    labels:
      severity: warning
      team: platform
    annotations:
      summary: "Pod 崩溃循环 ({{ $labels.namespace }}/{{ $labels.pod }})"
      description: "Pod {{ $labels.pod }} 在命名空间 {{ $labels.namespace }} 处于崩溃循环状态"

- name: kubernetes_alerts
  interval: 30s
  rules:
  # 节点宕机
  - alert: NodeDown
    expr: |
      kube_node_status_condition{condition="Ready", status!="true"} == 1
    for: 10m
    labels:
      severity: critical
      team: sre
    annotations:
      summary: "节点宕机 ({{ $labels.node }})"
      description: "节点 {{ $labels.node }} 已经不可用超过 10 分钟"

  # Pod 处于 Pending 状态
  - alert: PodStuckPending
    expr: |
      kube_pod_status_phase{namespace="production", phase="Pending"} == 1
    for: 10m
    labels:
      severity: warning
      team: sre
    annotations:
      summary: "Pod 卡在 Pending 状态"
      description: "Pod {{ $labels.pod }} 在命名空间 {{ $labels.namespace }} 已处于 Pending 状态超过 10 分钟"

  # 磁盘空间不足
  - alert: DiskSpaceLow
    expr: |
      (
        (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"})
        < 0.1
      )
    for: 5m
    labels:
      severity: warning
      team: sre
    annotations:
      summary: "磁盘空间不足 ({{ $labels.instance }})"
      description: "节点 {{ $labels.instance }} 磁盘使用率超过 90%"

  # 内存使用率过高
  - alert: HighMemoryUsage
    expr: |
      (
        sum(container_memory_working_set_bytes{namespace="production"})
        /
        sum(kube_node_status_capacity{resource="memory"})
      ) > 0.9
    for: 5m
    labels:
      severity: warning
      team: sre
    annotations:
      summary: "内存使用率过高"
      description: "集群内存使用率超过 90% (当前值: {{ $value | humanizePercentage }})"

- name: business_alerts
  interval: 1m
  rules:
  # 订单量下降
  - alert: OrderVolumeDrop
    expr: |
      (
        sum(rate(orders_total[5m]))
        <
        avg_over_time(sum(rate(orders_total[5m]))[1h:5m]) * 0.5
      )
    for: 10m
    labels:
      severity: warning
      team: business
    annotations:
      summary: "订单量下降"
      description: "订单量在过去 10 分钟内下降了超过 50%"

  # 支付失败率上升
  - alert: HighPaymentFailureRate
    expr: |
      (
        sum(rate(payment_failures_total[5m]))
        /
        sum(rate(payment_attempts_total[5m]))
      ) > 0.05
    for: 5m
    labels:
      severity: critical
      team: business
    annotations:
      summary: "支付失败率上升"
      description: "支付失败率超过 5% (当前值: {{ $value | humanizePercentage }})"
```

## 最佳实践

### CI/CD 流水线
```
✅ DO:
  - 快速反馈（并行化测试）
  - 缓存依赖
  - 使用制品仓库
  - 实施代码质量门禁
  - 自动化安全扫描
  - 金丝雀发布
  - 自动回滚机制

❌ DON'T:
  - 忽略测试覆盖率
  - 手动部署
  - 硬编码配置
  - 跳过安全扫描
  - 缺少监控告警
```

### 基础设施管理
```
✅ DO:
  - 版本控制所有配置
  - 使用模块化设计
  - 实施状态管理
  - 环境一致性
  - 定期备份
  - 灾难恢复演练

❌ DON'T:
  - 手动修改生产环境
  - 忽略文档
  - 共享管理员凭证
  - 跳过变更审批
```

### 监控告警
```
✅ DO:
  - 监控所有层级
  - 设置合理的阈值
  - 实施告警分级
  - 建立值班轮换
  - 编写运维手册
  - 定期演练故障处理

❌ DON'T:
  - 告警风暴
  - 忽略告警
  - 缺少文档
  - 无事故响应流程
```

## 工具和资源

### CI/CD 工具
- **Jenkins**: 开源 CI/CD 服务器
- **GitLab CI/CD**: 集成的 CI/CD
- **GitHub Actions**: GitHub 的 CI/CD
- **CircleCI**: 云端 CI/CD 平台
- **ArgoCD**: GitOps 持续部署
- **Flux**: GitOps 操作符

### IaC 工具
- **Terraform**: 多云 IaC
- **Pulumi**: 使用编程语言的 IaC
- **AWS CDK**: AWS 云开发工具包
- **Ansible**: 配置管理
- **Helm**: Kubernetes 包管理

### 监控工具
- **Prometheus**: 监控系统
- **Grafana**: 可视化平台
- **Jaeger**: 分布式追踪
- **ELK Stack**: 日志管理
- **Datadog**: APM 平台

### 文档资源
- [Jenkins 文档](https://www.jenkins.io/doc/)
- [Kubernetes 文档](https://kubernetes.io/docs/)
- [Terraform 文档](https://www.terraform.io/docs)
- [Prometheus 文档](https://prometheus.io/docs/)
