# GitAI 部署指南

> 🚀 **企业级部署和生产环境配置指南**

## 📋 目录

- [部署架构](#部署架构)
- [容器化部署](#容器化部署)
- [Kubernetes 部署](#kubernetes-部署)
- [云服务部署](#云服务部署)
- [高可用配置](#高可用配置)
- [监控和日志](#监控和日志)
- [安全配置](#安全配置)
- [性能优化](#性能优化)

## 🏗️ 部署架构

### 单机部署架构

```
┌─────────────────────────────────────────────────────────────┐
│                        用户工作站                           │
├─────────────────────────────────────────────────────────────┤
│  GitAI CLI                                                  │
│  ├── commit                                                 │
│  ├── review                                                 │
│  ├── scan                                                   │
│  └── mcp-server                                             │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    GitAI 服务器                             │
├─────────────────────────────────────────────────────────────┤
│  GitAI MCP Server (Port: 8080)                             │
│  ├── TreeSitter Analysis Service                           │
│  ├── AI Analysis Service                                   │
│  ├── DevOps Integration Service                            │
│  └── Security Scanning Service                             │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    AI 模型服务                              │
├─────────────────────────────────────────────────────────────┤
│  Ollama Server (Port: 11434)                               │
│  ├── qwen2.5:7b                                            │
│  ├── qwen2.5:14b                                           │
│  └── codellama:13b                                         │
└─────────────────────────────────────────────────────────────┘
```

### 分布式部署架构

```
┌─────────────────────────────────────────────────────────────┐
│                      负载均衡层                             │
├─────────────────────────────────────────────────────────────┤
│  Nginx / HAProxy                                            │
│  ├── SSL 终止                                              │
│  ├── 负载均衡                                              │
│  └── 限流保护                                              │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                    GitAI 服务集群                           │
├─────────────────────────────────────────────────────────────┤
│  GitAI Server 1    GitAI Server 2    GitAI Server 3        │
│  (Port: 8080)      (Port: 8080)      (Port: 8080)          │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                     AI 服务集群                             │
├─────────────────────────────────────────────────────────────┤
│  AI Server 1       AI Server 2       AI Server 3           │
│  (Port: 11434)     (Port: 11434)     (Port: 11434)         │
└─────────────────────────────────────────────────────────────┘
```

## 🐳 容器化部署

### Docker 单容器部署

#### 创建 Dockerfile

```dockerfile
# 多阶段构建
FROM rust:1.75-slim as builder

WORKDIR /app

# 安装依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# 复制源代码
COPY . .

# 构建应用
RUN cargo build --release

# 运行时镜像
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建用户
RUN useradd -m -s /bin/bash gitai

# 复制二进制文件
COPY --from=builder /app/target/release/gitai /usr/local/bin/

# 创建配置目录
RUN mkdir -p /home/gitai/.config/gitai \
    && chown -R gitai:gitai /home/gitai

# 切换用户
USER gitai
WORKDIR /home/gitai

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 暴露端口
EXPOSE 8080

# 启动命令
CMD ["gitai", "mcp-server", "--host", "0.0.0.0", "--port", "8080"]
```

#### 构建和运行

```bash
# 构建镜像
docker build -t gitai:latest .

# 运行容器
docker run -d \
    --name gitai-server \
    --restart=always \
    -p 8080:8080 \
    -v $(pwd)/config:/home/gitai/.config/gitai \
    -v $(pwd)/logs:/home/gitai/logs \
    gitai:latest
```

### Docker Compose 部署

#### 创建 docker-compose.yml

```yaml
version: '3.8'

services:
  gitai:
    build: .
    ports:
      - "8080:8080"
    volumes:
      - ./config:/home/gitai/.config/gitai
      - ./logs:/home/gitai/logs
    environment:
      - RUST_LOG=info
      - GITAI_AI_API_URL=http://ollama:11434/v1/chat/completions
      - GITAI_AI_MODEL=qwen2.5:7b
    depends_on:
      - ollama
      - redis
    networks:
      - gitai-net
    restart: unless-stopped

  ollama:
    image: ollama/ollama:latest
    ports:
      - "11434:11434"
    volumes:
      - ollama_data:/root/.ollama
    environment:
      - OLLAMA_HOST=0.0.0.0
    networks:
      - gitai-net
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    command: redis-server --appendonly yes
    networks:
      - gitai-net
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - gitai
    networks:
      - gitai-net
    restart: unless-stopped

volumes:
  ollama_data:
  redis_data:

networks:
  gitai-net:
    driver: bridge
```

#### 启动服务

```bash
# 启动所有服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f gitai

# 停止服务
docker-compose down
```

## ☸️ Kubernetes 部署

### 基础资源定义

#### ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: gitai-config
  namespace: gitai
data:
  config.toml: |
    [ai]
    api_url = "http://ollama-service:11434/v1/chat/completions"
    model_name = "qwen2.5:7b"
    temperature = 0.7
    
    [mcp]
    server_port = 8080
    server_host = "0.0.0.0"
    
    [logging]
    level = "info"
    format = "json"
```

#### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: gitai-deployment
  namespace: gitai
  labels:
    app: gitai
spec:
  replicas: 3
  selector:
    matchLabels:
      app: gitai
  template:
    metadata:
      labels:
        app: gitai
    spec:
      containers:
      - name: gitai
        image: gitai:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        - name: GITAI_CONFIG_PATH
          value: "/etc/gitai/config.toml"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        volumeMounts:
        - name: config
          mountPath: /etc/gitai
        - name: logs
          mountPath: /var/log/gitai
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
      volumes:
      - name: config
        configMap:
          name: gitai-config
      - name: logs
        emptyDir: {}
```

#### Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: gitai-service
  namespace: gitai
spec:
  selector:
    app: gitai
  ports:
    - protocol: TCP
      port: 80
      targetPort: 8080
  type: ClusterIP
```

#### Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: gitai-ingress
  namespace: gitai
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - gitai.your-domain.com
    secretName: gitai-tls
  rules:
  - host: gitai.your-domain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: gitai-service
            port:
              number: 80
```

### 部署命令

```bash
# 创建命名空间
kubectl create namespace gitai

# 应用所有资源
kubectl apply -f k8s/

# 查看部署状态
kubectl get pods -n gitai
kubectl get services -n gitai
kubectl get ingress -n gitai

# 查看日志
kubectl logs -f deployment/gitai-deployment -n gitai
```

## ☁️ 云服务部署

### AWS ECS 部署

#### 任务定义

```json
{
  "family": "gitai-task",
  "taskRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
  "executionRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "512",
  "memory": "1024",
  "containerDefinitions": [
    {
      "name": "gitai",
      "image": "your-repo/gitai:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/gitai",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

#### 服务配置

```bash
# 创建服务
aws ecs create-service \
  --cluster gitai-cluster \
  --service-name gitai-service \
  --task-definition gitai-task:1 \
  --desired-count 3 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[subnet-12345,subnet-67890],securityGroups=[sg-abcdef],assignPublicIp=ENABLED}"
```

### Azure Container Instances

```yaml
apiVersion: 2019-12-01
location: eastus
name: gitai-container-group
properties:
  containers:
  - name: gitai
    properties:
      image: your-registry/gitai:latest
      resources:
        requests:
          cpu: 1
          memoryInGb: 2
      ports:
      - port: 8080
        protocol: TCP
      environmentVariables:
      - name: RUST_LOG
        value: info
  osType: Linux
  restartPolicy: Always
  ipAddress:
    type: Public
    ports:
    - port: 8080
      protocol: TCP
```

### Google Cloud Run

```yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: gitai-service
  annotations:
    run.googleapis.com/ingress: all
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/maxScale: "10"
        run.googleapis.com/cpu-throttling: "false"
    spec:
      containerConcurrency: 80
      containers:
      - image: gcr.io/your-project/gitai:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: info
        resources:
          limits:
            cpu: 2
            memory: 4Gi
```

## 🔄 高可用配置

### 负载均衡配置

#### Nginx 配置

```nginx
upstream gitai_backend {
    least_conn;
    server 10.0.1.10:8080 weight=1 max_fails=3 fail_timeout=30s;
    server 10.0.1.11:8080 weight=1 max_fails=3 fail_timeout=30s;
    server 10.0.1.12:8080 weight=1 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    listen 443 ssl http2;
    server_name gitai.your-domain.com;

    ssl_certificate /etc/ssl/certs/gitai.crt;
    ssl_certificate_key /etc/ssl/private/gitai.key;

    location / {
        proxy_pass http://gitai_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # 健康检查
        proxy_next_upstream error timeout invalid_header http_500 http_502 http_503 http_504;
        proxy_connect_timeout 5s;
        proxy_send_timeout 10s;
        proxy_read_timeout 10s;
    }

    location /health {
        access_log off;
        return 200 "healthy\n";
    }
}
```

#### HAProxy 配置

```
global
    daemon
    maxconn 4096

defaults
    mode http
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms
    option httpchk GET /health

frontend gitai_frontend
    bind *:80
    bind *:443 ssl crt /etc/ssl/certs/gitai.pem
    redirect scheme https if !{ ssl_fc }
    default_backend gitai_backend

backend gitai_backend
    balance roundrobin
    option httpchk GET /health
    server gitai1 10.0.1.10:8080 check
    server gitai2 10.0.1.11:8080 check
    server gitai3 10.0.1.12:8080 check
```

### 数据备份策略

```bash
#!/bin/bash
# backup-gitai.sh - GitAI 数据备份脚本

BACKUP_DIR="/backup/gitai"
DATE=$(date +%Y%m%d_%H%M%S)

# 创建备份目录
mkdir -p "$BACKUP_DIR/$DATE"

# 备份配置文件
cp -r /etc/gitai/config.toml "$BACKUP_DIR/$DATE/"

# 备份日志文件
tar -czf "$BACKUP_DIR/$DATE/logs.tar.gz" /var/log/gitai/

# 备份数据库（如果有）
# mysqldump -u root -p gitai > "$BACKUP_DIR/$DATE/database.sql"

# 保留最近 7 天的备份
find "$BACKUP_DIR" -type d -mtime +7 -exec rm -rf {} \;

echo "备份完成: $BACKUP_DIR/$DATE"
```

## 📊 监控和日志

### Prometheus 监控

#### 监控配置

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'gitai'
    static_configs:
      - targets: ['gitai-service:8080']
    metrics_path: /metrics
    scrape_interval: 10s
```

#### 自定义指标

```rust
// src/metrics.rs
use prometheus::{Counter, Histogram, Gauge, register_counter, register_histogram, register_gauge};

lazy_static! {
    pub static ref COMMIT_REQUESTS: Counter = register_counter!(
        "gitai_commit_requests_total",
        "Total number of commit requests"
    ).unwrap();
    
    pub static ref AI_REQUEST_DURATION: Histogram = register_histogram!(
        "gitai_ai_request_duration_seconds",
        "Duration of AI requests in seconds"
    ).unwrap();
    
    pub static ref ACTIVE_CONNECTIONS: Gauge = register_gauge!(
        "gitai_active_connections",
        "Number of active connections"
    ).unwrap();
}
```

### ELK Stack 日志收集

#### Logstash 配置

```ruby
# logstash.conf
input {
  beats {
    port => 5044
  }
}

filter {
  if [fields][service] == "gitai" {
    json {
      source => "message"
    }
    
    date {
      match => [ "timestamp", "ISO8601" ]
    }
  }
}

output {
  elasticsearch {
    hosts => ["elasticsearch:9200"]
    index => "gitai-logs-%{+YYYY.MM.dd}"
  }
}
```

#### Filebeat 配置

```yaml
# filebeat.yml
filebeat.inputs:
- type: log
  enabled: true
  paths:
    - /var/log/gitai/*.log
  fields:
    service: gitai
  fields_under_root: true

output.logstash:
  hosts: ["logstash:5044"]

processors:
- add_host_metadata:
    when.not.contains.tags: forwarded
```

## 🔐 安全配置

### SSL/TLS 配置

```bash
# 生成自签名证书
openssl req -x509 -newkey rsa:4096 -keyout gitai.key -out gitai.crt -days 365 -nodes

# 或使用 Let's Encrypt
certbot certonly --webroot -w /var/www/html -d gitai.your-domain.com
```

### 网络安全

#### 防火墙配置

```bash
# UFW 配置
ufw default deny incoming
ufw default allow outgoing
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 8080/tcp
ufw enable
```

#### 安全组规则

```bash
# AWS 安全组
aws ec2 authorize-security-group-ingress \
  --group-id sg-12345678 \
  --protocol tcp \
  --port 443 \
  --cidr 0.0.0.0/0

aws ec2 authorize-security-group-ingress \
  --group-id sg-12345678 \
  --protocol tcp \
  --port 8080 \
  --source-group sg-87654321
```

### 访问控制

```toml
# config.toml
[security]
enable_authentication = true
allowed_origins = ["https://your-domain.com"]
allowed_ips = ["192.168.1.0/24", "10.0.0.0/8"]
rate_limit = 100  # requests per minute
```

## ⚡ 性能优化

### 系统优化

```bash
# 内核参数优化
echo "net.core.somaxconn = 65535" >> /etc/sysctl.conf
echo "net.ipv4.tcp_max_syn_backlog = 65535" >> /etc/sysctl.conf
echo "net.core.netdev_max_backlog = 10000" >> /etc/sysctl.conf
sysctl -p

# 文件描述符限制
echo "* soft nofile 65535" >> /etc/security/limits.conf
echo "* hard nofile 65535" >> /etc/security/limits.conf
```

### 应用优化

```toml
# config.toml
[performance]
max_concurrent_requests = 1000
request_timeout = 30
connection_pool_size = 100
cache_size = "512MB"
worker_threads = 8
```

### 缓存配置

```toml
# Redis 缓存配置
[cache]
backend = "redis"
redis_url = "redis://localhost:6379"
ttl = 3600  # 1 hour
max_entries = 10000
```

## 📋 部署检查清单

### 部署前检查

- [ ] 确认系统要求和依赖
- [ ] 准备配置文件
- [ ] 设置环境变量
- [ ] 配置 SSL 证书
- [ ] 设置监控和日志
- [ ] 准备备份策略

### 部署后验证

- [ ] 服务启动正常
- [ ] 端口监听正常
- [ ] 健康检查通过
- [ ] 日志输出正常
- [ ] 监控指标正常
- [ ] 负载均衡配置正确

### 安全检查

- [ ] 防火墙配置正确
- [ ] SSL/TLS 配置有效
- [ ] 访问控制配置正确
- [ ] 敏感信息已加密
- [ ] 日志审计配置正确

---

**🚀 现在您可以在生产环境中部署 GitAI 了！**

记住定期检查服务状态、监控系统性能、更新安全补丁，确保服务的稳定运行。