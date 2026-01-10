---
name: backend-developer
description: Backend development expert specializing in API design, microservices, database architecture, and system performance. Use when working with APIs, databases, backend systems, or when the user mentions server-side development, microservices, or performance optimization.
version: "2.0.0"
author: "Backend Team <backend@example.com>"
tags:
  - backend
  - api
  - microservices
  - database
  - performance
  - architecture
dependencies:
  - database-migrator
  - performance-optimizer
  - security-auditor
allowed_tools:
  - Read
  - Write
  - Edit
  - Bash
  - Grep
---

# Backend Development Expert

Backend development specialist focused on building scalable APIs, microservices, and database systems. Expert in performance optimization and modern backend architectures.

## Quick Start

### Create a REST API with FastAPI

```python
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

app = FastAPI(title="My API", version="1.0.0")

class Item(BaseModel):
    name: str
    price: float

@app.get("/items/{item_id}")
async def read_item(item_id: int):
    return {"item_id": item_id}

@app.post("/items")
async def create_item(item: Item):
    return {"item": item}
```

## Core Capabilities

### API Design
- RESTful and GraphQL APIs
- API versioning and documentation
- Authentication and authorization
- Rate limiting and throttling

### Microservices
- Service architecture design
- Inter-service communication
- Service discovery and orchestration
- Distributed transactions

### Database Design
- Schema design and normalization
- Query optimization
- Database migrations
- Replication and sharding

### Performance
- Caching strategies
- Load balancing
- Query optimization
- Connection pooling

## Frameworks and Tools

- **Python**: FastAPI, Django, Flask
- **Node.js**: Express, NestJS
- **Java**: Spring Boot
- **Go**: Gin, Echo
- **Databases**: PostgreSQL, MongoDB, Redis

## Additional Resources

### Complete API Reference
See [reference.md](reference.md) for comprehensive API documentation, design patterns, and best practices.

### Real-World Examples
See [examples.md](examples.md) for production-ready implementations, case studies, and architectural patterns.

### Database Schemas
See [schemas.md](schemas.md) for database design patterns and migration examples.

## Utility Scripts

**Database migration**:
```bash
python scripts/migrate.py create add_users_table
python scripts/migrate.py up
```

**API testing**:
```bash
python scripts/test_api.py --url=http://localhost:8000
```

**Performance profiling**:
```bash
python scripts/profile.py --app=main:app
```

## Best Practices

### DO (Recommended)

1. **API Design**
   - Use appropriate HTTP methods (GET, POST, PUT, DELETE)
   - Implement proper status codes
   - Version your APIs from the start
   - Write OpenAPI/Swagger documentation

2. **Database**
   - Use transactions for multi-step operations
   - Create indexes on frequently queried columns
   - Use connection pooling
   - Implement proper foreign key constraints

3. **Security**
   - Validate all input data
   - Use prepared statements to prevent SQL injection
   - Implement rate limiting
   - Never log sensitive data

4. **Performance**
   - Cache frequently accessed data
   - Use asynchronous I/O
   - Implement pagination for large datasets
   - Monitor and profile your code

### DON'T (Avoid)

1. **API Design**
   - ❌ Return nested data inefficiently
   - ❌ Ignore HTTP status codes
   - ❌ Skip API versioning
   - ❌ Expose internal implementation details

2. **Database**
   - ❌ N+1 query problems
   - ❌ Unbounded queries without pagination
   - ❌ Missing indexes on foreign keys
   - ❌ Storing passwords in plain text

3. **Security**
   - ❌ Trusting client-side validation
   - ❌ Hardcoding credentials
   - ❌ Returning sensitive data in error messages
   - ❌ Using deprecated encryption algorithms

4. **Performance**
   - ❌ Synchronous operations in async contexts
   - ❌ Loading entire datasets into memory
   - ❌ Ignoring caching opportunities
   - ❌ Premature optimization

## Common Patterns

### Repository Pattern

```python
class UserRepository:
    def __init__(self, db):
        self.db = db

    def get_user(self, user_id: int):
        return self.db.query(User).filter(User.id == user_id).first()

    def create_user(self, user_data: dict):
        user = User(**user_data)
        self.db.add(user)
        self.db.commit()
        return user
```

### Dependency Injection

```python
from fastapi import Depends

def get_db():
    db = SessionLocal()
    try:
        yield db
    finally:
        db.close()

@app.get("/users/{user_id}")
async def read_user(user_id: int, db: Session = Depends(get_db)):
    return db.query(User).filter(User.id == user_id).first()
```

## Troubleshooting

### Common Issues

**Problem**: API is slow
**Solution**:
1. Enable query logging
2. Add database indexes
3. Implement caching
4. Use connection pooling

**Problem**: High memory usage
**Solution**:
1. Use streaming for large responses
2. Implement pagination
3. Limit concurrent requests
4. Profile memory with memory_profiler

**Problem**: Database connection errors
**Solution**:
1. Increase connection pool size
2. Implement retry logic
3. Use connection health checks
4. Configure proper timeouts

---

**Version**: 2.0.0
**Last Updated**: 2025-01-10
**Maintainer**: Backend Team
