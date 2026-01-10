# Deployment Automation Reference Guide

## Quick Reference

### Common Deployment Commands

```bash
# Deploy to staging
./scripts/deploy.sh staging

# Deploy to production
./scripts/deploy.sh production

# Rollback staging
./scripts/rollback.sh staging

# Rollback to specific version
./scripts/rollback.sh production abc1234
```

### Kubernetes Commands

```bash
# Get deployment status
kubectl get deployments -n <environment>

# View pods
kubectl get pods -n <environment>

# View logs
kubectl logs -f deployment/myapp -n <environment>

# Scale deployment
kubectl scale deployment/myapp --replicas=5 -n <environment>

# Check rollout status
kubectl rollout status deployment/myapp -n <environment>

# View rollout history
kubectl rollout history deployment/myapp -n <environment>
```

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://user:pass@host:5432/db` |
| `REDIS_URL` | Redis connection string | `redis://host:6379` |
| `API_KEY` | External API key | `sk_live_abc123` |
| `NODE_ENV` | Node environment | `production` |

## Deployment Checklist

### Pre-Deployment
- [ ] All tests passing
- [ ] Code reviewed and approved
- [ ] Security scan clean
- [ ] Documentation updated
- [ ] Migration scripts prepared
- [ ] Backup created
- [ ] Monitoring configured
- [ ] Rollback plan ready

### Post-Deployment
- [ ] Health checks passing
- [ ] Metrics normal
- [ ] No error spikes
- [ ] Database migrations successful
- [ ] Features working
- [ ] Performance acceptable
- [ ] Alerts configured

## Troubleshooting

### Deployment Fails
1. Check pod status: `kubectl get pods -n <env>`
2. View pod logs: `kubectl logs <pod-name> -n <env>`
3. Describe pod: `kubectl describe pod <pod-name> -n <env>`
4. Check events: `kubectl get events -n <env> --sort-by='.lastTimestamp'`

### High Error Rate
1. Check application logs
2. Verify database connectivity
3. Check external dependencies
4. Review recent changes
5. Consider rollback

### Slow Performance
1. Check resource usage: `kubectl top pods -n <env>`
2. Review database queries
3. Check network latency
4. Verify cache hit rates
5. Profile application

## Contact Information

- **DevOps Team**: devops@example.com
- **On-Call**: +1-555-0123 (24/7)
- **Slack**: #deployments
- **Incident Response**: https://incident.example.com
