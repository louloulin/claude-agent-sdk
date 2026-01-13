---
name: cloud-infrastructure
description: "Expert in cloud infrastructure design, deployment, and management across AWS, Azure, and GCP"
version: "1.5.0"
author: "DevOps Team <devops@example.com>"
tags:
  - cloud
  - aws
  - azure
  - gcp
  - infrastructure
  - devops
  - terraform
dependencies:
  - docker-helper
  - security-auditor
---

# Cloud Infrastructure Expert

You are a cloud infrastructure expert specializing in AWS, Azure, and GCP. Help design, deploy, and manage cloud infrastructure.

## Cloud Platform Comparison

### AWS (Amazon Web Services)
```
Strengths:
  ✅ Most mature platform (200+ services)
  ✅ Largest ecosystem and community
  ✅ Best for enterprise workloads
  ✅ Global infrastructure

Best For:
  - Enterprise applications
  - Large-scale deployments
  - Hybrid cloud scenarios
  - Complex architectures

Key Services:
  - EC2, Lambda, ECS/EKS
  - S3, RDS, DynamoDB
  - CloudFront, Route53
  - CloudFormation, Terraform
```

### Azure (Microsoft)
```
Strengths:
  ✅ Excellent Windows integration
  ✅ Strong enterprise features
  ✅ Hybrid cloud leadership
  ✅ Good developer tools

Best For:
  - Microsoft shops
  - Enterprise Windows workloads
  - Hybrid deployments
  - Government/healthcare

Key Services:
  - VMs, Functions, AKS/Container Instances
  - Blob Storage, SQL Database, Cosmos DB
  - CDN, Traffic Manager
  - ARM Templates, Bicep
```

### GCP (Google Cloud Platform)
```
Strengths:
  ✅ Best Kubernetes (GKE)
  ✅ Strong data/analytics
  ✅ Good AI/ML services
  ✅ Competitive pricing

Best For:
  - Kubernetes workloads
  - Data analytics
  - AI/ML projects
  - Startups

Key Services:
  - Compute Engine, Cloud Run, GKE
  - Cloud Storage, BigQuery, Cloud Spanner
  - Cloud CDN, Cloud Load Balancing
  - Deployment Manager, Terraform
```

## Infrastructure as Code (IaC)

### Terraform (Multi-Cloud)

```hcl
# Main Terraform configuration
terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# VPC Configuration
resource "aws_vpc" "main" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name        = "${var.project_name}-vpc"
    Environment = var.environment
  }
}

# Public Subnets
resource "aws_subnet" "public" {
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = "10.0.${count.index}.0/24"
  availability_zone = var.availability_zones[count.index]

  map_public_ip_on_launch = true

  tags = {
    Name = "${var.project_name}-public-${count.index}"
  }
}

# Application Load Balancer
resource "aws_lb" "app" {
  name               = "${var.project_name}-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets            = aws_subnet.public[*].id

  enable_deletion_protection = false

  tags = {
    Environment = var.environment
  }
}

# ECS Cluster
resource "aws_ecs_cluster" "main" {
  name = "${var.project_name}-cluster"

  setting {
    name  = "containerInsights"
    value = "enabled"
  }
}

# Auto Scaling
resource "aws_appautoscaling_target" "ecs" {
  max_capacity       = 10
  min_capacity       = 2
  resource_id        = "service/${aws_ecs_cluster.main.name}/${aws_ecs_service.app.name}"
  scalable_dimension = "ecs:service:DesiredCount"
  service_namespace  = "ecs"
}

resource "aws_appautoscaling_policy" "ecs" {
  name               = "app-autoscaling"
  policy_type        = "TargetTrackingScaling"
  resource_id        = aws_appautoscaling_target.ecs.resource_id
  scalable_dimension = aws_appautoscaling_target.ecs.scalable_dimension
  service_namespace  = aws_appautoscaling_target.ecs.service_namespace

  target_tracking_scaling_policy_configuration {
    predefined_metric_specification {
      predefined_metric_type = "ECSServiceAverageCPUUtilization"
    }
    target_value       = 70.0
    scale_in_cooldown  = 300
    scale_out_cooldown = 60
  }
}
```

### AWS CloudFormation

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Description: 'Production web application infrastructure'

Parameters:
  Environment:
    Type: String
    AllowedValues:
      - dev
      - staging
      - production
    Default: dev

  InstanceType:
    Type: String
    Default: t3.micro

Resources:
  # VPC
  VPC:
    Type: AWS::EC2::VPC
    Properties:
      CidrBlock: 10.0.0.0/16
      EnableDnsHostnames: true
      EnableDnsSupport: true
      Tags:
        - Key: Name
          Value: !Sub '${AWS::StackName}-vpc'

  # Public Subnet
  PublicSubnet:
    Type: AWS::EC2::Subnet
    Properties:
      VpcId: !Ref VPC
      CidrBlock: 10.0.1.0/24
      AvailabilityZone: !Select [ 0, !GetAZs '' ]
      MapPublicIpOnLaunch: true

  # Security Group
  WebSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Enable HTTP/HTTPS access
      VpcId: !Ref VPC
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: 80
          ToPort: 80
          CidrIp: 0.0.0.0/0
        - IpProtocol: tcp
          FromPort: 443
          ToPort: 443
          CidrIp: 0.0.0.0/0

Outputs:
  WebsiteURL:
    Description: Website URL
    Value: !Sub 'http://${LoadBalancer.DNSName}'
```

## Container Orchestration

### Kubernetes (AWS EKS)

```yaml
# Deployment manifest
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-application
  namespace: production
spec:
  replicas: 3
  selector:
    matchLabels:
      app: web-application
  template:
    metadata:
      labels:
        app: web-application
    spec:
      containers:
      - name: app
        image: ${ECR_REGISTRY}/web-app:${IMAGE_TAG}
        ports:
        - containerPort: 8080
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url

---
# Service manifest
apiVersion: v1
kind: Service
metadata:
  name: web-application-service
  namespace: production
spec:
  type: LoadBalancer
  selector:
    app: web-application
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080

---
# Horizontal Pod Autoscaler
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: web-application-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: web-application
  minReplicas: 2
  maxReplicas: 10
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
```

### Docker (AWS ECS)

```json
{
  "family": "web-app",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "256",
  "memory": "512",
  "executionRoleArn": "arn:aws:iam::ACCOUNT_ID:role/ecsTaskExecutionRole",
  "containerDefinitions": [
    {
      "name": "web-app",
      "image": "ACCOUNT_ID.dkr.ecr.REGION.amazonaws.com/web-app:latest",
      "essential": true,
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "ENVIRONMENT",
          "value": "production"
        }
      ],
      "secrets": [
        {
          "name": "DATABASE_URL",
          "valueFrom": "arn:aws:secretsmanager:REGION:ACCOUNT_ID:secret:db-url"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/web-app",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      },
      "healthCheck": {
        "command": [
          "CMD-SHELL",
          "curl -f http://localhost:8080/health || exit 1"
        ],
        "interval": 30,
        "timeout": 5,
        "retries": 3
      }
    }
  ]
}
```

## Serverless Architectures

### AWS Lambda

```python
import json
import boto3
from datetime import datetime

s3 = boto3.client('s3')
dynamodb = boto3.resource('dynamodb')

def lambda_handler(event, context):
    """Process S3 upload event"""

    # Get bucket and object key from event
    for record in event['Records']:
        bucket = record['s3']['bucket']['name']
        key = record['s3']['object']['key']

        # Process the file
        try:
            # Get object from S3
            response = s3.get_object(Bucket=bucket, Key=key)
            content = response['Body'].read()

            # Process content
            result = process_content(content)

            # Save to DynamoDB
            table = dynamodb.Table('processed-files')
            table.put_item(Item={
                'fileKey': key,
                'processedAt': datetime.utcnow().isoformat(),
                'result': result
            })

            return {
                'statusCode': 200,
                'body': json.dumps({
                    'message': f'Processed {key}',
                    'result': result
                })
            }

        except Exception as e:
            print(f"Error processing {key}: {str(e)}")
            raise
```

### API Gateway + Lambda

```yaml
# Serverless application using SAM
AWSTemplateFormatVersion: '2010-09-09'
Transform: AWS::Serverless-2016-10-31

Resources:
  ProcessFileFunction:
    Type: AWS::Serverless::Function
    Properties:
      CodeUri: ./src
      Handler: app.lambda_handler
      Runtime: python3.9
      Timeout: 30
      MemorySize: 256
      Environment:
        Variables:
          TABLE_NAME: !Ref ProcessedFilesTable
      Policies:
        - DynamoDBCrudPolicy:
            TableName: !Ref ProcessedFilesTable
      Events:
        FileUpload:
          Type: S3
          Properties:
            Bucket: !Ref FileBucket
            Events: s3:ObjectCreated:*

  ProcessedFilesTable:
    Type: AWS::Serverless::SimpleTable

  FileBucket:
    Type: AWS::S3::Bucket
```

## Database Management

### Amazon RDS

```python
import boto3

rds = boto3.client('rds')

# Create RDS instance
response = rds.create_db_instance(
    DBInstanceIdentifier='production-db',
    DBInstanceClass='db.t3.micro',
    Engine='postgres',
    EngineVersion='14.7',
    MasterUsername='admin',
    MasterUserPassword='SecurePassword123!',
    AllocatedStorage=20,
    StorageType='gp2',
    StorageEncrypted=True,
    VpcSecurityGroupIds=['sg-12345'],
    DBSubnetGroupName='my-subnet-group',
    BackupRetentionPeriod=7,
    MultiAZ=False,
    PubliclyAccessible=False,
    Tags=[
        {'Key': 'Environment', 'Value': 'production'},
        {'Key': 'Application', 'Value': 'web-app'}
    ]
)
```

### Azure Cosmos DB

```python
from azure.cosmos import CosmosClient, PartitionKey, exceptions

url = "https://your-account.documents.azure.com:443/"
key = "your-master-key"
client = CosmosClient(url, credential=key)

# Create database
database = client.create_database_if_not_exists(id='app-database')

# Create container
container = database.create_container_if_not_exists(
    id='users',
    partition_key=PartitionKey(path="/userId"),
    offer_throughput=400
)

# Insert item
user_item = {
    'id': 'user-001',
    'userId': 'user-001',
    'name': 'John Doe',
    'email': 'john@example.com'
}

container.create_item(body=user_item)
```

## Monitoring & Observability

### AWS CloudWatch

```python
import boto3
from datetime import datetime, timedelta

cloudwatch = boto3.client('cloudwatch')

# Create custom metric
cloudwatch.put_metric_data(
    Namespace='WebApplication',
    MetricData=[
        {
            'MetricName': 'RequestCount',
            'Value': 100,
            'Unit': 'Count',
            'Timestamp': datetime.utcnow(),
            'Dimensions': [
                {'Name': 'Environment', 'Value': 'production'}
            ]
        }
    ]
)

# Create alarm
cloudwatch.put_metric_alarm(
    AlarmName='HighErrorRate',
    AlarmDescription='Alert when error rate exceeds threshold',
    Namespace='WebApplication',
    MetricName='ErrorRate',
    Statistic='Average',
    Period=300,
    EvaluationPeriods=2,
    Threshold=5.0,
    ComparisonOperator='GreaterThanThreshold',
    TreatMissingData='notBreaching'
)
```

### Azure Monitor

```python
from azure.monitor.query import MetricsQueryClient
from azure.identity import DefaultAzureCredential

credential = DefaultAzureCredential()
client = MetricsQueryClient(credential)

# Query metrics
metrics_uri = "https://your-resource-id"
response = client.query_resource(
    metrics_uri,
    metric_names=["RequestCount", "ResponseTime"],
    timespan=timedelta(hours=1)
)

for metric in response.metrics:
    print(f"Metric: {metric.name}")
    for timeseries in metric.timeseries:
        for data_point in timeseries.data:
            print(f"  {data_point.timestamp}: {data_point.average}")
```

## Security Best Practices

### IAM Policies

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "LeastPrivilegeAccess",
      "Effect": "Allow",
      "Action": [
        "s3:GetObject",
        "s3:PutObject"
      ],
      "Resource": "arn:aws:s3:::my-bucket/*"
    },
    {
      "Sid": "DenyUnencryptedAccess",
      "Effect": "Deny",
      "Action": [
        "s3:*"
      ],
      "Resource": "arn:aws:s3:::*",
      "Condition": {
        "Bool": {
          "aws:SecureTransport": "false"
        }
      }
    }
  ]
}
```

### Secrets Management

```python
import boto3
import json

secrets_client = boto3.client('secretsmanager')

# Store secret
secrets_client.create_secret(
    Name='prod/db-credentials',
    SecretString=json.dumps({
        'username': 'admin',
        'password': 'SecurePassword123!',
        'host': 'db.example.com',
        'port': 5432
    })
)

# Retrieve secret
response = secrets_client.get_secret_value(SecretId='prod/db-credentials')
credentials = json.loads(response['SecretString'])
```

## Cost Optimization

### Right-Sizing Instances

```python
import boto3

ce = boto3.client('ce')

# Get cost and usage data
response = ce.get_cost_and_usage(
    TimePeriod={
        'Start': '2024-01-01',
        'End': '2024-01-31'
    },
    Granularity='DAILY',
    Metrics=['BlendedCost', 'UsageQuantity'],
    GroupBy=[
        {'Type': 'DIMENSION', 'Key': 'INSTANCE_TYPE'},
        {'Type': 'DIMENSION', 'Key': 'SERVICE'}
    ]
)

# Analyze and recommend right-sizing
for result in response['ResultsByTime']:
    print(f"{result['TimePeriod']['Start']}: {result['Total']['BlendedCost']}")
```

### Auto Scaling Strategy

```python
# Schedule-based scaling
aws autoscaling put-scheduled-update-group-action \
  --auto-scaling-group-name my-asg \
  --scheduled-action-name scale-up-business-hours \
  --recurrence "0 9 * * Mon-Fri" \
  --min-size 5 \
  --max-size 20 \
  --desired-capacity 10

aws autoscaling put-scheduled-update-group-action \
  --auto-scaling-group-name my-asg \
  --scheduled-action-name scale-down-after-hours \
  --recurrence "0 18 * * Mon-Fri" \
  --min-size 2 \
  --max-size 5 \
  --desired-capacity 2
```

## Disaster Recovery

### Multi-Region Deployment

```hcl
# Terraform multi-region configuration
module "primary_region" {
  source = "./modules/infrastructure"

  aws_region  = "us-east-1"
  environment = "production"

  providers = {
    aws = aws.primary
  }
}

module "secondary_region" {
  source = "./modules/infrastructure"

  aws_region  = "us-west-2"
  environment = "production-dr"

  providers = {
    aws = aws.secondary
  }
}

# Route53 health checks and failover
resource "aws_route53_health_check" "primary" {
  provider = aws.primary

  fqdn              = module.primary_region.load_balancer_dns
  port              = 443
  type              = "HTTPS"
  resource_path     = "/health"
  request_interval  = 30
  failure_threshold = 3
}

resource "aws_route53_record" "www" {
  zone_id = aws_route53_zone.main.zone_id
  name    = "www.example.com"
  type    = "A"

  failover_routing_policy {
    type           = "PRIMARY"
    failover_record_set_id = aws_route53_record.secondary_dns.id
  }

  alias {
    name                   = module.primary_region.load_balancer_dns
    zone_id                = module.primary_region.load_balancer_zone_id
    evaluate_target_health = true
  }
}
```

## Tools & Resources

### Essential Tools
- **Terraform**: Infrastructure as Code
- **Pulumi**: Modern IaC with real programming languages
- **AWS CDK**: Cloud Development Kit for AWS
- **Packer**: Machine image builder
- **Ansible**: Configuration management
- **Kubernetes**: Container orchestration
- **Helm**: Kubernetes package manager
- **Prometheus**: Monitoring
- **Grafana**: Visualization

### Documentation
- [AWS Well-Architected Framework](https://aws.amazon.com/architecture/well-architected/)
- [Azure Architecture Center](https://docs.microsoft.com/en-us/azure/architecture/)
- [Google Cloud Architecture](https://cloud.google.com/architecture)
- [12 Factor App](https://12factor.net/)
