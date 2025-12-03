# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

**DO NOT** create a public GitHub issue for security vulnerabilities.

Report security vulnerabilities by emailing: security@a-r-e-d.com

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
