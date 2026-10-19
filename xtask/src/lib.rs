use semver::Version;
use std::{
    fs,
    path::{Path, PathBuf},
};
pub mod notes;
pub mod windows;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) fn ensure(value: bool, message: &str) -> Result<()> {
    if value { Ok(()) } else { Err(message.into()) }
}
pub struct Versions {
    pub version: String,
    pub channel: String,
}
pub fn versions(tag: &str) -> Result<Versions> {
    let value = tag
        .strip_prefix('v')
        .ok_or("Release tags must begin with v")?;
    let parsed = Version::parse(value)?;
    ensure(
        parsed.build.is_empty(),
        "Build metadata is not supported in release tags",
    )?;
    let channel = if parsed.pre.is_empty() {
        "stable"
    } else {
        let parts = parsed.pre.as_str().split('.').collect::<Vec<_>>();
        ensure(
            parts.len() == 2
                && matches!(parts[0], "alpha" | "beta")
                && parts[1].parse::<u32>().is_ok(),
            "Use alpha.N or beta.N prereleases",
        )?;
        parts[0]
    };
    Ok(Versions {
        version: parsed.to_string(),
        channel: channel.into(),
    })
}
pub(crate) fn safe_file(directory: &Path, name: &str) -> Result<PathBuf> {
    ensure(
        !name.is_empty() && !name.contains(['/', '\\', ':']) && !name.chars().any(char::is_control),
        "Invalid asset filename",
    )?;
    let path = directory.join(name);
    ensure(
        fs::symlink_metadata(&path)?.file_type().is_file(),
        "Release assets must be regular files",
    )?;
    Ok(path)
}

/// Generate localized appcasts and architecture-scoped, signed Windows feeds.
/// Existing channel metadata prevents semantic downgrades during promotion.
pub fn feeds(directory: &Path, output: &Path, tag: &str, repository: &str) -> Result<()> {
    feeds_with_keys(
        directory,
        output,
        tag,
        repository,
        &std::env::var("SPARKLE_PUBLIC_ED_KEY")?,
        &std::env::var("WINDOWS_UPDATE_PUBLIC_KEY")?,
    )
}

fn feeds_with_keys(
    directory: &Path,
    output: &Path,
    tag: &str,
    repository: &str,
    sparkle_key: &str,
    windows_key: &str,
) -> Result<()> {
    let release = versions(tag)?;
    let version = Version::parse(&release.version)?;
    ensure(
        repository == "tangxiangong/serial-port",
        "Unexpected release repository",
    )?;
    let url = format!("https://github.com/{repository}/releases/download/{tag}");
    // Verify every artifact even if an older release will not advance any channel.
    let mut appcasts = Vec::new();
    for arch in ["arm64", "x86_64"] {
        let source = fs::read_to_string(safe_file(directory, &format!("appcast-{arch}.xml"))?)?;
        verify_macos(
            directory,
            &source,
            &release.version,
            arch,
            &url,
            sparkle_key,
        )?;
        ensure(
            fs::metadata(safe_file(
                directory,
                &format!("SerialStudio-{}-macos-{arch}.dmg", release.version),
            )?)?
            .len()
                > 0,
            "Empty macOS installer",
        )?;
        appcasts.push((arch, source));
    }
    for locale in ["zh-Hans", "en"] {
        ensure(
            !fs::read_to_string(safe_file(directory, &format!("notes-macos-{locale}.md"))?)?
                .trim()
                .is_empty(),
            "Missing localized macOS notes",
        )?;
    }
    let mut windows_feeds = Vec::new();
    for arch in ["x64", "arm64"] {
        windows::verify(
            directory,
            arch,
            &release.version,
            &release.channel,
            windows_key,
        )?;
        ensure(
            fs::metadata(safe_file(
                directory,
                &format!("SerialStudio-{}-windows-{arch}.exe", release.version),
            )?)?
            .len()
                > 0,
            "Empty Windows installer",
        )?;
        for extension in ["json", "signed.json"] {
            windows_feeds.push((
                arch,
                extension,
                fs::read(safe_file(
                    directory,
                    &format!("releases.win-{arch}-{}.{extension}", release.channel),
                )?)?,
            ));
        }
    }
    let channels_file = output.join("channels.json");
    let mut channels: std::collections::BTreeMap<String, String> = if channels_file.exists() {
        serde_json::from_slice(&fs::read(&channels_file)?)?
    } else {
        Default::default()
    };
    for (channel, recorded) in &channels {
        ensure(
            matches!(channel.as_str(), "stable" | "beta" | "alpha"),
            "Unknown channel history",
        )?;
        Version::parse(recorded)?;
    }
    let audiences: &[&str] = match release.channel.as_str() {
        "stable" => &["stable", "beta", "alpha"],
        "beta" => &["beta", "alpha"],
        _ => &["alpha"],
    };
    let mut staged = Vec::<(String, Vec<u8>)>::new();
    for audience in audiences {
        if let Some(previous) = channels.get(*audience)
            && Version::parse(previous)? >= version
        {
            continue;
        }
        for (arch, source) in &appcasts {
            for locale in ["zh-Hans", "en"] {
                let xml = source.replace("</item>", &format!("<sparkle:releaseNotesLink>{url}/notes-macos-{locale}.md</sparkle:releaseNotesLink></item>"));
                staged.push((
                    format!("appcast-{arch}-{audience}-{locale}.xml"),
                    xml.into_bytes(),
                ));
            }
        }
        for (arch, extension, bytes) in &windows_feeds {
            staged.push((
                format!("releases.win-{arch}-{audience}.{extension}"),
                bytes.clone(),
            ));
        }
        channels.insert((*audience).into(), release.version.clone());
    }
    fs::create_dir_all(output)?;
    for (name, value) in staged {
        fs::write(output.join(name), value)?;
    }
    fs::write(channels_file, serde_json::to_vec_pretty(&channels)?)?;
    Ok(())
}

fn verify_macos(
    directory: &Path,
    xml: &str,
    version: &str,
    arch: &str,
    url: &str,
    public_key: &str,
) -> Result<()> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use ed25519_dalek::{Signature, VerifyingKey};
    use quick_xml::{Reader, events::Event};
    let mut reader = Reader::from_str(xml);
    let mut items = 0;
    let mut display_version = String::new();
    let mut enclosure = None;
    loop {
        match reader.read_event()? {
            Event::Start(event) if event.name().as_ref() == b"item" => items += 1,
            Event::Start(event) if event.name().as_ref() == b"sparkle:shortVersionString" => {
                display_version = reader.read_text(event.name())?.into_owned()
            }
            Event::Empty(event) | Event::Start(event) if event.name().as_ref() == b"enclosure" => {
                ensure(enclosure.is_none(), "Multiple Sparkle enclosures")?;
                let mut attributes = std::collections::BTreeMap::new();
                for attribute in event.attributes() {
                    let attribute = attribute?;
                    attributes.insert(
                        String::from_utf8(attribute.key.as_ref().to_vec())?,
                        attribute.unescape_value()?.into_owned(),
                    );
                }
                enclosure = Some(attributes);
            }
            Event::DocType(_) => return Err("DTD is forbidden in update metadata".into()),
            Event::Eof => break,
            _ => {}
        }
    }
    ensure(
        items == 1 && display_version == version,
        "Sparkle version or item count mismatch",
    )?;
    let enclosure = enclosure.ok_or("Missing Sparkle enclosure")?;
    let name = format!("SerialStudio-{version}-macos-{arch}.app.zip");
    let bytes = fs::read(safe_file(directory, &name)?)?;
    ensure(
        enclosure.get("url") == Some(&format!("{url}/{name}")),
        "Sparkle archive URL mismatch",
    )?;
    ensure(
        enclosure
            .get("length")
            .and_then(|length| length.parse::<usize>().ok())
            == Some(bytes.len()),
        "Sparkle archive size mismatch",
    )?;
    let key: [u8; 32] = STANDARD
        .decode(public_key.trim())?
        .try_into()
        .map_err(|_| "Invalid Sparkle public key")?;
    let signature = Signature::from_slice(
        &STANDARD.decode(
            enclosure
                .get("sparkle:edSignature")
                .ok_or("Missing Sparkle signature")?,
        )?,
    )?;
    VerifyingKey::from_bytes(&key)?.verify_strict(&bytes, &signature)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{Engine, engine::general_purpose::STANDARD};
    use ed25519_dalek::{Signer, SigningKey};

    fn release_fixture(version: &str) -> (tempfile::TempDir, String, String) {
        use p256::pkcs8::EncodePublicKey;
        use sha2::{Digest, Sha256};
        let directory = tempfile::tempdir().unwrap();
        let release = versions(&format!("v{version}")).unwrap();
        let signing = SigningKey::from_bytes(&[7; 32]);
        let sparkle_key = STANDARD.encode(signing.verifying_key().to_bytes());
        let signature = STANDARD.encode(signing.sign(b"archive").to_bytes());
        for arch in ["arm64", "x86_64"] {
            let name = format!("SerialStudio-{version}-macos-{arch}.app.zip");
            fs::write(directory.path().join(&name), b"archive").unwrap();
            fs::write(
                directory
                    .path()
                    .join(format!("SerialStudio-{version}-macos-{arch}.dmg")),
                b"disk",
            )
            .unwrap();
            let xml = format!(
                "<rss xmlns:sparkle=\"http://www.andymatuschak.org/xml-namespaces/sparkle\"><channel><item><sparkle:shortVersionString>{version}</sparkle:shortVersionString><enclosure url=\"https://github.com/tangxiangong/serial-port/releases/download/v{version}/{name}\" length=\"7\" sparkle:edSignature=\"{signature}\"/></item></channel></rss>"
            );
            fs::write(directory.path().join(format!("appcast-{arch}.xml")), xml).unwrap();
        }
        for locale in ["zh-Hans", "en"] {
            fs::write(
                directory.path().join(format!("notes-macos-{locale}.md")),
                "macOS notes",
            )
            .unwrap();
        }
        let signing = p256::ecdsa::SigningKey::from_bytes((&[23; 32]).into()).unwrap();
        let windows_key = STANDARD.encode(
            signing
                .verifying_key()
                .to_public_key_der()
                .unwrap()
                .as_bytes(),
        );
        for arch in ["x64", "arm64"] {
            let filename = format!(
                "SerialStudio-{version}-win-{arch}-{}-full.nupkg",
                release.channel
            );
            fs::write(directory.path().join(&filename), b"package").unwrap();
            fs::write(
                directory
                    .path()
                    .join(format!("SerialStudio-{version}-windows-{arch}.exe")),
                b"installer",
            )
            .unwrap();
            let feed = serde_json::json!({"Assets":[{"PackageId":"SerialStudio","Version":version,"Type":"Full","FileName":filename,"Size":7,"SHA256":format!("{:x}", Sha256::digest(b"package")),"NotesMarkdown":"Windows notes"}]});
            fs::write(
                directory
                    .path()
                    .join(format!("releases.win-{arch}-{}.json", release.channel)),
                serde_json::to_vec(&feed).unwrap(),
            )
            .unwrap();
        }
        windows::sign_with(
            directory.path(),
            version,
            &release.channel,
            &windows_key,
            &signing,
        )
        .unwrap();
        (directory, sparkle_key, windows_key)
    }

    #[test]
    fn channel_promotion_preserves_newer_previews_and_checks_older_payloads() {
        let output = tempfile::tempdir().unwrap();
        for version in ["1.0.0", "2.0.0-beta.1", "1.5.0"] {
            let (directory, sparkle, windows) = release_fixture(version);
            feeds_with_keys(
                directory.path(),
                output.path(),
                &format!("v{version}"),
                "tangxiangong/serial-port",
                &sparkle,
                &windows,
            )
            .unwrap();
        }
        let channels: serde_json::Value =
            serde_json::from_slice(&fs::read(output.path().join("channels.json")).unwrap())
                .unwrap();
        assert_eq!(channels["stable"], "1.5.0");
        assert_eq!(channels["beta"], "2.0.0-beta.1");
        assert_eq!(channels["alpha"], "2.0.0-beta.1");
        let beta = fs::read_to_string(output.path().join("appcast-arm64-beta-en.xml")).unwrap();
        assert!(beta.contains("v2.0.0-beta.1/notes-macos-en.md"));
        assert!(!beta.contains("v1.5.0/"));
        let history = fs::read(output.path().join("channels.json")).unwrap();
        let (directory, sparkle, windows) = release_fixture("0.9.0");
        fs::write(
            directory
                .path()
                .join("SerialStudio-0.9.0-macos-arm64.app.zip"),
            b"corrupt",
        )
        .unwrap();
        assert!(
            feeds_with_keys(
                directory.path(),
                output.path(),
                "v0.9.0",
                "tangxiangong/serial-port",
                &sparkle,
                &windows
            )
            .is_err()
        );
        assert_eq!(
            fs::read(output.path().join("channels.json")).unwrap(),
            history
        );
    }

    #[test]
    fn sparkle_verification_rejects_tampering_and_wrong_identity() {
        let directory = tempfile::tempdir().unwrap();
        let name = "SerialStudio-1.0.0-macos-arm64.app.zip";
        fs::write(directory.path().join(name), b"archive").unwrap();
        let signing = SigningKey::from_bytes(&[7; 32]);
        let key = STANDARD.encode(signing.verifying_key().to_bytes());
        let signature = STANDARD.encode(signing.sign(b"archive").to_bytes());
        let xml = format!(
            "<rss xmlns:sparkle=\"http://www.andymatuschak.org/xml-namespaces/sparkle\"><channel><item><sparkle:shortVersionString>1.0.0</sparkle:shortVersionString><enclosure url=\"https://example.test/{name}\" length=\"7\" sparkle:edSignature=\"{signature}\"/></item></channel></rss>"
        );
        verify_macos(
            directory.path(),
            &xml,
            "1.0.0",
            "arm64",
            "https://example.test",
            &key,
        )
        .unwrap();
        assert!(
            verify_macos(
                directory.path(),
                &xml,
                "1.0.0",
                "arm64",
                "https://wrong.test",
                &key
            )
            .is_err()
        );
        assert!(
            verify_macos(
                directory.path(),
                &xml,
                "2.0.0",
                "arm64",
                "https://example.test",
                &key
            )
            .is_err()
        );
        fs::write(directory.path().join(name), b"changed").unwrap();
        assert!(
            verify_macos(
                directory.path(),
                &xml,
                "1.0.0",
                "arm64",
                "https://example.test",
                &key
            )
            .is_err()
        );
    }
    #[test]
    fn release_versions_accept_only_supported_channels() {
        for value in ["v0.1.0", "v0.1.1-beta.2", "v0.2.0-alpha.1"] {
            assert!(versions(value).is_ok());
        }
        for value in [
            "0.1.0",
            "v0.1.0-rc.1",
            "v0.1.0+build.1",
            "v0.1.0-beta",
            "v0.1.0-alpha.x",
        ] {
            assert!(versions(value).is_err());
        }
    }
}
