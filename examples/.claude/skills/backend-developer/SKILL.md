---
name: "Backend Developer"
description: "后端开发专家，精通API设计、微服务架构、数据库设计和后端系统性能优化"
version: "1.5.0"
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
---

# 后端开发专家

你是后端开发专家，专精于API设计、微服务架构、数据库设计和后端系统性能优化。帮助构建可扩展、高性能的后端系统。

## API 设计最佳实践

### RESTful API 设计

```python
# 使用 FastAPI 构建现代 REST API
from fastapi import FastAPI, HTTPException, Depends, Query, Path
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from pydantic import BaseModel, Field, validator
from typing import List, Optional
from datetime import datetime
import uuid

app = FastAPI(
    title="用户管理API",
    description="企业级用户管理系统",
    version="2.0.0",
    docs_url="/docs",
    redoc_url="/redoc"
)

# 数据模型
class UserBase(BaseModel):
    """用户基础模型"""
    email: str = Field(..., description="用户邮箱", regex=r"^[\w\.-]+@[\w\.-]+\.\w+$")
    full_name: str = Field(..., description="用户全名", min_length=2, max_length=100)

    @validator('email')
    def email_must_be_valid(cls, v):
        if '@example.com' not in v:
            raise ValueError('只允许 example.com 域名')
        return v.lower()

class UserCreate(UserBase):
    """创建用户模型"""
    password: str = Field(..., description="密码", min_length=8, max_length=100)

    @validator('password')
    def password_strength(cls, v):
        if not any(c.isupper() for c in v):
            raise ValueError('密码必须包含至少一个大写字母')
        if not any(c.isdigit() for c in v):
            raise ValueError('密码必须包含至少一个数字')
        return v

class UserUpdate(BaseModel):
    """更新用户模型"""
    email: Optional[str] = None
    full_name: Optional[str] = None
    is_active: Optional[bool] = None

class UserResponse(UserBase):
    """用户响应模型"""
    user_id: str = Field(..., description="用户ID")
    is_active: bool = Field(..., description="是否激活")
    created_at: datetime = Field(..., description="创建时间")
    updated_at: datetime = Field(..., description="更新时间")

    class Config:
        orm_mode = True

# 分页模型
class PaginatedResponse(BaseModel):
    """分页响应"""
    total: int = Field(..., description="总记录数")
    page: int = Field(..., description="当前页码")
    page_size: int = Field(..., description="每页大小")
    items: List[UserResponse]

# API 路由
@app.post(
    "/api/v2/users",
    response_model=UserResponse,
    status_code=201,
    summary="创建用户",
    description="创建新用户账户",
    responses={
        201: {"description": "用户创建成功"},
        400: {"description": "请求数据无效"},
        409: {"description": "用户已存在"}
    }
)
async def create_user(user: UserCreate):
    """创建新用户"""
    # 检查邮箱是否已存在
    existing_user = await db.users.find_one({"email": user.email})
    if existing_user:
        raise HTTPException(
            status_code=409,
            detail=f"邮箱 {user.email} 已被使用"
        )

    # 密码哈希处理
    hashed_password = hash_password(user.password)

    # 创建用户文档
    user_doc = {
        "user_id": str(uuid.uuid4()),
        "email": user.email,
        "full_name": user.full_name,
        "hashed_password": hashed_password,
        "is_active": True,
        "created_at": datetime.utcnow(),
        "updated_at": datetime.utcnow()
    }

    # 保存到数据库
    result = await db.users.insert_one(user_doc)
    user_doc["_id"] = result.inserted_id

    return UserResponse(**user_doc)

@app.get(
    "/api/v2/users",
    response_model=PaginatedResponse,
    summary="获取用户列表",
    description="分页获取用户列表，支持搜索和过滤"
)
async def list_users(
    page: int = Query(1, ge=1, description="页码"),
    page_size: int = Query(20, ge=1, le=100, description="每页大小"),
    search: Optional[str] = Query(None, description="搜索关键词"),
    is_active: Optional[bool] = Query(None, description="是否激活"),
    sort_by: str = Query("created_at", description="排序字段"),
    sort_order: str = Query("desc", regex="^(asc|desc)$", description="排序方向")
):
    """获取用户列表"""
    # 构建查询条件
    query = {}
    if is_active is not None:
        query["is_active"] = is_active
    if search:
        query["$or"] = [
            {"email": {"$regex": search, "$options": "i"}},
            {"full_name": {"$regex": search, "$options": "i"}}
        ]

    # 计算总数
    total = await db.users.count_documents(query)

    # 获取数据
    skip = (page - 1) * page_size
    sort_direction = 1 if sort_order == "asc" else -1
    cursor = db.users.find(query).sort(sort_by, sort_direction).skip(skip).limit(page_size)

    users = await cursor.to_list(length=page_size)

    return PaginatedResponse(
        total=total,
        page=page,
        page_size=page_size,
        items=[UserResponse(**user) for user in users]
    )

@app.get(
    "/api/v2/users/{user_id}",
    response_model=UserResponse,
    summary="获取用户详情",
    description="根据用户ID获取用户详细信息"
)
async def get_user(
    user_id: str = Path(..., description="用户ID")
):
    """获取用户详情"""
    user = await db.users.find_one({"user_id": user_id})
    if not user:
        raise HTTPException(status_code=404, detail="用户不存在")

    return UserResponse(**user)

@app.put(
    "/api/v2/users/{user_id}",
    response_model=UserResponse,
    summary="更新用户",
    description="更新用户信息"
)
async def update_user(
    user_id: str,
    user_update: UserUpdate
):
    """更新用户信息"""
    # 获取现有用户
    existing_user = await db.users.find_one({"user_id": user_id})
    if not existing_user:
        raise HTTPException(status_code=404, detail="用户不存在")

    # 构建更新数据
    update_data = {k: v for k, v in user_update.dict().items() if v is not None}
    update_data["updated_at"] = datetime.utcnow()

    # 检查邮箱是否被其他用户使用
    if "email" in update_data:
        email_exists = await db.users.find_one({
            "email": update_data["email"],
            "user_id": {"$ne": user_id}
        })
        if email_exists:
            raise HTTPException(status_code=409, detail="邮箱已被使用")

    # 更新数据库
    await db.users.update_one(
        {"user_id": user_id},
        {"$set": update_data}
    )

    # 返回更新后的用户
    updated_user = await db.users.find_one({"user_id": user_id})
    return UserResponse(**updated_user)

@app.delete(
    "/api/v2/users/{user_id}",
    status_code=204,
    summary="删除用户",
    description="删除指定用户"
)
async def delete_user(user_id: str):
    """删除用户"""
    result = await db.users.delete_one({"user_id": user_id})
    if result.deleted_count == 0:
        raise HTTPException(status_code=404, detail="用户不存在")

    return None
```

### GraphQL API 设计

```python
# 使用 Graphene 构建 GraphQL API
import graphene
from graphene import relay, ObjectType, Schema, Field, List, String, Int, Boolean
from datetime import datetime

class User(ObjectType):
    """用户类型"""
    class Meta:
        interfaces = (relay.Node,)

    user_id = String(required=True)
    email = String(required=True)
    full_name = String(required=True)
    is_active = Boolean()
    created_at = graphene.DateTime()
    updated_at = graphene.DateTime()

    # 关联关系
    orders = List(lambda: Order)
    posts = List(lambda: Post)

    def resolve_orders(root, info):
        """解析用户的订单"""
        return get_orders_for_user(root.user_id)

    def resolve_posts(root, info):
        """解析用户的文章"""
        return get_posts_for_user(root.user_id)

class CreateUser(graphene.Mutation):
    """创建用户突变"""
    class Arguments:
        email = String(required=True)
        full_name = String(required=True)
        password = String(required=True)

    user = Field(User)
    success = Boolean()
    message = String()

    def mutate(root, info, email, full_name, password):
        # 创建用户逻辑
        user = create_user_in_db(email, full_name, password)

        return CreateUser(
            user=user,
            success=True,
            message="用户创建成功"
        )

class Query(ObjectType):
    """查询类型"""
    node = relay.Node.Field()
    all_users = relay.ConnectionField(Connection)

    user = Field(User, user_id=String())

    def resolve_user(root, info, user_id):
        """获取单个用户"""
        return get_user_by_id(user_id)

    def resolve_all_users(root, info):
        """获取所有用户"""
        return get_all_users()

class Mutation(ObjectType):
    """突变类型"""
    create_user = CreateUser.Field()

schema = Schema(query=Query, mutation=Mutation)
```

## 微服务架构

### 服务通信

```python
# 使用 gRPC 进行服务间通信
import grpc
from concurrent import futures
import user_pb2
import user_pb2_grpc

class UserService(user_pb2_grpc.UserServiceServicer):
    """用户服务实现"""

    def GetUser(self, request, context):
        """获取用户信息"""
        user = db.get_user(request.user_id)

        if not user:
            context.set_code(grpc.StatusCode.NOT_FOUND)
            context.set_details('用户不存在')
            return user_pb2.UserResponse()

        return user_pb2.UserResponse(
            user_id=user.user_id,
            email=user.email,
            full_name=user.full_name,
            is_active=user.is_active,
            created_at=int(user.created_at.timestamp())
        )

    def ListUsers(self, request, context):
        """获取用户列表"""
        users = db.list_users(
            page=request.page,
            page_size=request.page_size
        )

        return user_pb2.UserListResponse(
            users=[
                user_pb2.UserResponse(
                    user_id=user.user_id,
                    email=user.email,
                    full_name=user.full_name,
                    is_active=user.is_active
                )
                for user in users
            ],
            total=len(users)
        )

    def CreateUser(self, request, context):
        """创建用户"""
        try:
            user = db.create_user(
                email=request.email,
                full_name=request.full_name,
                password=request.password
            )

            return user_pb2.UserResponse(
                user_id=user.user_id,
                email=user.email,
                full_name=user.full_name,
                is_active=user.is_active
            )
        except ValueError as e:
            context.set_code(grpc.StatusCode.INVALID_ARGUMENT)
            context.set_details(str(e))
            return user_pb2.UserResponse()

# 启动 gRPC 服务器
def serve():
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=10))
    user_pb2_grpc.add_UserServiceServicer_to_server(UserService(), server)

    server.add_insecure_port('[::]:50051')
    server.start()

    print("用户服务运行在端口 50051")
    server.wait_for_termination()

if __name__ == '__main__':
    serve()
```

### 服务发现与注册

```python
# 使用 Consul 进行服务发现
import consul

class ServiceRegistry:
    """服务注册中心"""

    def __init__(self, consul_host='localhost', consul_port=8500):
        self.consul = consul.Consul(host=consul_host, port=consul_port)

    def register_service(self, service_name, service_id, address, port, tags=None):
        """注册服务"""
        check = consul.Check.http(
            f"http://{address}:{port}/health",
            interval="10s",
            timeout="5s"
        )

        self.consul.agent.service.register(
            name=service_name,
            service_id=service_id,
            address=address,
            port=port,
            tags=tags or [],
            check=check
        )
        print(f"服务 {service_name} ({service_id}) 注册成功")

    def deregister_service(self, service_id):
        """注销服务"""
        self.consul.agent.service.deregister(service_id)
        print(f"服务 {service_id} 注销成功")

    def discover_service(self, service_name):
        """发现服务"""
        _, services = self.consul.health.service(service_name, passing=True)

        if not services:
            raise Exception(f"没有可用的 {service_name} 服务")

        # 简单的负载均衡：随机选择
        import random
        service = random.choice(services)

        return {
            'address': service['Service']['Address'],
            'port': service['Service']['Port']
        }

# 使用示例
registry = ServiceRegistry()

# 注册服务
registry.register_service(
    service_name='user-service',
    service_id='user-service-1',
    address='192.168.1.100',
    port=50051,
    tags=['backend', 'grpc']
)

# 发现服务
user_service = registry.discover_service('user-service')
print(f"发现用户服务: {user_service}")
```

### API 网关

```python
# 使用 Kong API Gateway
# 配置文件 (kong.yml)
_format_version: "3.0"

services:
  - name: user-service
    url: http://user-service:50051
    connect_timeout: 60000
    write_timeout: 60000
    read_timeout: 60000
    retries: 3
    protocols:
      - grpc
      - http

    routes:
      - name: user-service-route
        paths:
          - /api/v2/users
        strip_path: false
        protocols:
          - http
          - https

    plugins:
      - name: jwt
        config:
          key_claim_name: user_id
          claims_to_verify:
            - exp

      - name: rate-limiting
        config:
          minute: 100
          hour: 1000
          policy: local

      - name: cors
        config:
          origins:
            - https://example.com
          methods:
            - GET
            - POST
            - PUT
            - DELETE
          headers:
            - Accept
            - Accept-Version
            - Content-Length
            - Content-MD5
            - Content-Type
            - Date
          exposed_headers:
            - X-Total-Count
          max_age: 3600
```

## 数据库设计

### 数据库选择策略

```
关系型数据库 (RDBMS):
  使用场景:
    ✅ 需要事务支持 (ACID)
    ✅ 复杂查询和关联
    ✅ 数据一致性要求高
    ✅ 结构化数据

  推荐选择:
    - PostgreSQL: 开源、功能强大、支持 JSON
    - MySQL: 广泛使用、成熟稳定
    - SQL Server: 企业级应用
    - Oracle: 大型企业应用

NoSQL 数据库:
  文档型 (MongoDB):
    ✅ 灵活的数据结构
    ✅ 快速迭代
    ✅ JSON 数据存储

  键值型 (Redis):
    ✅ 缓存
    ✅ 会话存储
    ✅ 实时数据

  列族型 (Cassandra):
    ✅ 大规模写入
    ✅ 时间序列数据
    ✅ 分布式存储

  图数据库 (Neo4j):
    ✅ 社交网络
    ✅ 关系复杂的数据
    ✅ 路径查询
```

### 数据库优化

```sql
-- PostgreSQL 优化示例

-- 1. 索引优化
CREATE INDEX CONCURRENTLY idx_users_email ON users(email);
CREATE INDEX CONCURRENTLY idx_users_active_created ON users(is_active, created_at DESC);
CREATE INDEX CONCURRENTLY idx_users_full_name_gin ON users USING gin(to_tsvector('english', full_name));

-- 2. 分区表
CREATE TABLE orders (
    order_id BIGSERIAL,
    user_id BIGINT NOT NULL,
    order_date DATE NOT NULL,
    total_amount DECIMAL(10, 2),
    status VARCHAR(50),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) PARTITION BY RANGE (order_date);

-- 创建分区
CREATE TABLE orders_2024_q1 PARTITION OF orders
    FOR VALUES FROM ('2024-01-01') TO ('2024-04-01');

CREATE TABLE orders_2024_q2 PARTITION OF orders
    FOR VALUES FROM ('2024-04-01') TO ('2024-07-01');

-- 3. 查询优化
-- 使用 EXPLAIN ANALYZE 分析查询
EXPLAIN ANALYZE
SELECT u.user_id, u.email, COUNT(o.order_id) as order_count
FROM users u
LEFT JOIN orders o ON u.user_id = o.user_id
WHERE u.is_active = true
  AND o.order_date >= '2024-01-01'
GROUP BY u.user_id, u.email
LIMIT 100;

-- 4. 物化视图
CREATE MATERIALIZED VIEW user_order_summary AS
SELECT
    u.user_id,
    u.email,
    COUNT(o.order_id) as total_orders,
    SUM(o.total_amount) as total_spent,
    MAX(o.order_date) as last_order_date
FROM users u
LEFT JOIN orders o ON u.user_id = o.user_id
GROUP BY u.user_id, u.email;

CREATE UNIQUE INDEX ON user_order_summary(user_id);

-- 定期刷新
REFRESH MATERIALIZED VIEW CONCURRENTLY user_order_summary;

-- 5. 连接池配置 (PgBouncer)
-- pgbouncer.ini
[databases]
production = host=localhost port=5432 dbname=production

[pgbouncer]
listen_addr = 0.0.0.0
listen_port = 6432
auth_type = md5
auth_file = /etc/pgbouncer/userlist.txt
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 25
```

### Redis 缓存策略

```python
import redis
import json
from typing import Optional
from functools import wraps

redis_client = redis.Redis(
    host='localhost',
    port=6379,
    db=0,
    decode_responses=True
)

def cache_result(key_prefix: str, expire: int = 3600):
    """缓存装饰器"""
    def decorator(func):
        @wraps(func)
        async def wrapper(*args, **kwargs):
            # 生成缓存键
            cache_key = f"{key_prefix}:{args[0]}"

            # 尝试从缓存获取
            cached = redis_client.get(cache_key)
            if cached:
                return json.loads(cached)

            # 执行函数
            result = await func(*args, **kwargs)

            # 存入缓存
            redis_client.setex(
                cache_key,
                expire,
                json.dumps(result, default=str)
            )

            return result
        return wrapper
    return decorator

# 使用示例
@cache_result(key_prefix="user", expire=300)
async def get_user(user_id: str):
    """获取用户（带缓存）"""
    user = await db.users.find_one({"user_id": user_id})
    return user

# 缓存失效策略
def invalidate_user_cache(user_id: str):
    """使用户缓存失效"""
    cache_key = f"user:{user_id}"
    redis_client.delete(cache_key)

# 缓存预热
async def warm_up_cache():
    """缓存预热"""
    active_users = await db.users.find({"is_active": True}).to_list(None)

    for user in active_users:
        cache_key = f"user:{user['user_id']}"
        redis_client.setex(
            cache_key,
            300,
            json.dumps(user, default=str)
        )

    print(f"已预热 {len(active_users)} 个用户缓存")
```

## 性能优化

### 数据库连接池

```python
import asyncio
from asyncpg import create_pool
from contextlib import asynccontextmanager

class DatabasePool:
    """数据库连接池"""

    def __init__(self, dsn: str, min_size: int = 10, max_size: int = 20):
        self.dsn = dsn
        self.min_size = min_size
        self.max_size = max_size
        self.pool = None

    async def init(self):
        """初始化连接池"""
        self.pool = await create_pool(
            self.dsn,
            min_size=self.min_size,
            max_size=self.max_size,
            command_timeout=60
        )

    @asynccontextmanager
    async def acquire(self):
        """获取连接"""
        async with self.pool.acquire() as connection:
            yield connection

    async def execute(self, query: str, *args):
        """执行查询"""
        async with self.acquire() as conn:
            return await conn.execute(query, *args)

    async def fetch(self, query: str, *args):
        """获取多行"""
        async with self.acquire() as conn:
            return await conn.fetch(query, *args)

    async def fetchrow(self, query: str, *args):
        """获取单行"""
        async with self.acquire() as conn:
            return await conn.fetchrow(query, *args)

    async def close(self):
        """关闭连接池"""
        await self.pool.close()

# 使用示例
async def main():
    db = DatabasePool(
        dsn="postgresql://user:password@localhost/database",
        min_size=10,
        max_size=20
    )
    await db.init()

    # 执行查询
    users = await db.fetch(
        "SELECT * FROM users WHERE is_active = $1 LIMIT $2",
        True, 100
    )

    for user in users:
        print(user['email'])

    await db.close()
```

### 异步处理

```python
import asyncio
from aiohttp import ClientSession
from typing import List, Dict

async def fetch_user_data(session: ClientSession, user_id: str) -> Dict:
    """获取单个用户数据"""
    async with session.get(f"/api/users/{user_id}") as response:
        return await response.json()

async def fetch_multiple_users(user_ids: List[str]) -> List[Dict]:
    """并发获取多个用户数据"""
    async with ClientSession() as session:
        tasks = [
            fetch_user_data(session, user_id)
            for user_id in user_ids
        ]
        return await asyncio.gather(*tasks)

# 使用示例
async def main():
    user_ids = ["user-1", "user-2", "user-3"]
    users = await fetch_multiple_users(user_ids)
    print(users)

# 后台任务
from fastapi import FastAPI, BackgroundTasks

app = FastAPI()

def send_welcome_email(email: str, full_name: str):
    """发送欢迎邮件（后台任务）"""
    # 模拟发送邮件
    import time
    time.sleep(2)
    print(f"欢迎邮件已发送至 {email}")

@app.post("/users")
async def create_user(
    user: UserCreate,
    background_tasks: BackgroundTasks
):
    """创建用户（发送欢迎邮件作为后台任务）"""
    # 创建用户
    user = await create_user_in_db(user)

    # 添加后台任务
    background_tasks.add_task(
        send_welcome_email,
        user.email,
        user.full_name
    )

    return user
```

## 安全最佳实践

### 认证与授权

```python
from fastapi import Depends, HTTPException, status
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from passlib.context import CryptContext
from jose import JWTError, jwt
from datetime import datetime, timedelta

# 密码哈希
pwd_context = CryptContext(schemes=["bcrypt"], deprecated="auto")

# JWT 配置
SECRET_KEY = "your-secret-key"
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 30

security = HTTPBearer()

def verify_password(plain_password: str, hashed_password: str) -> bool:
    """验证密码"""
    return pwd_context.verify(plain_password, hashed_password)

def get_password_hash(password: str) -> str:
    """哈希密码"""
    return pwd_context.hash(password)

def create_access_token(data: dict, expires_delta: timedelta = None):
    """创建访问令牌"""
    to_encode = data.copy()
    if expires_delta:
        expire = datetime.utcnow() + expires_delta
    else:
        expire = datetime.utcnow() + timedelta(minutes=15)
    to_encode.update({"exp": expire})
    encoded_jwt = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt

async def get_current_user(
    credentials: HTTPAuthorizationCredentials = Depends(security)
):
    """获取当前用户"""
    token = credentials.credentials

    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        user_id: str = payload.get("sub")
        if user_id is None:
            raise HTTPException(
                status_code=status.HTTP_401_UNAUTHORIZED,
                detail="无效的认证凭据"
            )
    except JWTError:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="无效的认证凭据"
        )

    user = await get_user_by_id(user_id)
    if user is None:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="用户不存在"
        )

    return user

# 权限检查
from enum import Enum

class Role(str, Enum):
    ADMIN = "admin"
    USER = "user"
    MODERATOR = "moderator"

def require_role(required_role: Role):
    """角色权限检查"""
    async def role_checker(current_user = Depends(get_current_user)):
        if current_user.role != required_role:
            raise HTTPException(
                status_code=status.HTTP_403_FORBIDDEN,
                detail="权限不足"
            )
        return current_user
    return role_checker

# 使用示例
@app.get("/admin/dashboard")
async def admin_dashboard(
    current_user = Depends(require_role(Role.ADMIN))
):
    """管理员仪表板"""
    return {"message": f"欢迎, {current_user.full_name}"}
```

### 数据验证

```python
from pydantic import BaseModel, validator, Field
import re

class UserCreate(BaseModel):
    """用户创建模型（带验证）"""

    email: str = Field(
        ...,
        description="邮箱地址",
        regex=r"^[\w\.-]+@[\w\.-]+\.\w+$"
    )

    password: str = Field(
        ...,
        min_length=8,
        max_length=100,
        description="密码"
    )

    full_name: str = Field(
        ...,
        min_length=2,
        max_length=100
    )

    @validator('email')
    def email_must_be_valid(cls, v):
        """验证邮箱格式"""
        if not re.match(r'^[\w\.-]+@[\w\.-]+\.\w+$', v):
            raise ValueError('邮箱格式无效')
        return v.lower()

    @validator('password')
    def password_must_be_strong(cls, v):
        """验证密码强度"""
        if not any(c.isupper() for c in v):
            raise ValueError('密码必须包含至少一个大写字母')
        if not any(c.islower() for c in v):
            raise ValueError('密码必须包含至少一个小写字母')
        if not any(c.isdigit() for c in v):
            raise ValueError('密码必须包含至少一个数字')
        if not re.search(r'[!@#$%^&*(),.?":{}|<>]', v):
            raise ValueError('密码必须包含至少一个特殊字符')
        return v

    @validator('full_name')
    def name_must_not_contain_numbers(cls, v):
        """验证姓名不包含数字"""
        if any(c.isdigit() for c in v):
            raise ValueError('姓名不能包含数字')
        return v.strip()

# SQL 注入防护
def safe_query(query: str, params: dict) -> str:
    """安全的查询构造"""
    # 使用参数化查询
    sanitized_params = {
        k: escape_sql_string(str(v))
        for k, v in params.items()
    }

    return query.format(**sanitized_params)

def escape_sql_string(s: str) -> str:
    """转义 SQL 字符串"""
    return s.replace("'", "''").replace("\\", "\\\\")

# XSS 防护
import html

def sanitize_html(text: str) -> str:
    """清理 HTML 内容"""
    return html.escape(text, quote=True)

def sanitize_user_input(text: str) -> str:
    """清理用户输入"""
    # 移除危险字符
    text = re.sub(r'[<>"\']', '', text)
    # 限制长度
    return text[:1000]
```

## 监控与日志

### 结构化日志

```python
import logging
import json
from datetime import datetime

class StructuredLogger:
    """结构化日志记录器"""

    def __init__(self, service_name: str):
        self.service_name = service_name
        self.logger = logging.getLogger(service_name)
        self.logger.setLevel(logging.INFO)

        handler = logging.StreamHandler()
        handler.setFormatter(logging.Formatter('%(message)s'))
        self.logger.addHandler(handler)

    def log(self, level: str, message: str, **kwargs):
        """记录日志"""
        log_entry = {
            "timestamp": datetime.utcnow().isoformat(),
            "service": self.service_name,
            "level": level,
            "message": message,
            **kwargs
        }

        log_json = json.dumps(log_entry)
        getattr(self.logger, level.lower())(log_json)

    def info(self, message: str, **kwargs):
        """信息日志"""
        self.log("INFO", message, **kwargs)

    def error(self, message: str, **kwargs):
        """错误日志"""
        self.log("ERROR", message, **kwargs)

    def warning(self, message: str, **kwargs):
        """警告日志"""
        self.log("WARNING", message, **kwargs)

# 使用示例
logger = StructuredLogger("user-service")

logger.info(
    "用户创建成功",
    user_id="user-123",
    email="user@example.com",
    duration_ms=45
)

logger.error(
    "数据库连接失败",
    error="Connection timeout",
    host="localhost",
    port=5432
)
```

### 性能监控

```python
import time
from functools import wraps
from prometheus_client import Counter, Histogram, generate_latest

# Prometheus 指标
request_count = Counter(
    'http_requests_total',
    'Total HTTP requests',
    ['method', 'endpoint', 'status']
)

request_duration = Histogram(
    'http_request_duration_seconds',
    'HTTP request duration',
    ['method', 'endpoint']
)

def monitor_performance(func):
    """性能监控装饰器"""
    @wraps(func)
    async def wrapper(*args, **kwargs):
        start_time = time.time()

        try:
            result = await func(*args, **kwargs)

            # 记录成功指标
            duration = time.time() - start_time
            request_duration.labels(
                method='POST',
                endpoint=func.__name__
            ).observe(duration)

            request_count.labels(
                method='POST',
                endpoint=func.__name__,
                status='success'
            ).inc()

            return result

        except Exception as e:
            # 记录失败指标
            duration = time.time() - start_time
            request_duration.labels(
                method='POST',
                endpoint=func.__name__
            ).observe(duration)

            request_count.labels(
                method='POST',
                endpoint=func.__name__,
                status='error'
            ).inc()

            raise

    return wrapper

# 使用示例
@app.post("/api/users")
@monitor_performance
async def create_user(user: UserCreate):
    """创建用户（带性能监控）"""
    return await create_user_in_db(user)

@app.get("/metrics")
async def metrics():
    """Prometheus 指标端点"""
    return generate_latest()
```

## 最佳实践总结

### API 设计
```
✅ DO:
  - 使用语义化的 URL
  - 实现适当的 HTTP 状态码
  - 版本化 API
  - 提供清晰的错误信息
  - 实现请求限流
  - 使用标准认证方式
  - 编写完整的 API 文档

❌ DON'T:
  - 在 URL 中使用动词
  - 返回不统一的响应格式
  - 忽略错误处理
  - 暴露敏感信息
  - 忽略安全认证
```

### 数据库设计
```
✅ DO:
  - 规范化数据库设计
  - 创建适当的索引
  - 使用连接池
  - 实现缓存策略
  - 定期备份数据
  - 使用事务保证一致性

❌ DON'T:
  - 过度规范化
  - N+1 查询问题
  - 忘记索引
  - 忽略查询优化
  - 存储明文密码
```

### 性能优化
```
✅ DO:
  - 使用异步处理
  - 实现缓存策略
  - 数据库查询优化
  - 使用连接池
  - 实现分页
  - 监控性能指标

❌ DON'T:
  - 同步阻塞操作
  - 过度缓存
  - 忽略监控
  - 忘记清理资源
```

## 工具和资源

### 开发工具
- **Postman**: API 测试工具
- **Swagger/OpenAPI**: API 文档规范
- **PgAdmin**: PostgreSQL 管理工具
- **RedisInsight**: Redis 管理工具
- **Prometheus**: 监控系统
- **Grafana**: 数据可视化

### 框架和库
- **FastAPI**: 现代 Python Web 框架
- **Django**: 全栈 Web 框架
- **Flask**: 轻量级 Web 框架
- **gRPC**: RPC 框架
- **SQLAlchemy**: ORM 库
- **Celery**: 异步任务队列

### 文档资源
- [FastAPI 官方文档](https://fastapi.tiangolo.com/zh/)
- [RESTful API 设计指南](https://restfulapi.net/)
- [微服务架构模式](https://microservices.io/patterns/)
- [PostgreSQL 性能优化](https://www.postgresql.org/docs/performance.html)
