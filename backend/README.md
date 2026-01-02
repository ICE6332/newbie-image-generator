# ComfyUI Image Generator - Backend

Rust/Axum 后端服务，作为前端与 ComfyUI 之间的代理层。

## 技术栈

- Rust + Axum Web 框架
- Tokio 异步运行时
- WebSocket (tokio-tungstenite)
- Reqwest HTTP 客户端

## 项目结构

```
src/
├── main.rs      # 入口，服务器启动，WebSocket 监听器
├── api.rs       # 路由处理器，WebSocket handler
├── comfyui.rs   # ComfyUI HTTP 客户端，workflow 构建
├── models.rs    # 请求/响应类型，WebSocket 消息类型
├── config.rs    # 环境配置
└── error.rs     # 错误类型定义
```

## 开发命令

```bash
# 启动服务器 (端口 3000)
cargo run

# 快速类型检查
cargo check

# 格式化代码
cargo fmt

# 运行测试
cargo test

# 生产构建
cargo build --release
```

## 配置

复制 `.env.example` 为 `.env`：

```env
HOST=127.0.0.1
PORT=3000
COMFYUI_HOST=127.0.0.1
COMFYUI_PORT=8188
ALLOW_REMOTE_COMFYUI=lan  # false|lan|true
PUBLIC_BASE_URL=http://localhost:3000
CORS_ORIGINS=http://localhost:3001,http://127.0.0.1:3001
RUST_LOG=info,tower_http=debug
```

## API 端点

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | `/health` | 健康检查 |
| GET | `/api/status` | 系统状态 |
| POST | `/api/generate` | 提交图像生成 |
| GET | `/api/queue` | 队列状态 |
| GET | `/api/history/{prompt_id}` | 获取生成结果 |
| GET | `/api/images/{filename}` | 获取图片 |
| POST | `/api/interrupt` | 中断当前生成 |
| POST | `/api/clear` | 清空队列 |
| POST | `/api/test-comfyui` | 测试 ComfyUI 连接 |
| WS | `/ws` | WebSocket 实时事件 |

## 数据流

1. 前端 POST `/api/generate`
2. Backend 转换为 ComfyUI workflow JSON
3. Backend 提交到 ComfyUI 队列
4. Backend 通过 WebSocket 监听 ComfyUI 事件
5. Backend 转发事件到前端 WebSocket

## WebSocket 消息类型

`connected`, `started`, `progress`, `preview` (base64), `completed`, `error`, `queue_status`
