# CERT-002 Certificate expiring soon

## Summary

A TLS certificate stored in a Kubernetes Secret (e.g. type kubernetes.io/tls) expires within a configured threshold (e.g. 30 or 90 days). Renew before expiry to avoid service disruption.


## Severity

Warning

## Example

N/A

## Symptoms

- Report shows certificate expiring within N days or lists certificate expiry in the Certificates section
- Certificate notAfter date is within the warning window

## Resolution

1. Identify the Secret and namespace; note the current expiry from the inspection report
2. Renew the certificate using your PKI or issuer (e.g. cert-manager, internal CA, public CA)
3. Update the Secret with the new tls.crt (and key if rotated); restart workloads that use the Secret
4. For control-plane certs (apiserver, etcd, kubelet), use `kubeadm cert renew` or equivalent on the nodes

## References

- [TLS Secrets](https://kubernetes.io/docs/concepts/configuration/secret/#tls-secrets)
- [cert-manager](https://cert-manager.io/docs/)
- [kubeadm cert](https://kubernetes.io/docs/reference/setup-tools/kubeadm/kubeadm-cert/)
