# CERT-001 CSR long Pending or abnormal

## Summary

CertificateSigningRequest (CSR) is the Kubernetes resource for requesting x509 certificates. Components like kubelet create CSRs to obtain certificates for apiserver access or TLS. This check reports unhealthy CSR state: long-standing Pending or Denied/Failed requests.


## Severity

Warning

## Example

N/A

## Symptoms

- CSRs in **Pending** state that have not been approved or denied
- CSRs in **Denied** or **Failed** state indicating rejection or issuance failure

## Resolution

1. **Pending CSR**: Use `kubectl get csr` to list; use `kubectl certificate approve <name>` or `kubectl certificate deny <name>` to act. For kubelet certificate requests when nodes join, approve as appropriate.
2. **Denied/Failed CSR**: Clean up or re-issue per cause; check cluster CA and signer configuration if needed.
3. **Control-plane certificate expiry**: Expiry of etcd, kube-apiserver, and kubelet server certificates is **not** exposed via this API. Run `kubeadm cert check-expiry` (or equivalent) on the relevant nodes to check and renew.

## References

- [Certificate Signing Requests](https://kubernetes.io/docs/reference/access-authn-authz/certificate-signing-requests/)
- [kubeadm cert check-expiry](https://kubernetes.io/docs/reference/setup-tools/kubeadm/kubeadm-cert/)
