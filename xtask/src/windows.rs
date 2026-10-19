//! Signed Velopack metadata. The signed bytes retain architecture and localized notes.
use crate::{Result, ensure, safe_file, versions};
use base64::{Engine, engine::general_purpose::STANDARD};
use p256::{
    ecdsa::{
        Signature, SigningKey, VerifyingKey,
        signature::{Signer, Verifier},
    },
    pkcs8::{DecodePrivateKey, DecodePublicKey},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::Path};

#[derive(Serialize, Deserialize)]
struct Envelope {
    feed: String,
    signature: String,
}

fn validate(feed: &[u8], arch: &str, version: &str) -> Result<Value> {
    let value: Value = serde_json::from_slice(feed)?;
    ensure(
        value["Architecture"] == arch,
        "Signed Windows architecture mismatch",
    )?;
    let assets = value["Assets"].as_array().ok_or("Invalid Windows feed")?;
    ensure(assets.len() == 1, "Expected one signed Windows package")?;
    let asset = &assets[0];
    let channel = versions(&format!("v{version}"))?.channel;
    ensure(
        asset["PackageId"] == "SerialStudio" && asset["Version"] == version,
        "Signed Windows identity mismatch",
    )?;
    ensure(
        asset["Type"] == "Full" || asset["Type"] == 1,
        "Expected full Windows package",
    )?;
    ensure(
        asset["FileName"] == format!("SerialStudio-{version}-win-{arch}-{channel}-full.nupkg"),
        "Signed Windows filename mismatch",
    )?;
    ensure(
        asset["Size"].as_u64().is_some_and(|size| size > 0),
        "Invalid Windows package size",
    )?;
    ensure(
        asset["SHA256"]
            .as_str()
            .is_some_and(|hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit())),
        "Invalid Windows package SHA-256",
    )?;
    Ok(value)
}

fn seal(feed: &[u8], key: &SigningKey) -> Result<Vec<u8>> {
    let signature: Signature = key.sign(feed);
    Ok(serde_json::to_vec(&Envelope {
        feed: STANDARD.encode(feed),
        signature: STANDARD.encode(signature.to_bytes()),
    })?)
}

/// Sign both architectures using the release environment's PKCS#8 key.
pub fn sign(directory: &Path, tag: &str, public_key: &str) -> Result<()> {
    let version = versions(tag)?;
    let private_key = STANDARD.decode(std::env::var("WINDOWS_UPDATE_PRIVATE_KEY")?.trim())?;
    let signing = SigningKey::from_pkcs8_der(&private_key)?;
    sign_with(
        directory,
        &version.version,
        &version.channel,
        public_key,
        &signing,
    )
}

pub(crate) fn sign_with(
    directory: &Path,
    version: &str,
    channel: &str,
    public_key: &str,
    signing: &SigningKey,
) -> Result<()> {
    let verifying = VerifyingKey::from_public_key_der(&STANDARD.decode(public_key.trim())?)?;
    ensure(
        signing.verifying_key() == &verifying,
        "Windows update private key does not match the shipped public key",
    )?;
    // Validate both architectures before writing either envelope.
    let mut envelopes = Vec::new();
    for arch in ["x64", "arm64"] {
        let name = format!("releases.win-{arch}-{channel}");
        let mut feed: Value =
            serde_json::from_slice(&fs::read(safe_file(directory, &format!("{name}.json"))?)?)?;
        feed["Architecture"] = arch.into();
        let bytes = serde_json::to_vec(&feed)?;
        validate(&bytes, arch, version)?;
        verify_payload(directory, &feed)?;
        envelopes.push((format!("{name}.signed.json"), seal(&bytes, signing)?));
    }
    for (name, bytes) in envelopes {
        fs::write(directory.join(name), bytes)?;
    }
    Ok(())
}

/// Verify the envelope and require the same assets as the legacy feed.
pub fn verify(
    directory: &Path,
    arch: &str,
    version: &str,
    channel: &str,
    public_key: &str,
) -> Result<()> {
    let name = format!("releases.win-{arch}-{channel}");
    let envelope: Envelope = serde_json::from_slice(&fs::read(safe_file(
        directory,
        &format!("{name}.signed.json"),
    )?)?)?;
    let feed = STANDARD.decode(envelope.feed)?;
    let signature = Signature::from_slice(&STANDARD.decode(envelope.signature)?)?;
    let key = VerifyingKey::from_public_key_der(&STANDARD.decode(public_key.trim())?)?;
    key.verify(&feed, &signature)
        .map_err(|_| "Invalid Windows update signature")?;
    let signed = validate(&feed, arch, version)?;
    verify_payload(directory, &signed)?;
    let plain: Value =
        serde_json::from_slice(&fs::read(safe_file(directory, &format!("{name}.json"))?)?)?;
    ensure(
        signed["Assets"] == plain["Assets"],
        "Signed and legacy Windows feeds differ",
    )
}

fn verify_payload(directory: &Path, value: &Value) -> Result<()> {
    use sha2::{Digest, Sha256};
    let asset = &value["Assets"][0];
    let bytes = fs::read(safe_file(
        directory,
        asset["FileName"]
            .as_str()
            .ok_or("Missing Windows package filename")?,
    )?)?;
    ensure(
        asset["Size"].as_u64() == Some(bytes.len() as u64),
        "Windows package size mismatch",
    )?;
    let checksum = format!("{:x}", Sha256::digest(&bytes));
    ensure(
        asset["SHA256"]
            .as_str()
            .is_some_and(|hash| hash.eq_ignore_ascii_case(&checksum)),
        "Windows package SHA-256 mismatch",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use p256::pkcs8::EncodePublicKey;
    use sha2::{Digest, Sha256};

    #[test]
    fn signed_windows_feed_rejects_tampering_and_wrong_architecture() {
        let directory = tempfile::tempdir().unwrap();
        let key = SigningKey::from_bytes((&[23; 32]).into()).unwrap();
        let public = STANDARD.encode(key.verifying_key().to_public_key_der().unwrap().as_bytes());
        for arch in ["x64", "arm64"] {
            let filename = format!("SerialStudio-1.0.0-win-{arch}-stable-full.nupkg");
            fs::write(directory.path().join(&filename), b"package").unwrap();
            let feed = serde_json::json!({"Assets":[{"PackageId":"SerialStudio","Version":"1.0.0","Type":"Full","FileName":filename,"Size":7,"SHA256":format!("{:x}", Sha256::digest(b"package")),"NotesMarkdown":"notes"}]});
            fs::write(
                directory
                    .path()
                    .join(format!("releases.win-{arch}-stable.json")),
                serde_json::to_vec(&feed).unwrap(),
            )
            .unwrap();
        }
        sign_with(directory.path(), "1.0.0", "stable", &public, &key).unwrap();
        verify(directory.path(), "x64", "1.0.0", "stable", &public).unwrap();
        fs::copy(
            directory.path().join("releases.win-x64-stable.signed.json"),
            directory
                .path()
                .join("releases.win-arm64-stable.signed.json"),
        )
        .unwrap();
        assert!(verify(directory.path(), "arm64", "1.0.0", "stable", &public).is_err());
        fs::write(
            directory
                .path()
                .join("SerialStudio-1.0.0-win-x64-stable-full.nupkg"),
            b"corrupt",
        )
        .unwrap();
        assert!(verify(directory.path(), "x64", "1.0.0", "stable", &public).is_err());
        let signed_path = directory.path().join("releases.win-x64-stable.signed.json");
        let mut envelope: Envelope =
            serde_json::from_slice(&fs::read(&signed_path).unwrap()).unwrap();
        let mut bytes = STANDARD.decode(&envelope.feed).unwrap();
        bytes[0] ^= 1;
        envelope.feed = STANDARD.encode(bytes);
        fs::write(signed_path, serde_json::to_vec(&envelope).unwrap()).unwrap();
        assert!(verify(directory.path(), "x64", "1.0.0", "stable", &public).is_err());
    }
}
