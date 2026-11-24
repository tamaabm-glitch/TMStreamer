//! Native TM-LIC v1 envelope verification.
//!
//! The device stores only public verification material and signed licenses.
//! Private signing keys must remain on the issuer workstation.

use std::{fs, path::Path};

use base64::{engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD}, Engine as _};
use ed25519_dalek::{Signature, VerifyingKey};
use serde_json::Value;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

const PEM_PREFIX: &[u8] = &[0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedLicense {
    pub license_id: String,
    pub machine_id: String,
    pub tier: String,
    pub features: Vec<String>,
    pub expires_at: Option<String>,
}

fn field<'a>(value: &'a Value, key: &str) -> Result<&'a Value, String> {
    value.get(key).ok_or_else(|| format!("Field {key} tidak ditemukan"))
}

fn string_field(value: &Value, key: &str) -> Result<String, String> {
    field(value, key)?.as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("Field {key} tidak valid"))
}

fn read_public_key(path: &Path) -> Result<VerifyingKey, String> {
    let pem = fs::read_to_string(path).map_err(|e| format!("Public key tidak dapat dibaca: {e}"))?;
    let encoded: String = pem.lines()
        .filter(|line| !line.starts_with("-----"))
        .flat_map(|line| line.chars())
        .filter(|c| !c.is_whitespace())
        .collect();
    let der = STANDARD.decode(encoded).map_err(|_| "Format PEM public key tidak valid".to_string())?;
    if der.len() != PEM_PREFIX.len() + 32 || !der.starts_with(PEM_PREFIX) {
        return Err("Public key bukan Ed25519 SubjectPublicKeyInfo".into());
    }
    let raw: [u8; 32] = der[PEM_PREFIX.len()..].try_into()
        .map_err(|_| "Panjang public key Ed25519 tidak valid".to_string())?;
    VerifyingKey::from_bytes(&raw).map_err(|_| "Public key Ed25519 tidak valid".into())
}

/// Verify a TM-LIC envelope against the active key registry and expected device ID.
/// serde_json's default Map is key-sorted (preserve_order is not enabled), matching
/// Python json.dumps(sort_keys=True, separators=(",", ":"), ensure_ascii=False).
pub fn verify_envelope(
    envelope: &Value,
    expected_machine_id: &str,
    registry: &Value,
    public_key_dir: &Path,
) -> Result<VerifiedLicense, String> {
    if envelope.get("format").and_then(Value::as_str) != Some("TM-LIC") {
        return Err("Format lisensi tidak dikenal".into());
    }
    if envelope.get("format_version").and_then(Value::as_u64) != Some(1) {
        return Err("Versi format tidak didukung".into());
    }

    let key_id = string_field(envelope, "key_id")?;
    if key_id == "." || key_id == ".."
        || !key_id.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        return Err("Key ID mengandung karakter yang tidak diizinkan".into());
    }
    let payload = field(envelope, "payload")?;
    if !payload.is_object() {
        return Err("Payload lisensi tidak valid".into());
    }
    let signature_text = string_field(envelope, "signature")?;

    if payload.get("product").and_then(Value::as_str) != Some("TMSTREAMER") {
        return Err("Produk lisensi tidak cocok".into());
    }
    if payload.get("schema_version").and_then(Value::as_u64) != Some(1) {
        return Err("Schema lisensi tidak didukung".into());
    }
    let machine_id = string_field(payload, "machine_id")?;
    if machine_id != expected_machine_id {
        return Err("Machine ID tidak cocok".into());
    }

    if registry.get("format").and_then(Value::as_str) != Some("TM-LIC-KEY-REGISTRY")
        || registry.get("version").and_then(Value::as_u64) != Some(1)
    {
        return Err("Format atau versi registry tidak didukung".into());
    }
    if registry.get("active_key_id").and_then(Value::as_str) != Some(key_id.as_str()) {
        return Err("Key ID lisensi bukan active key".into());
    }

    let key_entry = registry.get("keys")
        .and_then(|v| v.get(&key_id))
        .ok_or_else(|| "Key ID tidak terdaftar".to_string())?;
    if key_entry.get("algorithm").and_then(Value::as_str) != Some("Ed25519")
        || key_entry.get("status").and_then(Value::as_str) != Some("active")
    {
        return Err("Public key tidak aktif atau algoritma tidak didukung".into());
    }

    let public_key = read_public_key(&public_key_dir.join(format!("{key_id}.pem")))?;
    let signature_bytes = URL_SAFE_NO_PAD.decode(&signature_text)
        .or_else(|_| URL_SAFE.decode(&signature_text))
        .map_err(|_| "Encoding signature tidak valid".to_string())?;
    let signature = Signature::try_from(signature_bytes.as_slice())
        .map_err(|_| "Panjang signature tidak valid".to_string())?;
    let canonical_payload = serde_json::to_vec(payload)
        .map_err(|_| "Payload tidak dapat diserialisasi".to_string())?;
    public_key.verify_strict(&canonical_payload, &signature)
        .map_err(|_| "Tanda tangan digital tidak valid".to_string())?;

    let license_id = string_field(payload, "license_id")?;
    let tier = string_field(payload, "tier")?;
    if !matches!(tier.as_str(), "FREE" | "TRIAL" | "PRO" | "CUSTOM") {
        return Err("Tier lisensi tidak dikenal".into());
    }
    let features_value = field(payload, "features")?
        .as_array().ok_or_else(|| "Daftar features tidak valid".to_string())?;
    let mut features = Vec::with_capacity(features_value.len());
    for item in features_value {
        let feature = item.as_str().filter(|s| !s.trim().is_empty())
            .ok_or_else(|| "Nama feature tidak valid".to_string())?;
        if features.iter().any(|existing| existing == feature) {
            return Err("Daftar features berisi duplikasi".into());
        }
        features.push(feature.to_owned());
    }

    let issued_at = string_field(payload, "issued_at")?;
    let issued = OffsetDateTime::parse(&issued_at, &Rfc3339)
        .map_err(|_| "Waktu penerbitan atau zona waktunya tidak valid".to_string())?;
    let now = OffsetDateTime::now_utc();
    if issued > now {
        return Err("Waktu penerbitan berada di masa depan".into());
    }

    let expires_at = match payload.get("expires_at") {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) => {
            let expiry = OffsetDateTime::parse(value, &Rfc3339)
                .map_err(|_| "Format kedaluwarsa atau zona waktunya tidak valid".to_string())?;
            if now >= expiry {
                return Err("Lisensi sudah kedaluwarsa".into());
            }
            Some(value.clone())
        }
        _ => return Err("Format kedaluwarsa tidak valid".into()),
    };

    Ok(VerifiedLicense { license_id, machine_id, tier, features, expires_at })
}


#[derive(Debug, Clone, serde::Serialize)]
pub struct LicenseStatus {
    pub licensed: bool,
    pub machine_id: Option<String>,
    pub license_id: Option<String>,
    pub tier: Option<String>,
    pub features: Vec<String>,
    pub expires_at: Option<String>,
    pub message: Option<String>,
}

/// Persistent device-side licensing state. The state directory is inside
/// systemd's existing ystreamer StateDirectory, so it survives package upgrades.
#[derive(Clone)]
pub struct LicenseService {
    state_dir: std::path::PathBuf,
    public_key_dir: std::path::PathBuf,
    registry_path: std::path::PathBuf,
    machine_id: Result<String, String>,
    lock: std::sync::Arc<std::sync::Mutex<()>>,
}

impl LicenseService {
    pub fn load() -> Self {
        let state_dir = std::env::var_os("TM_LIC_STATE_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/var/lib/ystreamer/tm-lic"));
        let public_key_dir = std::env::var_os("TM_LIC_PUBLIC_KEY_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/usr/share/ystreamer/tm-lic/public_keys"));
        let registry_path = std::env::var_os("TM_LIC_REGISTRY")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/usr/share/ystreamer/tm-lic/registry.json"));
        let machine_id = load_or_create_machine_id(&state_dir);
        Self {
            state_dir,
            public_key_dir,
            registry_path,
            machine_id,
            lock: std::sync::Arc::new(std::sync::Mutex::new(())),
        }
    }

    pub fn machine_id(&self) -> Result<&str, String> {
        self.machine_id.as_deref().map_err(Clone::clone)
    }

    pub fn has_feature(&self, feature: &str) -> bool {
        let status = self.status();
        status.licensed && status.features.iter().any(|item| item == feature)
    }

    pub fn status(&self) -> LicenseStatus {
        let machine_id = match &self.machine_id {
            Ok(value) => value.clone(),
            Err(error) => return LicenseStatus {
                licensed: false,
                machine_id: None,
                license_id: None,
                tier: None,
                features: vec![],
                expires_at: None,
                message: Some(format!("Penyimpanan Machine ID bermasalah: {error}")),
            },
        };
        let license_path = self.state_dir.join("license.json");
        if !license_path.is_file() {
            return LicenseStatus {
                licensed: false,
                machine_id: Some(machine_id),
                license_id: None,
                tier: None,
                features: vec![],
                expires_at: None,
                message: None,
            };
        }
        let result = (|| -> Result<VerifiedLicense, String> {
            let bytes = fs::read(&license_path)
                .map_err(|e| format!("File lisensi tidak dapat dibaca: {e}"))?;
            if bytes.len() > 65_536 {
                return Err("File lisensi melebihi batas 64 KiB".into());
            }
            let envelope: Value = serde_json::from_slice(&bytes)
                .map_err(|_| "Format JSON lisensi tidak valid".to_string())?;
            let registry: Value = serde_json::from_slice(&fs::read(&self.registry_path)
                .map_err(|e| format!("Registry lisensi tidak tersedia: {e}"))?)
                .map_err(|_| "Format registry lisensi tidak valid".to_string())?;
            verify_envelope(&envelope, &machine_id, &registry, &self.public_key_dir)
        })();
        match result {
            Ok(license) => LicenseStatus {
                licensed: true,
                machine_id: Some(machine_id),
                license_id: Some(license.license_id),
                tier: Some(license.tier),
                features: license.features,
                expires_at: license.expires_at,
                message: None,
            },
            Err(error) => LicenseStatus {
                licensed: false,
                machine_id: Some(machine_id),
                license_id: None,
                tier: None,
                features: vec![],
                expires_at: None,
                message: Some(error),
            },
        }
    }

    pub fn activate(&self, bytes: &[u8]) -> Result<LicenseStatus, String> {
        if bytes.is_empty() || bytes.len() > 65_536 {
            return Err("Ukuran file lisensi harus 1 sampai 65536 byte".into());
        }
        let _guard = self.lock.lock().map_err(|_| "License lock poisoned".to_string())?;
        let machine_id = self.machine_id()?;
        let envelope: Value = serde_json::from_slice(bytes)
            .map_err(|_| "Format JSON lisensi tidak valid".to_string())?;
        let registry: Value = serde_json::from_slice(&fs::read(&self.registry_path)
            .map_err(|e| format!("Registry lisensi tidak tersedia: {e}"))?)
            .map_err(|_| "Format registry lisensi tidak valid".to_string())?;
        verify_envelope(&envelope, machine_id, &registry, &self.public_key_dir)?;

        fs::create_dir_all(&self.state_dir)
            .map_err(|e| format!("Direktori lisensi tidak dapat dibuat: {e}"))?;
        #[cfg(unix)]
        fs::set_permissions(&self.state_dir, std::os::unix::fs::PermissionsExt::from_mode(0o700))
            .map_err(|e| format!("Izin direktori lisensi gagal diatur: {e}"))?;

        let destination = self.state_dir.join("license.json");
        let temporary = self.state_dir.join(format!("license.{}.tmp", std::process::id()));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)
            .map_err(|e| format!("File sementara lisensi tidak dapat dibuat: {e}"))?;
        use std::io::Write;
        if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
            let _ = fs::remove_file(&temporary);
            return Err(format!("Lisensi gagal disimpan: {error}"));
        }
        drop(file);
        if let Err(error) = fs::rename(&temporary, &destination) {
            let _ = fs::remove_file(&temporary);
            return Err(format!("Lisensi gagal diaktifkan: {error}"));
        }
        Ok(self.status())
    }
}

fn load_or_create_machine_id(directory: &Path) -> Result<String, String> {
    let path = directory.join("machine_id");
    if path.exists() {
        let value = fs::read_to_string(&path)
            .map_err(|e| format!("Machine ID tidak dapat dibaca: {e}"))?;
        let value = value.trim().to_owned();
        if value.len() != 35 || !value.starts_with("TM-")
            || !value[3..].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("Machine ID tersimpan rusak; tidak akan dibuat ulang otomatis".into());
        }
        return Ok(value.to_ascii_uppercase());
    }

    fs::create_dir_all(directory)
        .map_err(|e| format!("Direktori Machine ID tidak dapat dibuat: {e}"))?;
    #[cfg(unix)]
    fs::set_permissions(directory, std::os::unix::fs::PermissionsExt::from_mode(0o700))
        .map_err(|e| format!("Izin direktori Machine ID gagal diatur: {e}"))?;

    let mut random = [0u8; 16];
    use std::io::Read;
    fs::File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(&mut random))
        .map_err(|e| format!("Sumber acak sistem tidak tersedia: {e}"))?;
    let value = format!("TM-{}", hex::encode_upper(random));
    let temporary = directory.join(format!("machine_id.{}.tmp", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temporary)
        .map_err(|e| format!("Machine ID sementara tidak dapat dibuat: {e}"))?;
    use std::io::Write;
    file.write_all(value.as_bytes())
        .and_then(|_| file.write_all(b"\\n"))
        .and_then(|_| file.sync_all())
        .map_err(|e| {
            let _ = fs::remove_file(&temporary);
            format!("Machine ID gagal disimpan: {e}")
        })?;
    drop(file);
    match fs::rename(&temporary, &path) {
        Ok(()) => Ok(value),
        Err(error) if path.exists() => {
            let _ = fs::remove_file(&temporary);
            let existing = fs::read_to_string(&path)
                .map_err(|e| format!("Machine ID hasil pembuatan paralel tidak dapat dibaca: {e}"))?;
            let existing = existing.trim().to_owned();
            if existing.len() == 35 && existing.starts_with("TM-")
                && existing[3..].bytes().all(|b| b.is_ascii_hexdigit())
            {
                Ok(existing.to_ascii_uppercase())
            } else {
                Err("Machine ID hasil pembuatan paralel tidak valid".into())
            }
        }
        Err(error) => {
            let _ = fs::remove_file(&temporary);
            Err(format!("Machine ID gagal dipasang: {error}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture() -> (Value, Value, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "tm-lic-test-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let payload = serde_json::json!({
            "schema_version": 1,
            "product": "TMSTREAMER",
            "license_id": "test-license",
            "machine_id": "TM-0123456789ABCDEF0123456789ABCDEF",
            "tier": "PRO",
            "issued_at": OffsetDateTime::now_utc().format(&Rfc3339).unwrap(),
            "expires_at": null,
            "features": ["recording", "streaming"]
        });
        let bytes = serde_json::to_vec(&payload).unwrap();
        let signature = URL_SAFE_NO_PAD.encode(signing.sign(&bytes).to_bytes());
        let envelope = serde_json::json!({
            "format": "TM-LIC",
            "format_version": 1,
            "key_id": "test-key",
            "payload": payload,
            "signature": signature
        });
        let registry = serde_json::json!({
            "format": "TM-LIC-KEY-REGISTRY",
            "version": 1,
            "active_key_id": "test-key",
            "keys": {"test-key": {"algorithm": "Ed25519", "status": "active"}}
        });
        let der = [PEM_PREFIX, signing.verifying_key().as_bytes()].concat();
        fs::write(dir.join("test-key.pem"), format!(
            "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----\n",
            STANDARD.encode(der)
        )).unwrap();
        (envelope, registry, dir)
    }

    #[test]
    fn accepts_valid_signed_license() {
        let (envelope, registry, dir) = fixture();
        let result = verify_envelope(&envelope,
            "TM-0123456789ABCDEF0123456789ABCDEF", &registry, &dir).unwrap();
        assert_eq!(result.tier, "PRO");
        assert_eq!(result.features, vec!["recording", "streaming"]);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_tampered_payload() {
        let (mut envelope, registry, dir) = fixture();
        envelope["payload"]["tier"] = Value::String("FREE".into());
        assert!(verify_envelope(&envelope,
            "TM-0123456789ABCDEF0123456789ABCDEF", &registry, &dir).is_err());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_inactive_key() {
        let (envelope, mut registry, dir) = fixture();
        registry["keys"]["test-key"]["status"] = Value::String("revoked".into());
        assert!(verify_envelope(&envelope,
            "TM-0123456789ABCDEF0123456789ABCDEF", &registry, &dir).is_err());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_wrong_device() {
        let (envelope, registry, dir) = fixture();
        assert!(verify_envelope(&envelope, "TM-OTHER", &registry, &dir).is_err());
        let _ = fs::remove_dir_all(dir);
    }
}
