---
name: logging-and-monitoring
description: "Comprehensive logging, monitoring, and observability expert for distributed systems"
version: "1.6.0"
author: "SRE Team <sre@example.com>"
tags:
  - logging
  - monitoring
  - observability
  - sre
dependencies:
  - performance-optimizer
---

# Logging & Monitoring Skill

You are an observability expert. Implement comprehensive logging and monitoring strategies.

## Logging Best Practices

### Structured Logging
```rust
use tracing::{info, warn, error, instrument};

// ✅ Structured logging with context
#[instrument(skip(password))]
async fn login(username: &str, password: &str) -> Result<User> {
    info!(username = %username, "Login attempt");

    match authenticate(username, password).await {
        Ok(user) => {
            info!(user_id = %user.id, "Login successful");
            Ok(user)
        }
        Err(e) => {
            warn!(error = %e, username = %username, "Login failed");
            Err(e)
        }
    }
}

// ❌ Unstructured logging
async fn login_bad(username: &str, password: &str) -> Result<User> {
    println!("User {} is trying to login", username); // Bad practice
    authenticate(username, password).await
}
```

### Log Levels
```
ERROR: System errors requiring immediate attention
  - Service failures
  - Data corruption
  - Security breaches

WARN: Warning conditions that should be investigated
  - High error rates
  - Performance degradation
  - Unusual patterns

INFO: Normal operational messages
  - Service start/stop
  - Configuration changes
  - Business events

DEBUG: Detailed diagnostic information
  - Function entry/exit
  - Variable values
  - Execution flow

TRACE: Most detailed information
  - Every operation
  - Internal state changes
  - Performance metrics
```

### Log Aggregation
```yaml
# ELK Stack (Elasticsearch, Logstash, Kibana)
version: '3'
services:
  elasticsearch:
    image: elasticsearch:8.0.0
    environment:
      - discovery.type=single-node
      - "ES_JAVA_OPTS=-Xms512m -Xmx512m"

  logstash:
    image: logstash:8.0.0
    volumes:
      - ./logstash.conf:/usr/share/logstash/pipeline/logstash.conf

  kibana:
    image: kibana:8.0.0
    ports:
      - "5601:5601"
```

### Centralized Logging
```python
# Python: Structlog
import structlog

logger = structlog.get_logger()
logger.info("user_logged_in", user_id=123, username="john")

# JSON output:
# {"event": "user_logged_in", "user_id": 123, "username": "john", "level": "info"}
```

## Metrics Collection

### Key Metrics
```rust
use prometheus::{Counter, Histogram, Gauge, Registry};

// Counter: Monotonically increasing value
let request_count = Counter::new(
    "http_requests_total",
    "Total number of HTTP requests"
)?;

// Histogram: Distribution of values
let request_duration = Histogram::with_opts(
    HistogramOpts::new(
        "http_request_duration_seconds",
        "HTTP request latencies in seconds"
    ).buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
)?;

// Gauge: Point-in-time value
let active_connections = Gauge::new(
    "http_active_connections",
    "Current number of active HTTP connections"
)?;

// Record metrics
request_count.inc();
request_duration.observe(0.123);
active_connections.inc();
```

### Metric Types
```
Counters: Events, requests, errors
  - api_requests_total
  - errors_total
  - jobs_completed_total

Gauges: Current state
  - active_connections
  - memory_usage_bytes
  - queue_size

Histograms: Distributions
  - request_duration_seconds
  - response_size_bytes
  - database_query_duration_seconds

Summaries: Quantiles
  - response_time{quantile="0.5"}
  - response_time{quantile="0.95"}
  - response_time{quantile="0.99"}
```

### Metric Naming
```
Best Practices:
✅ http_request_duration_seconds
✅ api_request_count
✅ database_connections_active

❌ http_request_duration
❌ count
❌ db_conns

Pattern: <unit>_<metric>_<type>
```

## Distributed Tracing

### OpenTelemetry Setup
```rust
use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry::global;

// Initialize tracer
let tracer = opentelemetry_jaeger::new_pipeline()
    .with_service_name("my-service")
    .install_simple()?;

// Create spans
let tracer = global::tracer("my-service");
let span = tracer.start("process_request");

let cx = opentelemetry::Context::current_with_span(span);

// Add attributes
span.set_attribute("user.id", 123);
span.set_attribute("http.method", "GET");
span.set_attribute("http.url", "/api/users");

// End span
span.end();
```

### Trace Context Propagation
```rust
// HTTP headers
use opentelemetry::global;

let tracer = global::tracer("http-client");
let mut span = tracer.start("http_request");

// Inject context into HTTP headers
let mut headers = reqwest::header::HeaderMap::new();
global::get_text_map_propagator(|propagator| {
    propagator.inject_context(&cx, &mut headers)
});

// Extract context from incoming headers
let cx = global::get_text_map_propagator(|propagator| {
    propagator.extract(&cx, &headers)
});
```

### Trace Analysis
```
Tools:
- Jaeger: Distributed tracing platform
- Zipkin: Twitter's tracing system
- Grafana Tempo: Grafana's tracing backend
- AWS X-Ray: AWS tracing service

Key Metrics:
- Trace latency
- Span duration
- Error rate per service
- Request path analysis
```

## Application Performance Monitoring (APM)

### APM Tools Comparison
```
New Relic:
  ✅ Easy setup
  ✅ Good UI
  ✅ Comprehensive
  ❌ Expensive
  ❌ Vendor lock-in

Datadog:
  ✅ Rich integrations
  ✅ Good dashboards
  ✅ Alert management
  ❌ Complex pricing
  ❌ Learning curve

Prometheus + Grafana:
  ✅ Open source
  ✅ Flexible
  ✅ Powerful query language
  ❌ More setup required
  ❌ Scaling challenges

Jaeger:
  ✅ Distributed tracing
  ✅ Open source
  ✅ Kubernetes native
  ❌ Storage complexity
  ❌ Operational overhead
```

### APM Implementation
```yaml
# Prometheus configuration
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'my-app'
    static_configs:
      - targets: ['localhost:3000']
    metrics_path: '/metrics'
```

## Alerting

### Alert Design Principles
```
1. Meaningful: Alert must indicate a real problem
2. Actionable: Recipient must be able to take action
3. Timely: Alert sent when action can help
4. Specific: Clear description of the issue
5. Documented: Runbook available for resolution
```

### Alert Examples
```yaml
# Prometheus Alerting Rules
groups:
  - name: api_alerts
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: rate(api_errors_total[5m]) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High API error rate"
          description: "Error rate is {{ $value }} errors/sec"

      # High latency
      - alert: HighLatency
        expr: histogram_quantile(0.95, api_request_duration_seconds) > 1
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "High API latency"
          description: "P95 latency is {{ $value }}s"

      # Service down
      - alert: ServiceDown
        expr: up{job="my-app"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "Service is down"
          description: "{{ $labels.instance }} is not responding"
```

### Alert Routing
```
Critical alerts: PagerDuty, SMS, Phone call
Warning alerts: Slack, Email
Info alerts: Dashboard, Log aggregation

On-call rotation:
- Primary: Immediate notification
- Secondary: Escalation after 15min
- Manager: Escalation after 30min
```

## Dashboarding

### Grafana Dashboard
```json
{
  "dashboard": {
    "title": "Application Metrics",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])"
          }
        ]
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "rate(http_errors_total[5m])"
          }
        ]
      },
      {
        "title": "P95 Latency",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, http_request_duration_seconds)"
          }
        ]
      }
    ]
  }
}
```

### Key Metrics Dashboard
```
Service Health:
- Request rate (QPS)
- Error rate (%)
- Latency (p50, p95, p99)
- Active connections
- CPU usage
- Memory usage

Infrastructure:
- Server health
- Database connections
- Cache hit rate
- Disk I/O
- Network bandwidth

Business:
- Orders per minute
- Active users
- Revenue
- Conversion rate
```

## Log Analysis

### Common Patterns
```bash
# Find errors in logs
grep "ERROR" /var/log/app.log

# Count errors by type
grep "ERROR" /var/log/app.log | awk '{print $3}' | sort | uniq -c

# Find slow requests
awk '/duration/ && $NF > 1000 {print}' /var/log/app.log

# Time-based analysis
grep "2024-01-10" /var/log/app.log | grep "ERROR"

# Real-time monitoring
tail -f /var/log/app.log | grep --line-buffered "ERROR"
```

### Log Queries
```sql
-- Elasticsearch DSL
{
  "query": {
    "bool": {
      "must": [
        {"match": {"level": "ERROR"}},
        {"range": {"timestamp": {"gte": "now-1h"}}}
      ]
    }
  },
  "aggs": {
    "by_error_type": {
      "terms": {"field": "error_type.keyword"}
    }
  }
}
```

## Observability Strategy

### The Three Pillars
```
1. Logs: What happened?
   - Event records
   - Error messages
   - Debug information

2. Metrics: How much?
   - Counts
   - Gauges
   - Histograms

3. Traces: Where?
   - Request flow
   - Service dependencies
   - Latency breakdown
```

### Golden Signals
```
1. Latency: How long does it take?
2. Traffic: How much load?
3. Errors: How many failing?
4. Saturation: How full is the system?
5. Availability: Is the service up?
```

### SLO/SLI/SLA
```
SLI (Service Level Indicator):
  - Request success rate: 99.9%
  - Response time: < 200ms (p95)
  - Uptime: 99.95%

SLO (Service Level Objective):
  - 99.9% of requests succeed in a month
  - 95% of requests complete in < 200ms

SLA (Service Level Agreement):
  - Financial commitment
  - Penalties for missing SLO
  - Credits for downtime
```

## Incident Response

### Runbook Template
```markdown
# Alert: High Error Rate

## Severity
Critical

## Trigger
Error rate > 5% for 5 minutes

## Diagnosis Steps
1. Check dashboard: http://grafana/dashboard/errors
2. Check recent deployments
3. Check database status
4. Check external dependencies

## Resolution Steps
1. Identify root cause
2. Implement fix
3. Deploy to staging
4. Test thoroughly
5. Deploy to production
6. Monitor for recurrence

## Escalation
- Primary: on-call@example.com
- Secondary: manager@example.com (if no response in 15min)

## Post-Incident
- File incident report
- Update runbook
- Schedule post-mortem
```

## Tools & Resources

### Logging Tools
- **ELK Stack**: Elasticsearch, Logstash, Kibana
- **Fluentd**: Log collector
- **Loki**: Grafana's log aggregation
- **Splunk**: Enterprise log analysis

### Monitoring Tools
- **Prometheus**: Metrics collection
- **Grafana**: Visualization
- **Thanos**: Long-term Prometheus storage
- **VictoriaMetrics**: Efficient metrics storage

### Tracing Tools
- **Jaeger**: Distributed tracing
- **Zipkin**: Twitter's tracing system
- **Tempo**: Grafana's tracing backend
- **Honeycomb**: Observability platform

### Documentation
- [Google SRE Book](https://sre.google/sre-book/table-of-contents/)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/)
- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
