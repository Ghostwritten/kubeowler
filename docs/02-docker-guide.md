# Docker Build and Run Guide

This guide covers building and running the **Kubeowler main binary** as a Docker image. Supported architectures and base environment align with the [main README](../README.md): **architectures** `linux/amd64` and `linux/arm64`; **base OS** glibc-compatible Linux. For multi-arch builds, see the multi-arch section in [01-installation-guide.md](01-installation-guide.md) or use Docker buildx.

---

## Building the Docker Image

```bash
docker build -t kubeowler:latest .
docker images | grep kubeowler
```

---

## Running the Container

**With kubeconfig mounted:**

```bash
docker run --rm \
  -v ~/.kube/config:/home/kubeowler/.kube/config:ro \
  -v $(pwd)/reports:/app/reports \
  kubeowler:latest check -o /app/reports/cluster-report.md
```

**Inside a Kubernetes cluster (using ServiceAccount):**

```bash
docker run --rm \
  -v $(pwd)/reports:/app/reports \
  kubeowler:latest check -o /app/reports/cluster-report.md
```

---

## Kubernetes Deployment Example

The following example defines a ServiceAccount, ClusterRole, ClusterRoleBinding, and a CronJob that runs Kubeowler daily.

```yaml
# kubeowler-cronjob.yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: kubeowler
  namespace: default
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: kubeowler-reader
rules:
- apiGroups: [""]
  resources: ["nodes", "pods", "services", "namespaces", "persistentvolumes", "persistentvolumeclaims"]
  verbs: ["get", "list"]
- apiGroups: ["apps"]
  resources: ["deployments", "replicasets", "daemonsets", "statefulsets"]
  verbs: ["get", "list"]
- apiGroups: ["rbac.authorization.k8s.io"]
  resources: ["roles", "rolebindings", "clusterroles", "clusterrolebindings"]
  verbs: ["get", "list"]
- apiGroups: ["networking.k8s.io"]
  resources: ["networkpolicies"]
  verbs: ["get", "list"]
- apiGroups: ["storage.k8s.io"]
  resources: ["storageclasses"]
  verbs: ["get", "list"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: kubeowler-binding
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: kubeowler-reader
subjects:
- kind: ServiceAccount
  name: kubeowler
  namespace: default
---
apiVersion: batch/v1
kind: CronJob
metadata:
  name: kubeowler-inspection
  namespace: default
spec:
  schedule: "0 9 * * *"   # 09:00 daily
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: kubeowler
          containers:
          - name: kubeowler
            image: ghostwritten/kubeowler:v0.1.0
            command: ["kubeowler"]
            args: ["check", "-o", "/tmp/cluster-report.md"]
            volumeMounts:
            - name: reports
              mountPath: /tmp
          volumes:
          - name: reports
            emptyDir: {}
          restartPolicy: OnFailure
```

Apply to the cluster:

```bash
kubectl apply -f kubeowler-cronjob.yaml
```

Adjust the image name, schedule, and output path as needed. For node-level inspection, deploy the Node Inspector DaemonSet as described in [06-node-inspector-build-deploy.md](06-node-inspector-build-deploy.md).
