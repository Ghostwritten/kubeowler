# CERT-003 Certificate expired

## Summary

A TLS certificate stored in a Kubernetes Secret (e.g. type kubernetes.io/tls) has already expired. Services using this certificate will fail or be insecure. Renew immediately and update the Secret.

## Severity

Critical

## Example

N/A

## Symptoms

- Report shows certificate with negative "days until expiry" or expired status
- TLS handshakes or ingress using the Secret fail

## Resolution

1. Identify the Secret and namespace from the inspection report
2. Renew the certificate using your PKI or issuer (e.g. cert-manager, internal CA, public CA)
3. Update the Secret with the new tls.crt (and key if rotated); restart workloads that use the Secret
4. For control-plane certs (apiserver, etcd, kubelet), use `kubeadm cert renew` or equivalent on the nodes

## References

- [TLS Secrets](https://kubernetes.io/docs/concepts/configuration/secret/#tls-secrets)
- [cert-manager](https://cert-manager.io/docs/)
- [kubeadm cert](https://kubernetes.io/docs/reference/setup-tools/kubeadm/kubeadm-cert/)
