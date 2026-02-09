# Kubeowler Go 迁移规划

本文档描述将 Kubeowler 从 Rust 重建为 Go 的架构规划、模块映射、实施顺序与注意事项。适用于在添加前端前进行的语言迁移。

---

## 1. 迁移目标与原则

### 1.1 目标

- **功能对等**：保持现有 CLI、报告格式（md/json/csv/html）、巡检模块、节点采集、评分逻辑完全一致
- **前端就绪**：Go 版本作为库可被 HTTP API 调用，便于后期接入 Web 前端
- **生态优势**：利用 client-go 与 K8s 生态，简化开发与维护

### 1.2 原则

- **渐进式迁移**：按模块分批实现，每阶段可单独验证
- **复用现有资产**：`scripts/node-check-universal.sh`、`docs/issues/*.md`、`deploy/node-inspector/` 全部保留
- **报告兼容**：输出报告格式、字段、表格结构与 Rust 版本保持一致，便于对比验证

---

## 2. 当前 Rust 架构概要

```
kubeowler/
├── src/
│   ├── main.rs              # CLI 入口
│   ├── lib.rs               # 库入口
│   ├── cli/                 # 命令行解析 (clap)
│   ├── k8s/                 # K8s 客户端封装
│   ├── inspections/         # 14 个巡检模块
│   │   ├── runner.rs        # 主调度器
│   │   ├── types.rs         # InspectionResult, ClusterReport, Issue 等
│   │   ├── issue_codes.rs   # 规则 ID → 文档路径
│   │   └── *.rs             # nodes, pods, network, storage, ...
│   ├── scoring/             # 加权评分引擎
│   ├── node_inspection/     # DaemonSet Pod 日志采集
│   ├── reporting/           # Markdown/JSON/CSV/HTML 报告生成
│   └── utils/               # 格式化、资源解析等
├── scripts/
│   └── node-check-universal.sh   # 节点采集脚本（不改动）
├── deploy/node-inspector/        # DaemonSet（不改动）
└── docs/issues/                  # 规则文档（不改动）
```

### 2.1 核心数据流

1. CLI 解析参数 → 构建 `K8sClient`
2. `InspectionRunner.run_inspections()` 依次调用各模块
3. 可选：`collect_node_inspections()` 拉取 DaemonSet Pod 日志
4. 汇总为 `ClusterReport`（含 inspections + node_inspection_results）
5. `ScoringEngine` 计算总分与 executive summary
6. `ReportGenerator` 生成报告并写入文件

---

## 3. Go 目标架构

```
kubeowler-go/
├── cmd/
│   └── kubeowler/
│       └── main.go              # CLI 入口
├── internal/
│   ├── cli/                     # cobra/viper 命令行
│   ├── k8s/                     # client-go 封装
│   ├── inspections/             # 14 个巡检模块
│   │   ├── runner.go
│   │   ├── types.go
│   │   ├── issue_codes.go
│   │   ├── nodes.go
│   │   ├── pods.go
│   │   └── ...
│   ├── scoring/                 # 评分引擎
│   ├── nodeinspection/          # DaemonSet 日志采集
│   ├── reporting/               # 报告生成
│   └── utils/                   # 工具函数
├── pkg/                         # 可对外暴露的公共 API（供未来 API 服务器使用）
│   └── kubeowler/
│       └── api.go               # RunInspection(ctx, opts) -> ClusterReport
├── scripts/
│   └── node-check-universal.sh  # 符号链接或复制自原项目
├── deploy/
│   └── node-inspector/          # 符号链接或复制自原项目
├── docs/
│   └── issues/                  # 符号链接或复制自原项目
├── go.mod
├── go.sum
└── Makefile
```

---

## 4. Rust → Go 模块映射

| Rust 模块 | Go 模块 | 说明 |
|-----------|---------|------|
| `cli/mod.rs` | `internal/cli/*.go` | cobra 替代 clap |
| `k8s/client.rs` | `internal/k8s/client.go` | client-go 替代 kube-rs |
| `inspections/runner.rs` | `internal/inspections/runner.go` | 主调度逻辑 |
| `inspections/types.rs` | `internal/inspections/types.go` | struct 定义 |
| `inspections/issue_codes.rs` | `internal/inspections/issue_codes.go` | 规则 ID 映射 |
| `inspections/nodes.rs` | `internal/inspections/nodes.go` | Node 巡检 |
| `inspections/pods.rs` | `internal/inspections/pods.go` | Pod 巡检 |
| `inspections/network.rs` | `internal/inspections/network.go` | Network 巡检 |
| `inspections/storage.rs` | `internal/inspections/storage.go` | Storage 巡检 |
| `inspections/resources.rs` | `internal/inspections/resources.go` | Resource 巡检 |
| `inspections/security.rs` | `internal/inspections/security.go` | Security 巡检 |
| `inspections/control_plane.rs` | `internal/inspections/controlplane.go` | ControlPlane 巡检 |
| `inspections/autoscaling.rs` | `internal/inspections/autoscaling.go` | Autoscaling 巡检 |
| `inspections/batch.rs` | `internal/inspections/batch.go` | Batch/CronJob 巡检 |
| `inspections/policies.rs` | `internal/inspections/policies.go` | Policies 巡检 |
| `inspections/observability.rs` | `internal/inspections/observability.go` | Observability 巡检 |
| `inspections/certificates.rs` | `internal/inspections/certificates.go` | Certificates 巡检 |
| `inspections/upgrade.rs` | `internal/inspections/upgrade.go` | Upgrade 巡检 |
| `inspections/namespace_summary.rs` | `internal/inspections/namespacesummary.go` | Namespace 摘要 |
| `scoring/scoring_engine.rs` | `internal/scoring/engine.go` | 评分逻辑 |
| `node_inspection/collector.rs` | `internal/nodeinspection/collector.go` | Pod 日志采集 |
| `node_inspection/types.rs` | `internal/nodeinspection/types.go` | NodeInspectionResult |
| `reporting/generator.rs` | `internal/reporting/generator.go` | Markdown 生成 |
| `reporting/md_export.rs` | `internal/reporting/export.go` | JSON/CSV/HTML 导出 |
| `utils/format.rs` | `internal/utils/format.go` | 字符串截断等 |
| `utils/resource_quantity.go` | `internal/utils/resource.go` | CPU/Memory 解析 |

---

## 5. Go 依赖规划

### 5.1 核心依赖

| 用途 | Go 包 | 对应 Rust |
|------|-------|-----------|
| K8s 客户端 | `k8s.io/client-go` | kube, k8s-openapi |
| K8s API 类型 | `k8s.io/api` | k8s-openapi |
| CLI | `github.com/spf13/cobra` | clap |
| 配置 | `github.com/spf13/viper` | - |
| JSON/YAML | `encoding/json`, `gopkg.in/yaml.v3` | serde_json, serde_yaml |
| UUID | `github.com/google/uuid` | uuid |
| Markdown → HTML | `github.com/gomarkdown/markdown` 或 `github.com/russross/blackfriday` | comrak |
| X.509 证书 | `crypto/x509` | x509-parser |
| HTTP 请求 | `net/http` | reqwest |

### 5.2 go.mod 示例

```go
module github.com/Ghostwritten/kubeowler

go 1.21

require (
    k8s.io/api v0.28.0
    k8s.io/apimachinery v0.28.0
    k8s.io/client-go v0.28.0
    github.com/spf13/cobra v1.8.0
    github.com/google/uuid v1.5.0
    github.com/gomarkdown/markdown v0.0.0-20231222211730-1d6d20845b47
    // 按需补充
)
```

---

## 6. 核心类型迁移（Rust → Go）

### 6.1 主要 struct

```go
// ClusterReport
type ClusterReport struct {
    ClusterName           string
    ReportID              string
    Timestamp             time.Time
    OverallScore          float64
    ExecutiveSummary      ExecutiveSummary
    Inspections           []InspectionResult
    ClusterOverview       *ClusterOverview
    NodeInspectionResults []NodeInspectionResult
    RecentEvents          []EventRow
}

// InspectionResult
type InspectionResult struct {
    InspectionType       string
    Timestamp            time.Time
    OverallScore         float64
    Checks               []CheckResult
    Summary              InspectionSummary
    CertificateExpiries  []CertificateExpiryRow
    PodContainerStates   []PodContainerStateRow
    NamespaceSummaryRows []NamespaceSummaryRow
}

// Issue
type Issue struct {
    Severity       IssueSeverity // Info, Warning, Critical
    Category       string
    Description    string
    Resource       *string
    Recommendation string
    RuleID         *string
}

// NodeInspectionResult (与 node-check-universal.sh 输出 JSON 对齐)
type NodeInspectionResult struct {
    NodeName             string
    Hostname             string
    Timestamp            string
    Runtime              string
    Resources            NodeResources
    Services             NodeServices
    Security             NodeSecurity
    Kernel               NodeKernel
    NodeDisks            []NodeDiskMount
    NodeCertificates     []NodeCertificate
    ContainerStateCounts map[string]uint32
    ZombieCount          *uint32
    // ... 与现有 JSON schema 一致
}
```

### 6.2 IssueSeverity 枚举

```go
type IssueSeverity string

const (
    SeverityInfo    IssueSeverity = "Info"
    SeverityWarning IssueSeverity = "Warning"
    SeverityCritical IssueSeverity = "Critical"
)
```

---

## 7. 实施阶段与顺序

### Phase 1：基础框架（约 1 周）

1. 初始化 `go mod`、目录结构
2. 实现 `internal/cli`：cobra 子命令 `kubeowler check`
3. 实现 `internal/k8s/client.go`：基于 client-go 的集群连接、Nodes/Pods/... 列表
4. 实现 `internal/inspections/types.go`：所有核心 struct
5. 实现 `internal/utils/resource.go`：`ParseCPUQuantity`, `ParseMemoryQuantity`

**验收**：能连接集群并列出 Node/Pod 等资源。

### Phase 2：评分与 Runner（约 1 周）

1. 实现 `internal/scoring/engine.go`：`CalculateWeightedScore`、`GetHealthStatus`、权重表
2. 实现 `internal/inspections/runner.go`：按 `InspectionType` 调度各模块的骨架
3. 实现 `internal/inspections/issue_codes.go`：RuleID → 文档路径

**验收**：Runner 能调用空模块并返回空的 ClusterReport，评分逻辑可单测。

### Phase 3：巡检模块（约 2–3 周）

按依赖顺序实现：

| 顺序 | 模块 | 依赖 | 复杂度 |
|------|------|------|--------|
| 1 | nodes | Nodes API, cluster overview | 中 |
| 2 | control_plane | Pods (kube-system) | 中 |
| 3 | network | Services, Endpoints, Deployments | 中 |
| 4 | storage | PV, PVC, StorageClass | 中 |
| 5 | resources | Pods, Namespaces, Metrics | 高 |
| 6 | pods | Pods, Events | 高 |
| 7 | autoscaling | HPA, VPA | 低 |
| 8 | batch | CronJob, Job | 中 |
| 9 | security | Pods, RBAC | 高 |
| 10 | policies | ResourceQuota, LimitRange, PDB | 中 |
| 11 | observability | Deployments, DaemonSets | 低 |
| 12 | namespace_summary | Namespaces, Deployments, NetworkPolicy | 低 |
| 13 | certificates | CSR, Secrets (TLS) | 中 |
| 14 | upgrade | Nodes, API version | 低 |

每完成 2–3 个模块，运行一次完整巡检并与 Rust 版报告对比。

### Phase 4：节点采集（约 3–5 天）

1. 实现 `internal/nodeinspection/types.go`：与 `node-check-universal.sh` JSON 对齐
2. 实现 `internal/nodeinspection/collector.go`：列出 DaemonSet Pod、拉取 container 日志、解析 JSON、填充 `container_state_counts`
3. 在 Runner 中集成：`inspection_type == All` 时调用 collector

**验收**：部署 DaemonSet 后，Go 版报告包含与 Rust 版一致的 Node Inspection 表格。

### Phase 5：报告生成（约 1–2 周）

1. 实现 `internal/reporting/generator.go`：Markdown 报告（Cluster Overview、Executive Summary、Node Inspection、Detailed Results、Key Findings、Recommendations）
2. 实现 `internal/reporting/export.go`：JSON（ClusterReport 序列化）、CSV（解析 MD 表）、HTML（MD → HTML）
3. 实现 `internal/cli` 的 `--output`、`--format`、`--level` 参数

**验收**：四种格式报告与 Rust 版在结构、表格、链接上一致（允许少量格式差异）。

### Phase 6：CLI 完善与测试（约 1 周）

1. 实现 `--namespace`、`--node-inspector-namespace`、`--config-file`、`--level`
2. 默认输出路径：`{cluster}-kubernetes-inspection-report-{YYYY-MM-DD-HHMMSS}.{ext}`
3. 单元测试：scoring、resource 解析、JSON 解析
4. 集成测试：对同一集群分别运行 Rust 与 Go 版，对比报告关键字段
5. 文档：安装、CLI 参考、与现有 docs 对齐

### Phase 7：构建与发布（约 2–3 天）

1. Makefile：`build`、`build-linux-amd64`、`build-linux-arm64`、`test`
2. 静态链接 / CGO_ENABLED=0，确保 CentOS 7 兼容
3. GitHub Actions：构建 amd64/arm64 二进制并发布
4. 更新 README、CHANGELOG，标注 Go 版为正式版本

---

## 8. 关键实现注意事项

### 8.1 K8s 客户端

- 使用 `clientcmd.BuildConfigFromFlags` 或 `clientcmd.BuildConfigFromKubeconfigGetter` 加载 kubeconfig
- `rest.InClusterConfig()` 支持集群内运行（CronJob/Deployment）
- List 使用 `ListOptions` 指定 namespace、label selector

### 8.2 资源量解析

- Kubernetes `resource.Quantity` 格式（如 `100m`、`1Gi`）需与 Rust 版 `parse_cpu_str`、`parse_memory_str` 逻辑一致
- 可参考 `k8s.io/apimachinery/pkg/api/resource` 的 `Quantity` 类型

### 8.3 报告表格

- Markdown 表头、对齐、链接格式需与 Rust 版逐项对照
- Issue 文档路径：`docs/issues/{CODE}.md`，与 `issue_codes` 映射一致

### 8.4 节点采集

- 只取每个 Pod 的**最后一行**或**完整 stdout**（脚本只输出一行 JSON）
- 空日志或解析失败时跳过该 Pod，不中断整体采集
- `container_state_counts` 从集群内所有 Pod 聚合，按 node 填充

### 8.5 并发

- Go 可利用 goroutine 并行拉取 Pod 日志、并行执行部分巡检模块，以缩短总耗时
- 注意 client-go 的 rate limiting，避免对 API Server 造成压力

---

## 9. 保留不改动的资产

| 路径 | 说明 |
|------|------|
| `scripts/node-check-universal.sh` | 节点采集脚本，JSON 结构稳定 |
| `deploy/node-inspector/Dockerfile` | 镜像构建 |
| `deploy/node-inspector/daemonset.yaml` | DaemonSet 配置 |
| `docs/issues/*.md` | 规则文档 |
| `docs/node-inspection-schema.md` | 节点 JSON schema |
| `docs/node-inspector-build-deploy.md` | 部署说明 |
| `docs/node-inspector-limitations.md` | 限制说明 |

---

## 10. 风险评估与缓解

| 风险 | 缓解措施 |
|------|----------|
| 报告格式差异 | 建立 Rust/Go 报告对比测试，逐表校验 |
| 规则逻辑遗漏 | 按 issue code 逐条对照 Rust 实现 |
| client-go 版本与集群不兼容 | 使用与目标 K8s 版本匹配的 client-go 分支 |
| 性能回退 | 用 goroutine 并行化，必要时加 profile |

---

## 11. 时间估算

| 阶段 | 估算 |
|------|------|
| Phase 1 基础框架 | 1 周 |
| Phase 2 评分与 Runner | 1 周 |
| Phase 3 巡检模块 | 2–3 周 |
| Phase 4 节点采集 | 3–5 天 |
| Phase 5 报告生成 | 1–2 周 |
| Phase 6 CLI 与测试 | 1 周 |
| Phase 7 构建与发布 | 2–3 天 |
| **合计** | **约 6–8 周**（单人全职） |

---

## 12. 后期前端预留

Go 版完成后，可增加：

```
internal/
  server/           # HTTP API
    handler.go      # POST /inspect, GET /reports/:id, GET /reports/:id/download
    middleware.go   # 认证、日志
pkg/kubeowler/
  api.go            # 对外暴露 RunInspection(ctx, opts) -> (*ClusterReport, error)
```

`cmd/kubeowler/main.go` 调用 `pkg/kubeowler`；`cmd/server/main.go`（或单独服务）启动 HTTP 服务器并调用同一 API。前端通过 REST 触发巡检并下载报告。

---

## 13. 参考资料

- [client-go](https://github.com/kubernetes/client-go)
- [kubectl 源码](https://github.com/kubernetes/kubectl)（CLI 与 K8s 交互参考）
- [Cobra](https://github.com/spf13/cobra)
- 当前 Rust 实现：`src/` 目录
- 数据流说明：`docs/data-collection.md`
