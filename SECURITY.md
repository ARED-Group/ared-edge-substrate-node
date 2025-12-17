# Security Policy

## Supported Versions

Currently supported versions:

- 1.x.x: Supported
- Below 1.0: Not supported

## Reporting a Vulnerability

**DO NOT** create a public GitHub issue for security vulnerabilities.



Include:
1. Type of vulnerability
2. Affected source files
3. Steps to reproduce
4. Impact assessment

### Response Timeline
- Initial Response: Within 48 hours
- Resolution Target: Within 30 days

## Blockchain Security Considerations

### Key Management
- Node keys stored in K8s secrets
- Session keys rotated periodically
- Never expose private keys in logs or metrics

### Runtime Security
- All runtime upgrades require governance approval
- WASM blobs verified before deployment
- State migrations tested thoroughly

### Network Security
- P2P ports isolated via NetworkPolicies
- RPC endpoints require authentication
- Admin endpoints not exposed externally
