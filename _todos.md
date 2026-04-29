# Contrix Principal Server Admin - Migration Plan

> 将 Palpo/Matrix admin 面板迁移为 Contrix Principal Server 管理面板

## Phase 1: 移除不必要的页面和功能

- [x] 1.1 删除 Matrix 专有页面: `destinations.rs`, `server_actions.rs`, `server_notices.rs`, `server_notifications.rs`, `billing.rs`, `auth_status.rs`, `notification_preferences.rs`, `registration_tokens.rs`, `appservices.rs`
- [x] 1.2 删除 Pasion 页面: `pasion/` 目录及所有子页面 (8 个)
- [x] 1.3 删除 Matrix/Pasion 专有 API 模块: `destinations.rs`, `palpo_admin.rs`, `pasion.rs`, `registration_tokens.rs`, `matrix.rs`, `appservices.rs`, `rooms.rs`, `users.rs`, `media.rs`, `reports.rs`, `server_info.rs`
- [x] 1.4 删除 Matrix 专有类型: `pasion.rs`, 适配 `api.rs`, 删除 `appservices.rs`
- [x] 1.5 删除 Matrix 专有组件: `media_ops.rs`, `scheduled_commands.rs`, `server_notices.rs`, `experimental_features.rs`, `user_account_data.rs`, `user_import.rs`, `user_rate_limits.rs`
- [x] 1.6 删除 Matrix 专有工具: `mxid.rs`, `instance_config.rs`

## Phase 2: 创建 Contrix 类型定义

- [x] 2.1 重写 `types/api.rs` → Contrix 核心类型 (Actor, Space, Device, Applet, Agent, Report, Capability, Federation, Policy, Blob, Audit, etc.)
- [x] 2.2 重写 `types/appservices.rs` → `types/applets.rs` (Contrix Applet 类型)
- [x] 2.3 删除 `types/pasion.rs`
- [x] 2.4 更新 `types/mod.rs`

## Phase 3: 创建 Contrix API 层

- [x] 3.1 重写 `api/client.rs` → Contrix admin API client
- [x] 3.2 重写 `api/auth.rs` → coauth OAuth2 认证
- [x] 3.3 新建 `api/coauth.rs` → coauth admin API (用户/会话/上游/OAuth2管理)
- [x] 3.4 新建 `api/server.rs` → Contrix 服务器信息/统计/状态
- [x] 3.5 新建 `api/actors.rs` → Actor 管理接口
- [x] 3.6 新建 `api/spaces.rs` → Space 管理接口
- [x] 3.7 新建 `api/devices.rs` → Device 管理接口
- [x] 3.8 新建 `api/capabilities.rs` → Capability 管理接口
- [x] 3.9 新建 `api/federation.rs` → Federation 管理接口
- [x] 3.10 新建 `api/applets.rs` → Applet 管理接口
- [x] 3.11 新建 `api/agents.rs` → Agent 管理接口
- [x] 3.12 新建 `api/reports.rs` → Report 管理接口
- [x] 3.13 新建 `api/invite_tokens.rs` → Invite token 管理接口
- [x] 3.14 新建 `api/audit.rs` → Audit log 接口
- [x] 3.15 新建 `api/policy.rs` → Policy 管理接口
- [x] 3.16 新建 `api/media.rs` → Blob/Media 管理接口
- [x] 3.17 更新 `api/mod.rs`

## Phase 4: 创建/适配 Contrix 页面

- [x] 4.1 重写 Dashboard → Contrix 仪表盘 (server stats/info)
- [x] 4.2 重写 `users/` → `actors/` (Actor 列表、创建、详情)
- [x] 4.3 重写 `rooms/` → `spaces/` (Space 列表、创建、详情)
- [x] 4.4 重写 `media.rs` → Contrix Blob 管理 (统计/按Actor/列表)
- [x] 4.5 重写 `reports.rs` → `reports/` (Report 列表、详情)
- [x] 4.6 重写 `server_status.rs` → Contrix 服务器状态
- [x] 4.7 重写 `appservices.rs` → `applets.rs` (Applet 管理)
- [x] 4.8 新建 `capabilities.rs` (Capability 管理页)
- [x] 4.9 新建 `federation/` (联邦管理页, 含详情)
- [x] 4.10 新建 `devices.rs` (设备管理页)
- [x] 4.11 新建 `agents/` (Agent 管理页, 含详情/内存)
- [x] 4.12 新建 `audit.rs` (审计日志页)
- [x] 4.13 新建 `invite_tokens.rs` (邀请令牌管理)
- [x] 4.14 新建 `policy.rs` (策略管理页)
- [x] 4.15 简化 `login.rs` → coauth OAuth2 登录
- [x] 4.16 重写 `oauth_callback.rs` → 简化认证流程
- [x] 4.17 新建 `coauth/` 页面 (审计日志/OAuth2会话/个人令牌/上游提供商/上游链接/注册令牌/通知频道/通知模板/连接器健康)

## Phase 5: 更新路由和导航

- [x] 5.1 重写 `router.rs` → Contrix 路由
- [x] 5.2 重写 `sidebar.rs` → Contrix 导航菜单
- [x] 5.3 更新 `pages/mod.rs` → Contrix 模块导出
- [x] 5.4 更新 `api/mod.rs` → Contrix API 模块导出
- [x] 5.5 更新 `types/mod.rs` → Contrix 类型导出
- [x] 5.6 更新 `components/mod.rs` → Contrix 组件导出
- [x] 5.7 更新 `utils/mod.rs` → 工具模块导出
- [x] 5.8 更新键盘快捷键 (g a → Actors, g s → Spaces)

## Phase 6: 更新辅助模块

- [x] 6.1 重写 `utils/config.rs` → Contrix 配置 (coauth_public_url)
- [x] 6.2 更新 `Cargo.toml` → 添加 contrix-sdk 依赖, 重命名包
- [x] 6.3 更新 `main.rs` → Contrix 入口
- [x] 6.4 更新 `components/ui/page_header.rs` → 支持 Route 类型面包屑

## Phase 7: 生成 REST API 规格

- [x] 7.1 生成 `_restapi.json` → 管理后端 REST API 要求及数据格式
