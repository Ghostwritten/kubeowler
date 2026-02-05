# Issue Code Index

Inspection reports classify findings by issue type and assign stable codes for linking to the documentation in this directory. Each code corresponds to a Markdown document with: Summary, Severity (Info / Warning / Critical), Symptoms, Resolution, Example, References.

## By category

### NODE
| Code | Short Title |
|------|-------------|
| [NODE-001](NODE-001.md) | Node not ready |
| [NODE-002](NODE-002.md) | Node has resource pressure |
| [NODE-003](NODE-003.md) | Zombie processes on node |
| [NODE-004](NODE-004.md) | Node disk usage high (Warning) |
| [NODE-005](NODE-005.md) | Node disk usage critical |

### POD
| Code | Short Title |
|------|-------------|
| [POD-001](POD-001.md) | Pod in Failed state |
| [POD-002](POD-002.md) | Pod cannot be scheduled |
| [POD-003](POD-003.md) | Container restart count too high |
| [POD-004](POD-004.md) | Container in abnormal state |
| [POD-005](POD-005.md) | ImagePullBackOff |
| [POD-006](POD-006.md) | ErrImagePull |
| [POD-007](POD-007.md) | CrashLoopBackOff |
| [POD-008](POD-008.md) | ContainerCreating |
| [POD-009](POD-009.md) | CreateContainerConfigError |
| [POD-010](POD-010.md) | OOMKilled |
| [POD-011](POD-011.md) | Container terminated (non-zero exit) |
| [POD-012](POD-012.md) | Pod Running but not Ready |

### RES
| Code | Short Title |
|------|-------------|
| [RES-001](RES-001.md) | Container has no resource requests |
| [RES-002](RES-002.md) | Container has no resource limits |
| [RES-003](RES-003.md) | Namespace has no resource quota |
| [RES-004](RES-004.md) | CPU limit below request |
| [RES-005](RES-005.md) | Memory limit below request |

### NET
| Code | Short Title |
|------|-------------|
| [NET-001](NET-001.md) | LoadBalancer has no external IP |
| [NET-002](NET-002.md) | NodePort outside recommended range |
| [NET-003](NET-003.md) | Service has no selector or endpoints |
| [NET-004](NET-004.md) | DNS deployment not ready |
| [NET-005](NET-005.md) | DNS service not found |

### STO
| Code | Short Title |
|------|-------------|
| [STO-001](STO-001.md) | PV config or backing storage issue |
| [STO-002](STO-002.md) | PV Released, needs cleanup |
| [STO-003](STO-003.md) | PV Retained, manual action needed |
| [STO-004](STO-004.md) | PV has no reclaim policy |
| [STO-005](STO-005.md) | PVC storage class or capacity issue |
| [STO-006](STO-006.md) | PVC has data loss risk |
| [STO-007](STO-007.md) | PVC has no storage class |
| [STO-008](STO-008.md) | StorageClass has no provisioner |
| [STO-009](STO-009.md) | No default StorageClass |
| [STO-010](STO-010.md) | Multiple StorageClasses marked default |

### SEC
| Code | Short Title |
|------|-------------|
| [SEC-001](SEC-001.md) | ClusterRole has excessive permissions |
| [SEC-002](SEC-002.md) | User has cluster-admin |
| [SEC-003](SEC-003.md) | ServiceAccount has cluster-admin |
| [SEC-004](SEC-004.md) | Pod runs as root |
| [SEC-005](SEC-005.md) | Container runs privileged |
| [SEC-006](SEC-006.md) | Container runs as root |
| [SEC-007](SEC-007.md) | Container allows privilege escalation |
| [SEC-008](SEC-008.md) | Insufficient network policy coverage |
| [SEC-009](SEC-009.md) | Uses default ServiceAccount |

### CTRL
| Code | Short Title |
|------|-------------|
| [CTRL-001](CTRL-001.md) | Control plane component not ready |
| [CTRL-002](CTRL-002.md) | Static Pod not ready |

### AUTO
| Code | Short Title |
|------|-------------|
| [AUTO-001](AUTO-001.md) | HPA replica range too narrow |
| [AUTO-002](AUTO-002.md) | HPA has no metrics configured |
| [AUTO-003](AUTO-003.md) | HPA target workload or metrics issue |
| [AUTO-004](AUTO-004.md) | HPA behavior limits scaling |
| [AUTO-005](AUTO-005.md) | HPA metric target not configured |

### BATCH
| Code | Short Title |
|------|-------------|
| [BATCH-001](BATCH-001.md) | CronJob suspended |
| [BATCH-002](BATCH-002.md) | CronJob job failed |
| [BATCH-003](BATCH-003.md) | CronJob schedule or controller issue |
| [BATCH-004](BATCH-004.md) | Job needs backoffLimit or resource check |
| [BATCH-005](BATCH-005.md) | Job Pod stuck or timeout adjustment needed |

### POLICY
| Code | Short Title |
|------|-------------|
| [POLICY-001](POLICY-001.md) | No ResourceQuota configured |
| [POLICY-002](POLICY-002.md) | No LimitRange configured |
| [POLICY-003](POLICY-003.md) | Critical workload has no PDB |
| [POLICY-004](POLICY-004.md) | Replica count does not satisfy PDB |

### OBS
| Code | Short Title |
|------|-------------|
| [OBS-001](OBS-001.md) | metrics-server not deployed |
| [OBS-002](OBS-002.md) | kube-state-metrics not deployed |
| [OBS-003](OBS-003.md) | Log aggregation not deployed |
| [OBS-004](OBS-004.md) | Prometheus/monitoring not deployed |

### CERT
| Code | Short Title |
|------|-------------|
| [CERT-001](CERT-001.md) | CSR long Pending or abnormal |
| [CERT-002](CERT-002.md) | Certificate expiring soon |
| [CERT-003](CERT-003.md) | Certificate expired |

Report Code links point to the corresponding document in this directory. Documents are shipped with the repository.
