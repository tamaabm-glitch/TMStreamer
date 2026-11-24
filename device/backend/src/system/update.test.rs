use super::*;

use ed25519_dalek::{Signer, SigningKey};

const MESSAGE: &[u8] = b"TMStreamer update signature test";

/// A deterministic throwaway key used only by unit tests. Never a release key.
fn test_signing_key() -> SigningKey {
    SigningKey::from_bytes(&[0x42; 32])
}

fn test_public_key() -> String {
    base64::engine::general_purpose::STANDARD.encode(test_signing_key().verifying_key().to_bytes())
}

fn test_signature(message: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(test_signing_key().sign(message).to_bytes())
}

#[test]
fn test_signing_key_accepts_matching_signature() {
    let public_key = test_public_key();
    let signature = test_signature(MESSAGE);
    assert_eq!(verify(&public_key, MESSAGE, &signature), Ok(()));
    assert_eq!(
        verify(&public_key, MESSAGE, &format!("{signature}\n")),
        Ok(())
    );
}

#[test]
fn changed_package_is_refused() {
    let public_key = test_public_key();
    let signature = test_signature(MESSAGE);
    assert!(verify(&public_key, b"TMStreamer update signature tesT", &signature).is_err());
}

#[test]
fn signature_from_another_key_is_refused() {
    let other = SigningKey::from_bytes(&[0x24; 32]);
    let other_signature =
        base64::engine::general_purpose::STANDARD.encode(other.sign(MESSAGE).to_bytes());
    assert!(verify(&test_public_key(), MESSAGE, &other_signature).is_err());
}

#[test]
fn junk_signature_is_refused() {
    let public_key = test_public_key();
    assert!(verify(&public_key, MESSAGE, "").is_err());
    assert!(verify(&public_key, MESSAGE, "not base64!").is_err());
    assert!(verify(&public_key, MESSAGE, "c2hvcnQ=").is_err());
}

#[test]
fn malformed_public_key_is_rejected_without_panicking() {
    let signature = test_signature(MESSAGE);
    assert!(verify("not base64!", MESSAGE, &signature).is_err());
    assert!(verify("c2hvcnQ=", MESSAGE, &signature).is_err());
}

#[test]
fn packaging_revision_is_dropped_from_the_version() {
    assert_eq!(display_version("0.1.0-1"), "0.1.0");
    assert_eq!(display_version("0.2.0"), "0.2.0");
}

#[test]
fn only_a_higher_version_is_newer() {
    assert!(is_newer("0.2.0", "0.1.0"));
    assert!(is_newer("0.10.0", "0.9.5"));
    assert!(is_newer("1.0.0.1", "1.0.0"));
    assert!(!is_newer("0.1.0", "0.1.0"));
    assert!(!is_newer("0.1.0", "0.2.0"));
    assert!(!is_newer("latest", "0.1.0"));
    assert!(!is_newer("", "0.1.0"));
}

#[test]
fn release_manifest_and_debian_package_versions_must_match_and_advance() {
    assert_eq!(
        validate_release_version("0.1.3-1", Some("0.1.3"), "0.1.2"),
        Ok(())
    );
    assert!(validate_release_version("0.1.3-1", Some("0.1.4"), "0.1.2").is_err());
    assert!(validate_release_version("0.1.1-1", Some("0.1.1"), "0.1.2").is_err());
    assert!(validate_release_version("garbled-1", Some("garbled"), "0.1.2").is_err());
}

#[test]
fn manifest_reads_without_notes() {
    let manifest: Manifest = serde_json::from_str(
        r#"{"version": "0.2.0", "packages": {"arm64": {"url": "https://example.com/a.deb", "signature": "c2ln"}}}"#,
    )
    .unwrap();
    assert_eq!(manifest.version, "0.2.0");
    assert_eq!(manifest.notes, "");
    assert_eq!(manifest.packages["arm64"].url, "https://example.com/a.deb");
}

/// The install script in a scratch folder, with apt-get, curl and sleep
/// replaced: `apt` is the exit code of each apt-get call in turn, `health`
/// what the device answers
fn run_script(name: &str, apt: &[u8], health: &str, before: &[(&str, &str)]) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let dir = std::env::temp_dir().join(format!("ystreamer-update-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let bin = dir.join("bin");
    fs::create_dir_all(&bin).unwrap();
    for (file, contents) in before {
        fs::write(dir.join(file), contents).unwrap();
    }
    let codes: Vec<String> = apt.iter().map(u8::to_string).collect();
    fs::write(dir.join("apt-codes"), codes.join("\n") + "\n").unwrap();
    fs::write(dir.join("health"), health).unwrap();
    let stubs = [
        (
            "apt-get",
            "echo \"$@\" >>apt-calls\ncode=$(head -1 apt-codes); sed -i 1d apt-codes; exit ${code:-0}",
        ),
        ("curl", "cat health"),
        ("sleep", "true"),
    ];
    for (stub, body) in stubs {
        let path = bin.join(stub);
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let keep =
        format!("[ ! -f {CURRENT} ] || mv -f {CURRENT} {PREVIOUS}; mv -f {STAGED} {CURRENT}");
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(script(&dir.to_string_lossy(), STAGED, "0.2.0", 80, &keep))
        .current_dir(&dir)
        .env(
            "PATH",
            format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()),
        )
        .status()
        .unwrap();
    assert!(status.success());
    dir
}

fn read(dir: &Path, name: &str) -> String {
    fs::read_to_string(dir.join(name))
        .unwrap_or_default()
        .trim()
        .to_string()
}

const RUNNING: &[(&str, &str)] = &[(STAGED, "new"), (CURRENT, "old")];

#[test]
fn version_that_answers_is_kept() {
    let dir = run_script("ok", &[0], r#"{"status":"ok","version":"0.2.0"}"#, RUNNING);
    assert_eq!(read(&dir, RESULT), "0");
    assert_eq!(read(&dir, CURRENT), "new");
    assert_eq!(read(&dir, PREVIOUS), "old");
    assert!(!dir.join(STAGED).exists());
    assert_eq!(read(&dir, "apt-calls").lines().count(), 1);
}

#[test]
fn version_that_never_answers_is_replaced_by_the_one_before() {
    let dir = run_script("silent", &[0, 0], "", RUNNING);
    assert_eq!(read(&dir, RESULT), REVERTED);
    assert_eq!(read(&dir, CURRENT), "old");
    assert_eq!(read(&dir, STAGED), "new");
    assert!(!dir.join(PREVIOUS).exists());
    let calls = read(&dir, "apt-calls");
    assert!(calls.lines().nth(1).unwrap().ends_with(CURRENT), "{calls}");
}

#[test]
fn old_version_still_answering_is_not_taken_for_the_new_one() {
    let dir = run_script(
        "stale",
        &[0, 0],
        r#"{"status":"ok","version":"0.1.0"}"#,
        RUNNING,
    );
    assert_eq!(read(&dir, RESULT), REVERTED);
}

#[test]
fn nothing_to_go_back_to_is_reported() {
    let dir = run_script("first", &[0], "", &[(STAGED, "new")]);
    assert_eq!(read(&dir, RESULT), UNANSWERED);
    assert_eq!(read(&dir, "apt-calls").lines().count(), 1);
}

#[test]
fn failed_install_changes_nothing() {
    let dir = run_script("failed", &[100], "", RUNNING);
    assert_eq!(read(&dir, RESULT), "100");
    assert_eq!(read(&dir, CURRENT), "old");
    assert_eq!(read(&dir, STAGED), "new");
    assert_eq!(read(&dir, "apt-calls").lines().count(), 1);
}

#[test]
fn production_release_updater_stays_closed_without_its_own_key() {
    assert!(update_public_key().is_err());
}
