TM-LIC public verification keys

This directory is intentionally shipped without a production public key.

Before issuing production licenses:
1. Generate a new Ed25519 keypair on the trusted issuer workstation.
2. Keep the private key only on that workstation, protected by a strong passphrase and backup.
3. Replace ../registry.json with the matching active key ID and an Ed25519 active entry.
4. Add the matching <key-id>.pem public key file to this directory.
5. Rebuild and verify the Debian package before installing it.

Never copy private key material, encrypted private-key files, issuer databases, sample licenses,
or signing tools into the device package.
