# sodmin

Contrix Principal Server 和 coauth 部署的管理员 Web UI。项目使用 Dioxus 构建，并编译为 WebAssembly 运行。

## 范围

- **Dashboard**：服务器 profile、健康状态、存储状态和 conformance 状态。
- **Actors**：搜索、详情、设备、会话、DID/handle 和账号生命周期状态。
- **Spaces**：成员、邀请、可见性、策略，以及带审计上下文的高风险操作。
- **Devices / Capabilities**：设备信任状态、密钥状态、grant/delegation/revocation 和有效权限解释。
- **Federation**：peer、service DID、transaction、replay/fork quarantine 和 verify-actor 状态。
- **Blob/media**：配额、元数据、保留策略和避免私有 blob 枚举的诊断。
- **coauth**：账号、会话、上游身份提供者、OAuth2 client、注册令牌、通知渠道和审计日志。

`sodmin` 不实现 Contrix reducer 或授权判定，只消费 `soland` 和 `coauth` 暴露的稳定管理员 API contract。

## 开发

```bash
cargo check
cargo test
dx serve --platform web
```

启动时 UI 会读取 `/config.json`。本地开发配置示例：

```json
{
  "coauth_public_url": "http://localhost:7080"
}
```

## 构建

```bash
cargo build --release
dx build --platform web --release
```

## 容器

```bash
docker build -t sodmin .
docker run -p 9090:80 \
  -e SOLAND_URL=http://soland:8008 \
  -e COAUTH_URL=http://coauth:7080 \
  -e COAUTH_PUBLIC_URL=https://auth.example.com \
  sodmin
```

## 部署

发布的镜像把 Dioxus/WASM 静态产物放在 nginx 后。容器启动时
`docker-entrypoint.sh` 读取运行时环境，写出
`/usr/share/nginx/html/config.json` 以及一份带默认安全头的 nginx vhost。

环境变量：

| 变量 | 必填 | 用途 | 兼容旧名 |
| --- | --- | --- | --- |
| `SOLAND_URL` | 是 | soland Principal Server 内网 URL，用于 `/_palpo/`、`/_matrix/`、`/_synapse/` 兼容代理。 | `PALPO_URL`、`MATRIX_URL` |
| `COAUTH_URL` | 推荐 | coauth admin 服务的内网 URL，开启 `/auth/`、`/api/v1/auth/`、`/api/admin/`、`/authorize`、`/oauth2/`、`/.well-known/` 代理。 | `PASION_URL` |
| `COAUTH_PUBLIC_URL` | 推荐 | 浏览器侧能访问的 coauth origin，写入 `/config.json`，OAuth2 PKCE 重定向需要。 | `PASION_PUBLIC_URL` |
| `SODMIN_PORT` | 否 | nginx 监听端口（默认 `80`）。 | `PADMIN_PORT` |

旧名变量保留一个 release 周期。

生成的 nginx 配置默认下发紧的 Content-Security-Policy
（`default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; ...`）、
`X-Frame-Options: DENY`、`Referrer-Policy: no-referrer`，以及禁用
camera / microphone / geolocation / payment 的 `Permissions-Policy`。
若部署在 sodmin 之外还要连接其他域，请修改
`docker-entrypoint.sh` 中的 `connect-src`。

`/config.json` schema：

```json
{
  "coauth_public_url": "https://auth.example.com"
}
```

为兼容老构建，运行时同时接受 `pasion_public_url` 字段。新部署应使用
`coauth_public_url`。

## API Contract

`_restapi.json` 仅作为迁移辅助。目标工作流是从以下来源生成或校验 UI types：

- `soland` Principal Server Admin OpenAPI。
- `coauth` Auth / Account Admin OpenAPI。

在生成式 contract 接入前，API client 必须保留 Contrix error envelope，拒绝 URL query credential，传递 `X-Contrix-Request-Id`，为 mutation 发送 `Idempotency-Key`，并在诊断中隐藏敏感信息。

## 目录结构

```text
src/
  api/          Contrix/coauth admin API client
  components/   共享 UI 组件
  pages/        路由页面
  types/        请求/响应 DTO
  utils/        i18n、storage、config、error 和诊断工具
e2e/            Playwright smoke/stack 测试
examples/       本地部署示例，等待替换为 Contrix stack
```

## 当前缺口

详见 [`_todos.md`](./_todos.md)。剩余 P0 包括生成式 contract source-of-truth、完整 coauth 账号页面、高风险操作 approval UX，以及替换 legacy fixture 的本地 Contrix compose stack。
