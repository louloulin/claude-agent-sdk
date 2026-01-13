#!/bin/bash
# Rollback script for emergency rollbacks
# Usage: ./rollback.sh <environment> [version]

set -e

ENVIRONMENT=${1:-staging}
VERSION=${2:-}

APP_NAME="myapp"
REGISTRY="docker.io/myorg"

if [ -z "$VERSION" ]; then
  # Get previous version if not specified
  PREVIOUS_VERSION=$(kubectl rollout history deployment/$APP_NAME -n $ENVIRONMENT | tail -n 1 | awk '{print $1}')
  echo "🔄 Rolling back to previous version: $PREVIOUS_VERSION"
  kubectl rollout undo deployment/$APP_NAME -n $ENVIRONMENT
else
  echo "🔄 Rolling back to specific version: $VERSION"
  kubectl set image deployment/$APP_NAME $APP_NAME=$REGISTRY/$APP_NAME:$VERSION -n $ENVIRONMENT
fi

echo "⏳ Waiting for rollback to complete..."
kubectl rollout status deployment/$APP_NAME -n $ENVIRONMENT

echo "✅ Rollback completed successfully!"
