#!/bin/bash
# Deployment script for automated deployments
# Usage: ./deploy.sh <environment>

set -e  # Exit on error

ENVIRONMENT=${1:-staging}
APP_NAME="myapp"
REGISTRY="docker.io/myorg"
VERSION=$(git rev-parse --short HEAD)

echo "🚀 Starting deployment to $ENVIRONMENT..."
echo "📦 Version: $VERSION"

# Build Docker image
echo "🔨 Building Docker image..."
docker build -t $REGISTRY/$APP_NAME:$VERSION .
docker tag $REGISTRY/$APP_NAME:$VERSION $REGISTRY/$APP_NAME:latest

# Push to registry
echo "⬆️  Pushing to registry..."
docker push $REGISTRY/$APP_NAME:$VERSION
docker push $REGISTRY/$APP_NAME:latest

# Deploy based on environment
case $ENVIRONMENT in
  staging)
    echo "🎯 Deploying to staging..."
    kubectl set image deployment/$APP_NAME $APP_NAME=$REGISTRY/$APP_NAME:$VERSION -n staging
    kubectl rollout status deployment/$APP_NAME -n staging
    ;;
  production)
    echo "🎯 Deploying to production..."
    kubectl set image deployment/$APP_NAME $APP_NAME=$REGISTRY/$APP_NAME:$VERSION -n production
    kubectl rollout status deployment/$APP_NAME -n production
    ;;
  *)
    echo "❌ Unknown environment: $ENVIRONMENT"
    exit 1
    ;;
esac

echo "✅ Deployment completed successfully!"
