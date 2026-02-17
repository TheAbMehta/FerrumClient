# Security Policy

## Supported Versions

Currently, Ferrum is in early development (v0.1.0). Security updates will be provided for the latest version only.

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Ferrum, please report it responsibly:

### What to Report

- Security vulnerabilities in the client code
- Potential exploits in the networking layer
- Asset loading vulnerabilities
- Subprocess management security issues
- Any other security concerns

### How to Report

1. **Do NOT** open a public GitHub issue for security vulnerabilities
2. Use [GitHub Security Advisories](https://github.com/TheAbMehta/FerrumClient/security/advisories/new) to report privately
3. Provide detailed information about the vulnerability:
   - Description of the issue
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- Acknowledgment of your report within 48 hours
- Regular updates on the progress of fixing the vulnerability
- Credit for responsible disclosure (if desired)

## Security Considerations

### Asset Loading

Ferrum loads Minecraft assets from multiple sources. Users should:

- Only use assets from trusted sources (official Mojang servers, legitimate Minecraft installations)
- Be cautious when using third-party asset mirrors
- Verify asset integrity when possible

### Subprocess Management

Ferrum spawns Pumpkin-MC as a subprocess. Security considerations:

- Pumpkin binary should be from a trusted source
- Process isolation is enforced via process groups
- Subprocess communication is limited to stdin/stdout

### Network Security

- Ferrum connects to localhost by default (Pumpkin subprocess)
- No remote server connections in current version
- Future versions will implement proper authentication

## Known Security Limitations

- No sandboxing of Pumpkin subprocess (runs with same privileges as client)
- No asset signature verification
- No encrypted communication (localhost only currently)

## Future Security Enhancements

- Asset signature verification
- Subprocess sandboxing
- Encrypted network communication
- Authentication system for remote servers
