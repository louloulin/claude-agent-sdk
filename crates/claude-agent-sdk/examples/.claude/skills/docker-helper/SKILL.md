---
name: docker-helper
description: "Docker and container orchestration expert for development and deployment"
version: "2.3.0"
author: "Infrastructure Team <infra@example.com>"
tags:
  - docker
  - containers
  - devops
  - deployment
dependencies:
  - kubernetes-expert
---

# Docker Helper Skill

You are a Docker and containerization expert. Help with Dockerfile creation, optimization, and troubleshooting.

## Dockerfile Best Practices

### Base Images
```dockerfile
# Preferred: Use specific version tags
FROM python:3.11-slim

# Avoid: Using 'latest' tag
FROM python:latest  # ❌ Unpredictable

# Good: Alpine for small images
FROM node:20-alpine

# Good: Distroless for minimal attack surface
FROM gcr.io/distroless/python3-debian11
```

### Layer Optimization
```dockerfile
# ❌ Bad: Too many layers
RUN apt-get update
RUN apt-get install -y git
RUN apt-get install -y curl
RUN apt-get install -y vim

# ✅ Good: Combined commands
RUN apt-get update && apt-get install -y \
    git \
    curl \
    vim \
    && rm -rf /var/lib/apt/lists/*
```

### Multi-stage Builds
```dockerfile
# Build stage
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/myapp /usr/local/bin/myapp
CMD ["myapp"]
```

## Docker Compose Patterns

### Development Setup
```yaml
version: '3.8'

services:
  app:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "3000:3000"
    volumes:
      - .:/app
      - /app/node_modules
    environment:
      - NODE_ENV=development
    depends_on:
      - db
      - redis

  db:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: myapp
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

volumes:
  postgres_data:
```

### Production Setup
```yaml
version: '3.8'

services:
  app:
    image: myapp:latest
    restart: always
    environment:
      - NODE_ENV=production
      - DATABASE_URL=postgresql://user:pass@db:5432/myapp
    depends_on:
      - db
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '0.5'
          memory: 512M

  db:
    image: postgres:15-alpine
    restart: always
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      - POSTGRES_HOST_AUTH_METHOD=trust

volumes:
  postgres_data:
```

## Common Patterns

### Running Commands
```dockerfile
# Shell form (uses shell, variable expansion)
CMD node server.js

# Exec form (no shell, direct execution)
CMD ["node", "server.js"]

# ENTRYPOINT vs CMD
ENTRYPOINT ["python"]
CMD ["app.py"]  # Can be overridden
```

### Working Directory
```dockerfile
# Create if not exists
WORKDIR /app

# Better than RUN mkdir /app && cd /app
```

### User Management
```dockerfile
# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser
USER appuser

# Good practice: Don't run as root
```

### Copy vs Add
```dockerfile
# COPY: Simple file copying
COPY package.json .

# ADD: Extra features (URLs, auto-extract)
ADD https://example.com/file.tar.gz /tmp/

# Prefer COPY for local files
# Use ADD only for URLs or tar extraction
```

## Optimization Tips

### Reduce Image Size
1. Use alpine or distroless base images
2. Multi-stage builds
3. Combine RUN commands
4. Remove package caches
5. Use .dockerignore file

### .dockerignore
```
node_modules
npm-debug.log
.git
.env
*.md
tests/
.coverage
```

### Build Arguments
```dockerfile
ARG VERSION=1.0
ARG BUILD_DATE

ENV APP_VERSION=$VERSION
ENV BUILD_DATE=$BUILD_DATE

LABEL version=$VERSION
LABEL build-date=$BUILD_DATE
```

## Troubleshooting

### View Logs
```bash
# Container logs
docker logs <container-id>

# Follow logs
docker logs -f <container-id>

# Last N lines
docker logs --tail 100 <container-id>
```

### Debug Running Container
```bash
# Execute command in container
docker exec -it <container-id> sh

# Inspect container
docker inspect <container-id>

# View resource usage
docker stats
```

### Clean Up
```bash
# Remove stopped containers
docker container prune

# Remove unused images
docker image prune -a

# Remove unused volumes
docker volume prune

# Remove all unused resources
docker system prune -a
```

## Security Best Practices

### Scanning Images
```bash
# Trivy scanner
trivy image myapp:latest

# Docker Scout
docker scout quickview myapp:latest
docker scout cves myapp:latest
```

### Security Checklist
- [ ] Use specific version tags
- [ ] Run as non-root user
- [ ] Scan for vulnerabilities
- [ ] Don't include secrets
- [ ] Minimize installed packages
- [ ] Keep images updated
- [ ] Use content trust (DOCKER_CONTENT_TRUST=1)

### Secrets Management
```yaml
# ❌ Don't: Pass secrets in environment
services:
  app:
    environment:
      - API_KEY=secret123

# ✅ Do: Use Docker secrets
services:
  app:
    secrets:
      - api_key

secrets:
  api_key:
    file: ./secrets/api_key.txt
```

## Networking

### Network Modes
```yaml
# Bridge (default)
networks:
  default:
    driver: bridge

# Host networking (Linux only)
network_mode: host

# No network
network_mode: none

# Custom network
networks:
  my_network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

## Health Checks

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1
```

```yaml
services:
  app:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 10s
```

## Performance Tuning

### BuildKit
```bash
# Use BuildKit for faster builds
export DOCKER_BUILDKIT=1
docker build .
```

### Layer Caching
```dockerfile
# Copy dependency files first
COPY package.json package-lock.json ./
RUN npm install

# Then copy source code
COPY . .
```

### Parallel Builds
```bash
docker build --build-arg BUILDKIT_INLINE_CACHE=1
```

## Production Checklist

- [ ] Use specific image versions
- [ ] Multi-stage build implemented
- [ ] Non-root user configured
- [ ] Health checks defined
- [ ] Resource limits set
- [ ] Restart policy configured
- [ ] Logging configured
- [ ] Security scan passed
- [ ] Secrets externalized
- [ ] Backup strategy in place
