# Security & Argument Sanitization

## Principle

> *"Never log passwords, tokens, private keys, or unfiltered sensitive command arguments."*

Monitoring process activity on a Linux machine invariably encounters commands that contain sensitive secrets (e.g., automated CI scripts, developer tools, or installer commands). Persisting raw command strings could create credential leaks.

## Redaction Strategy

InstallMon employs multi-layer sanitization at event ingest time:

1. **Option Flag Value Redaction**:
   - Flags including `--password`, `-p`, `--token`, `--api-key`, `--auth`, `--secret`, `--access-key`, `--private-key`, and `-u` / `--user` cause following values to be replaced with `[REDACTED]`.
2. **Inline Flag Redaction**:
   - Arguments formatted as `--password=SECRET` or `--token=SECRET` are rewritten to `--password=[REDACTED]`.
3. **URL Credentials**:
   - Embedded authentication URLs (`https://user:password@host/repo.git`) are transformed to `https://user:[REDACTED]@host/repo.git`.
4. **HTTP Authorization Headers**:
   - Headers like `Authorization: Bearer <token>` or `Authorization: Basic <base64>` are scrubbed to `Authorization: Bearer [REDACTED]`.
5. **Private Key Blocks**:
   - Blocks starting with `-----BEGIN ... PRIVATE KEY-----` are replaced with `[REDACTED_PRIVATE_KEY]`.

Because sanitization preserves command names, flags, and targets while scrubbing secret values, auditability and process correlation remain intact without compromising security.
