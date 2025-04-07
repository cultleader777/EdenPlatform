use std::{
    collections::BTreeMap, fmt::Display, fs::Permissions, io::Write,
    os::unix::prelude::PermissionsExt, process::Command, str::FromStr,
};

use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::static_analysis::PlatformValidationError;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SecretKind {
    Guid,
    ConsulEncryptionKey,
    NomadEncryptionKey,
    DnsRfc2136Key,
    TlsCertificate,
    TlsPrivateKey,
    VaultToken,
    DnsSecZSKPublicKey,
    DnsSecZSKPrivateKey,
    DnsSecKSKPublicKey,
    DnsSecKSKPrivateKey,
    DnsSecDSRecord,
    WireguardPrivateKey,
    WireguardPublicKey,
    OspfPassword,
    SshPrivateKey,
    SshPublicKey,
    NixCachePrivateKey,
    NixCachePublicKey,
    StrongPassword42Symbols,
    HtpasswdFile,
    LibsodiumBoxPrivateKey,
    Misc, // Any complex secret that doesn't fix any other format
}

struct Secret {
    contents: String,
    salt: String,
    checksum: String,
    kind: SecretKind,
}

pub struct SecretsStorage {
    map: BTreeMap<String, Secret>,
    secrets_path: String,
    checksums_path: String,
    is_changed_since_write: bool,
}

#[derive(Serialize, Deserialize)]
struct SecretChecksum {
    salt: String,
    kind: SecretKind,
    checksum: String,
}

#[derive(Clone)]
pub struct SecretFile {
    pub kind: SecretKind,
    pub key: String,
    pub file_name: String,
}

#[derive(Debug)]
pub struct UnknownSecretKindError {
    value: String,
}

impl std::error::Error for UnknownSecretKindError {}

impl std::fmt::Display for UnknownSecretKindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Unknown secret kind value: {}", self.value))
    }
}

impl FromStr for SecretKind {
    type Err = UnknownSecretKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Guid" => Ok(SecretKind::Guid),
            "ConsulEncryptionKey" => Ok(SecretKind::ConsulEncryptionKey),
            "NomadEncryptionKey" => Ok(SecretKind::NomadEncryptionKey),
            "TlsCertificate" => Ok(SecretKind::TlsCertificate),
            "TlsPrivateKey" => Ok(SecretKind::TlsPrivateKey),
            "Misc" => Ok(SecretKind::Misc),
            "VaultToken" => Ok(SecretKind::VaultToken),
            unknown => Err(UnknownSecretKindError {
                value: unknown.to_string(),
            }),
        }
    }
}

pub fn sec_files(deps: &[(SecretKind, &str, &str)]) -> Vec<SecretFile> {
    let mut res = Vec::with_capacity(deps.len());

    for (kind, key, file) in deps {
        res.push(SecretFile {
            kind: *kind,
            key: key.to_string(),
            file_name: file.to_string(),
        })
    }

    res
}

type RawSecretStorageFormat = BTreeMap<String, String>;

type ChecksumStorageFormat = BTreeMap<String, SecretChecksum>;

#[derive(Clone)]
pub struct SecretValue {
    contents: String,
}

impl SecretValue {
    pub fn value(&self) -> &String {
        &self.contents
    }

    /// Don't use it unless we can't derive secret from secrets engine
    pub fn from_string(contents: String) -> SecretValue {
        SecretValue { contents }
    }
}

impl Display for SecretValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.contents.as_str())
    }
}

fn digest(bytes: &[u8]) -> String {
    let mut hash = hmac_sha256::Hash::new();
    hash.update(bytes);
    let fin = hash.finalize();
    hex::encode(fin)
}


#[cfg(test)]
fn mocked_secret_value(key: &str, sk: &SecretKind) -> String {
    match sk {
        // We mock these because regex is used to parse values and it might fail
        SecretKind::DnsSecKSKPublicKey => {
            r#"
  ; This is a key-signing key, keyid 2439, for 10.in-addr.arpa.
  ; Created: 20230724172044 (Mon Jul 24 20:20:44 2023)
  ; Publish: 20230724172044 (Mon Jul 24 20:20:44 2023)
  ; Activate: 20230724172044 (Mon Jul 24 20:20:44 2023)
  10.in-addr.arpa. IN DNSKEY 257 3 15 CSwz13q+gdrPxEY63YagwXaP5Fz13khQqZd4WZ3F3sA=
"#.to_string()
        }
        SecretKind::DnsSecZSKPublicKey => {
            r#"
; This is a zone-signing key, keyid 18630, for epl-infra.net.
; Created: 20230714153245 (Fri Jul 14 15:32:45 2023)
; Publish: 20230714153245 (Fri Jul 14 15:32:45 2023)
; Activate: 20230714153245 (Fri Jul 14 15:32:45 2023)
epl-infra.net. IN DNSKEY 256 3 15 IgTyZxRb9t3VVD6A1fO9Lnq17op2Jl+sMR6XxzoxO/A=
"#.to_string()
        }
        _ => {
            format!("SECRET_VALUE_{key}")
        }
    }
}

impl SecretsStorage {
    #[cfg(test)]
    pub fn new_testing() -> SecretsStorage {
        SecretsStorage {
            map: BTreeMap::new(),
            secrets_path: "/dev/null".to_string(),
            checksums_path: "/dev/null".to_string(),
            is_changed_since_write: false,
        }
    }

    pub fn new(
        secrets_path: &str,
        checksums_path: &str,
    ) -> Result<SecretsStorage, PlatformValidationError> {
        let secrets_map: RawSecretStorageFormat = std::fs::read(secrets_path)
            .map(|bytes| {
                serde_yaml::from_slice(bytes.as_slice()).map_err(|e| {
                    PlatformValidationError::SecretsFileCannotBeDeserialized {
                        secrets_file_path: secrets_path.to_string(),
                        deserialization_error: e.to_string(),
                    }
                })
            })
            .unwrap_or_else(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(BTreeMap::new())
                } else {
                    Err(PlatformValidationError::SecretsFileCannotBeOpened {
                        secrets_file_path: secrets_path.to_string(),
                        error: e.to_string(),
                    })
                }
            })?;

        let mut checksums_map: ChecksumStorageFormat = std::fs::read(checksums_path)
            .map(|bytes| {
                serde_yaml::from_slice(bytes.as_slice()).map_err(|e| {
                    PlatformValidationError::SecretsChecksumsFileCannotBeDeserialized {
                        checksums_file_path: checksums_path.to_string(),
                        deserialization_error: e.to_string(),
                    }
                })
            })
            .unwrap_or_else(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(BTreeMap::new())
                } else {
                    Err(
                        PlatformValidationError::SecretsChecksumsFileCannotBeOpened {
                            checksums_file_path: checksums_path.to_string(),
                            error: e.to_string(),
                        },
                    )
                }
            })?;

        for k in secrets_map.keys() {
            if checksums_map.get(k).is_none() {
                return Err(
                    PlatformValidationError::SecretInSecretsFileDoesNotExistInChecksumFile {
                        secrets_file_path: secrets_path.to_string(),
                        checksums_file_path: checksums_path.to_string(),
                        key: k.clone(),
                    },
                );
            }
        }

        for k in checksums_map.keys() {
            if secrets_map.get(k).is_none() {
                return Err(
                    PlatformValidationError::ChecksumInChecksumsFileDoesNotExistInSecretsFile {
                        secrets_file_path: secrets_path.to_string(),
                        checksums_file_path: checksums_path.to_string(),
                        key: k.clone(),
                    },
                );
            }
        }

        assert_eq!(secrets_map.len(), checksums_map.len());

        let mut secrets_storage: BTreeMap<String, Secret> = BTreeMap::new();

        for (k, v) in secrets_map {
            let cs = checksums_map.remove(&k).unwrap();
            let to_hash = format!("{}{}", v, cs.salt);
            let checksum = digest(to_hash.as_bytes());

            if checksum != cs.checksum {
                return Err(PlatformValidationError::ChecksumMismatchForKey {
                    secrets_file_path: secrets_path.to_string(),
                    checksums_file_path: checksums_path.to_string(),
                    key: k.clone(),
                    expected_secret_checksum: cs.checksum,
                    actual_secret_checksum: checksum,
                });
            }

            let o = secrets_storage.insert(
                k,
                Secret {
                    contents: v,
                    salt: cs.salt,
                    checksum: cs.checksum,
                    kind: cs.kind,
                },
            );

            assert!(o.is_none());
        }

        Ok(SecretsStorage {
            map: secrets_storage,
            is_changed_since_write: false,
            secrets_path: secrets_path.to_string(),
            checksums_path: checksums_path.to_string(),
        })
    }

    #[cfg(not(test))]
    pub fn fetch_secret(&mut self, key: String, kind: SecretKind) -> SecretValue {
        let e = self.map.entry(key).or_insert_with(|| {
            self.is_changed_since_write = true;
            let contents = generate_secret(kind);
            let salt = random_base64_bytes(16);
            let to_hash = format!("{}{}", contents, salt);
            let checksum = digest(to_hash.as_bytes());
            Secret {
                contents,
                salt,
                checksum,
                kind,
            }
        });

        assert_eq!(e.kind, kind);

        SecretValue {
            contents: e.contents.clone(),
        }
    }

    #[cfg(test)]
    pub fn fetch_secret(&mut self, key: String, kind: SecretKind) -> SecretValue {
        let e = self.map.entry(key.clone()).or_insert_with(|| {
            self.is_changed_since_write = true;
            let contents = mocked_secret_value(&key, &kind);
            let salt = random_base64_bytes(16);
            let to_hash = format!("{}{}", contents, salt);
            let checksum = digest(to_hash.as_bytes());
            Secret {
                contents,
                salt,
                checksum,
                kind,
            }
        });

        assert_eq!(e.kind, kind);

        SecretValue {
            contents: e.contents.clone(),
        }
    }

    pub fn get_secret(&self, key: &String) -> Option<SecretValue> {
        self.map.get(key).map(|i| SecretValue {
            contents: i.contents.clone(),
        })
    }

    /// For things like we need to invalidate tls certs
    pub fn delete_secret(&mut self, key: &str) {
        assert!(self.map.remove(key).is_some(), "Secret {key} doesn't exist")
    }

    #[cfg(test)]
    pub fn multi_secret_derive(
        &mut self,
        _static_inputs: &[(&str, &str)],
        inputs: Vec<SecretFile>,
        outputs: Vec<SecretFile>,
        _shell_script: &str,
    ) -> Vec<SecretValue> {

        let mut missing_input_secrets = Vec::new();
        for input in &inputs {
            if !self.contains_secret(&input.key) {
                missing_input_secrets.push(input.key.clone())
            } else {
                // ensure inputs are of appropriate kind
                assert_eq!(
                    input.kind,
                    self.map.get(&input.key).unwrap().kind,
                    "Input secrets for derivation diverge in kinds"
                );
            }
        }

        if !missing_input_secrets.is_empty() {
            panic!(
                "Missing input secrets for secret derivation: {:?}",
                missing_input_secrets
            );
        }

        let mut found_output_secrets = 0;

        for output in &outputs {
            if self.contains_secret(&output.key) {
                found_output_secrets += 1;
            }
        }

        if found_output_secrets == outputs.len() {
            // every secret found, return cached
            let mut res = Vec::with_capacity(outputs.len());
            for output in &outputs {
                res.push(self.fetch_secret(output.key.clone(), output.kind));
            }
            return res;
        }

        if found_output_secrets == 0 {
            // no secrets found, emulate running the derivation
            let mut mock_outputs = Vec::new();
            for o in &outputs {
                let sv = mocked_secret_value(&o.key, &o.kind);
                self.put_secret(o.key.clone(), o.kind, sv.clone());
                mock_outputs.push(SecretValue { contents: sv });
            }

            return mock_outputs;
        }

        let mut partials = Vec::new();
        for output in &outputs {
            if self.contains_secret(&output.key) {
                partials.push(output.key.clone());
            }
        }

        panic!(
            "Partial output secrets found in multi secret derive: {:?}",
            partials
        )
    }

    /// Specify inputs for secret derivation and outputs.
    /// If outputs don't exist run the shell script to derive them.
    /// If inputs are missing in current secret storage state panics.
    /// If partial outputs already exist in current secret storage panics.
    #[cfg(not(test))]
    pub fn multi_secret_derive(
        &mut self,
        static_inputs: &[(&str, &str)],
        inputs: Vec<SecretFile>,
        outputs: Vec<SecretFile>,
        shell_script: &str,
    ) -> Vec<SecretValue> {
        let mut missing_input_secrets = Vec::new();
        for input in &inputs {
            if !self.contains_secret(&input.key) {
                missing_input_secrets.push(input.key.clone())
            } else {
                // ensure inputs are of appropriate kind
                assert_eq!(
                    input.kind,
                    self.map.get(&input.key).unwrap().kind,
                    "Input secrets for derivation diverge in kinds"
                );
            }
        }

        if !missing_input_secrets.is_empty() {
            panic!(
                "Missing input secrets for secret derivation: {:?}",
                missing_input_secrets
            );
        }

        let mut found_output_secrets = 0;

        for output in &outputs {
            if self.contains_secret(&output.key) {
                found_output_secrets += 1;
            }
        }

        if found_output_secrets == outputs.len() {
            // every secret found, return cached
            let mut res = Vec::with_capacity(outputs.len());
            for output in &outputs {
                res.push(self.fetch_secret(output.key.clone(), output.kind));
            }
            return res;
        }

        if found_output_secrets == 0 {
            // no secrets found, run the derivation

            let mut mapped_inputs: Vec<(String, String)> = inputs
                .iter()
                .map(|input| {
                    (
                        input.file_name.clone(),
                        self.fetch_secret(input.key.clone(), input.kind).contents,
                    )
                })
                .collect::<Vec<_>>();
            for (si_file, si_contents) in static_inputs {
                mapped_inputs.push((si_file.to_string(), si_contents.to_string()));
            }
            let mapped_outputs = outputs
                .iter()
                .map(|output| output.file_name.as_str())
                .collect::<Vec<_>>();
            let shell_outputs = crate::codegen::secrets::temp_script_with_outputs(
                mapped_inputs,
                &mapped_outputs,
                shell_script,
            );

            assert_eq!(shell_outputs.len(), outputs.len());
            for (idx, o) in outputs.iter().enumerate() {
                self.put_secret(o.key.clone(), o.kind, shell_outputs[idx].clone());
            }

            return shell_outputs
                .into_iter()
                .map(|i| SecretValue { contents: i })
                .collect();
        }

        let mut partials = Vec::new();
        for output in &outputs {
            if self.contains_secret(&output.key) {
                partials.push(output.key.clone());
            }
        }

        panic!(
            "Partial output secrets found in multi secret derive: {:?}",
            partials
        )
    }

    pub fn contains_secret(&self, key: &String) -> bool {
        self.map.contains_key(key)
    }

    pub fn override_secret(&mut self, key: String, kind: SecretKind, contents: String) {
        let _ = self.map.remove(&key);
        self.put_secret(key, kind, contents);
    }

    pub fn put_secret(&mut self, key: String, kind: SecretKind, contents: String) {
        precheck_secret(kind, &contents);

        if self.map.contains_key(&key) {
            // not validation exception because this is our stuff
            panic!("Key with value {key} was attempted to be put but already exists in map")
        }

        self.is_changed_since_write = true;
        let salt = random_base64_bytes(16);
        let to_hash = format!("{}{}", contents, salt);
        let checksum = digest(to_hash.as_bytes());
        let value = Secret {
            contents,
            salt,
            checksum,
            kind,
        };

        let res = self.map.insert(key, value);
        assert!(res.is_none());
    }

    pub fn flush_to_disk(&mut self) -> Result<(), PlatformValidationError> {
        if !self.is_changed_since_write {
            return Ok(());
        }

        let mut secrets_map: RawSecretStorageFormat = BTreeMap::new();
        let mut checksums_map: ChecksumStorageFormat = BTreeMap::new();

        for (k, v) in &self.map {
            let _ = secrets_map.insert(k.clone(), v.contents.clone());
            let _ = checksums_map.insert(
                k.clone(),
                SecretChecksum {
                    salt: v.salt.clone(),
                    kind: v.kind,
                    checksum: v.checksum.clone(),
                },
            );
        }

        let secrets_file_contents = serde_yaml::to_string(&secrets_map).unwrap();
        let checksums_file_contents = serde_yaml::to_string(&checksums_map).unwrap();

        {
            let temp_path = format!("{}.temp", self.secrets_path);
            let err = |e: std::io::Error| PlatformValidationError::CannotWriteSecretsFile {
                secrets_file_path: temp_path.clone(),
                error: e.to_string(),
            };

            let mut fl = std::fs::File::create(&temp_path).map_err(err)?;
            let mut perms = fl.metadata().map_err(err)?.permissions();
            perms.set_mode(0o600);
            fl.set_permissions(perms).map_err(err)?;
            if let Ok(md) = std::fs::metadata(&self.secrets_path) {
                if md.is_file() {
                    let status = Command::new("chown")
                        .arg("--reference")
                        .arg(&self.secrets_path)
                        .arg(&temp_path)
                        .status()
                        .expect("Cannot chown secrets temp file");
                    if !status.success() {
                        panic!("Failed changing temporary secrets file owner");
                    }
                }
            }
            fl.write_all(secrets_file_contents.as_bytes())
                .map_err(err)?;
            drop(fl);

            std::fs::rename(&temp_path, &self.secrets_path).map_err(|e| {
                PlatformValidationError::CannotWriteSecretsFile {
                    secrets_file_path: self.secrets_path.clone(),
                    error: e.to_string(),
                }
            })?;
        }

        {
            let temp_path = format!("{}.temp", self.checksums_path);
            let err = |e: std::io::Error| PlatformValidationError::CannotWriteChecksumsFile {
                checksums_file_path: temp_path.clone(),
                error: e.to_string(),
            };

            let mut fl = std::fs::File::create(&temp_path).map_err(err)?;
            fl.write_all(checksums_file_contents.as_bytes())
                .map_err(err)?;
            drop(fl);

            std::fs::rename(&temp_path, &self.checksums_path).map_err(|e| {
                PlatformValidationError::CannotWriteChecksumsFile {
                    checksums_file_path: self.checksums_path.clone(),
                    error: e.to_string(),
                }
            })?;
        }

        self.is_changed_since_write = false;

        Ok(())
    }
}

pub fn check_secrets_are_defined(checked: &crate::static_analysis::CheckedDB, storage: &SecretsStorage) -> Result<(), PlatformValidationError> {
    let db = &checked.db;
    for cs in db.custom_secret().rows_iter() {
        let key = db.custom_secret().c_key(cs);
        let key_wprefix = format!("custom_{key}");
        let prefix = if db.custom_secret().c_is_file(cs) { "file" } else { "value" };
        let prefix_uppercase = prefix.to_uppercase();
        if !storage.contains_secret(&key_wprefix) {
            return Err(
                PlatformValidationError::CustomSecretIsUndefined {
                    secret_key: key.to_string(),
                    filename: "secrets.yml".to_string(),
                    suggestion: format!("Please define secret with \"make define-secret_{prefix}_{key} SECRET_{prefix_uppercase}=xxxxxxx\""),
                }
            );
        }
    }

    let secret_check_regex = regex::Regex::new(r"^[ -~]+$").unwrap();
    for region in db.region().rows_iter() {
        let secrets = checked.projections.server_runtime.region_secrets_from_yml(region);
        for (sec, usages) in secrets {
            if !usages.used_in_env {
                continue;
            }

            let key = db.custom_secret().c_key(*sec);
            let key_wprefix = format!("custom_{key}");
            let the_secret = storage.get_secret(&key_wprefix).expect("We checked secrets exist already");
            if !secret_check_regex.is_match(&the_secret.value()) {
                return Err(
                    PlatformValidationError::CustomSecretUsedInEnvironmentButHasSpecialCharacters {
                        secret_key: key.to_string(),
                        regex_check: secret_check_regex.to_string(),
                        suggestion: "Secret doesn't pass regex check and cannot be used as an environment variable. Use it as file instead.".to_string(),
                    }
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
pub fn tls_cert_expiration_days(_sec_val: &SecretValue) -> u64 {
    // always valid for tests
    return 365;
}

#[cfg(not(test))]
pub fn tls_cert_expiration_days(sec_val: &SecretValue) -> u64 {
    let (_, parsed) = x509_parser::pem::parse_x509_pem(sec_val.contents.as_bytes()).expect("Can't parse x509 cert");
    let other_parsed = parsed.parse_x509().expect("Couldn't parse x509 cert");
    let seconds = other_parsed.validity()
                              .time_to_expiration()
                              .map(|i| i.as_secs())
                              .unwrap_or(0);
    return seconds / (60 * 60 * 24);
}

#[cfg(test)]
fn precheck_secret(_kind: SecretKind, _contents: &str) {
    // we're mocking secrets here
}

#[cfg(not(test))]
fn precheck_secret(kind: SecretKind, contents: &str) {
    match kind {
        SecretKind::Guid => {
            uuid::Uuid::from_str(contents).expect("Cannot deserialize valid guid");
        }
        SecretKind::ConsulEncryptionKey => {
            let decoded =
                base64::decode(contents).expect("Cannot deserialize consul encrypt bytes from b64");
            assert_eq!(decoded.len(), 32);
        }
        SecretKind::NomadEncryptionKey => {
            let decoded =
                base64::decode(contents).expect("Cannot deserialize nomad encrypt bytes from b64");
            assert_eq!(decoded.len(), 32);
        }
        SecretKind::DnsRfc2136Key => {
            let decoded =
                base64::decode(contents).expect("Cannot deserialize dns rfc2136 key bytes from b64");
            assert_eq!(decoded.len(), 32);
        }
        SecretKind::TlsCertificate => {
            assert!(contents.contains("CERTIFICATE--"));
        }
        SecretKind::TlsPrivateKey => {
            assert!(contents.contains("PRIVATE KEY--"));
        }
        SecretKind::VaultToken => {
            assert!(contents.contains("hvs."));
        }
        SecretKind::Misc => {}
        SecretKind::DnsSecZSKPrivateKey => {
            assert!(contents.contains("Private-key-format:"));
        }
        SecretKind::DnsSecZSKPublicKey => {
            assert!(contents.contains(" IN DNSKEY 256 "));
        }
        SecretKind::DnsSecKSKPrivateKey => {
            assert!(contents.contains("Private-key-format:"));
        }
        SecretKind::DnsSecKSKPublicKey => {
            assert!(contents.contains(" IN DNSKEY 257 "));
        }
        SecretKind::DnsSecDSRecord => {
            assert!(contents.contains(" IN DS "));
        }
        SecretKind::WireguardPrivateKey => {
            let decoded =
                base64::decode(contents).expect("Cannot deserialize wireguard private key bytes from b64");
            assert_eq!(decoded.len(), 32);
        }
        SecretKind::WireguardPublicKey => {
            let decoded =
                base64::decode(contents).expect("Cannot deserialize wireguard public key bytes from b64");
            assert_eq!(decoded.len(), 32);
        }
        SecretKind::OspfPassword => {
            assert_eq!(contents.len(), 16, "Ospf password must be 16 bytes in length");
        }
        SecretKind::SshPrivateKey => {
            assert!(contents.contains("-----BEGIN OPENSSH PRIVATE KEY-----"), "This ain't no ssh key?");
        }
        SecretKind::SshPublicKey => {
            println!("|{contents}|");
            assert!(contents.starts_with("ssh-ed25519 A"), "This ain't no ssh public key?");
        }
        SecretKind::NixCachePrivateKey => {
            let spl = contents.split(":").collect::<Vec<_>>();
            assert_eq!(spl.len(), 2, "Invalid nix cache private key");
            let dec = base64::decode(spl[1]);
            assert!(dec.is_ok(), "Invalid nix cache private key");
            assert_eq!(dec.unwrap().len(), 64, "Invalid nix cache private key");
        },
        SecretKind::NixCachePublicKey => {
            let spl = contents.split(":").collect::<Vec<_>>();
            assert_eq!(spl.len(), 2, "Invalid nix cache public key");
            let dec = base64::decode(spl[1]);
            assert!(dec.is_ok(), "Invalid nix cache public key");
            assert_eq!(dec.unwrap().len(), 32, "Invalid nix cache public key");
        },
        SecretKind::HtpasswdFile => {
            assert!(contents.contains(":$2a$"), "Invalid htpasswd file [{contents}]");
        }
        SecretKind::LibsodiumBoxPrivateKey => {
            let key = base64::decode(contents).expect("Cannot decode libsodium private key");
            assert_eq!(key.len(), sodiumoxide::crypto::box_::SECRETKEYBYTES, "Invalid sodium box secret key length");
        }
        SecretKind::StrongPassword42Symbols => {
            assert_eq!(contents.len(), 42);
        }
    }
}

#[cfg(not(test))]
fn generate_secret(kind: SecretKind) -> String {
    match kind {
        SecretKind::Guid => uuid::Uuid::new_v4().to_string(),
        SecretKind::ConsulEncryptionKey => random_base64_bytes(32),
        SecretKind::NomadEncryptionKey => random_base64_bytes(32),
        SecretKind::DnsRfc2136Key => random_base64_bytes(32),
        SecretKind::LibsodiumBoxPrivateKey => {
            let (_, sec_key) = sodiumoxide::crypto::box_::gen_keypair();
            base64::encode(sec_key.0)
        }
        SecretKind::OspfPassword => {
            random_alphanum_chars(16)
        }
        SecretKind::StrongPassword42Symbols => {
            random_alphanum_chars(42)
        }
        SecretKind::TlsCertificate => {
            panic!("TLS certificates cannot be trivially generated")
        }
        SecretKind::TlsPrivateKey => {
            panic!("TLS private keys cannot be trivially generated")
        }
        SecretKind::Misc => {
            panic!("Miscellanious secrets cannot be generated")
        }
        SecretKind::VaultToken => {
            panic!("Vault tokens cannot be generated")
        }
        SecretKind::DnsSecZSKPrivateKey => {
            panic!("DNS Sec private keys cannot be generated standalone, user derivation")
        }
        SecretKind::DnsSecZSKPublicKey => {
            panic!("DNS Sec public keys cannot be generated standalone, user derivation")
        }
        SecretKind::DnsSecKSKPrivateKey => {
            panic!("DNS Sec private keys cannot be generated standalone, use derivation")
        }
        SecretKind::DnsSecKSKPublicKey => {
            panic!("DNS Sec public keys cannot be generated standalone, use derivation")
        }
        SecretKind::DnsSecDSRecord => {
            panic!("DNS Sec ds record cannot be generated standalone, use derivation")
        }
        SecretKind::WireguardPrivateKey => {
            panic!("Wireguard private key cannot be generated standalone, use derivation")
        }
        SecretKind::WireguardPublicKey => {
            panic!("Wireguard public key cannot be generated standalone, use derivation")
        },
        SecretKind::SshPrivateKey => {
            panic!("SSH private keys cannot be generated, use derivation")
        },
        SecretKind::SshPublicKey => {
            panic!("SSH public keys cannot be generated, use derivation")
        },
        SecretKind::NixCachePrivateKey => {
            panic!("Nix cache private keys cannot be generated, use derivation")
        },
        SecretKind::NixCachePublicKey => {
            panic!("Nix cache public keys cannot be generated, use derivation")
        },
        SecretKind::HtpasswdFile => {
            panic!(".htpasswd file cannot be generated, use derivation")
        },
    }
}

fn random_base64_bytes(len: usize) -> String {
    let mut random_bytes = vec![0; len];
    OsRng.fill_bytes(&mut random_bytes);
    base64::encode(random_bytes)
}

#[cfg(not(test))]
fn random_alphanum_chars(len: usize) -> String {
    let mut random_bytes = String::with_capacity(len);
    while random_bytes.len() < len {
        if let Some(c) = char::from_u32(0x7f & OsRng.next_u32()) {
            if c.is_ascii_alphanumeric() {
                random_bytes.push(c);
            }
        }
    }
    random_bytes
}

pub fn temp_script_with_outputs(
    input_files: Vec<(String, String)>,
    output_files: &[&str],
    input: &str,
) -> Vec<String> {
    let mut res = Vec::with_capacity(output_files.len());

    let tdir = tempfile::tempdir().expect("Cannot create temp dir for script");
    let perms = Permissions::from_mode(0o700);
    std::fs::set_permissions(tdir.path(), perms).expect("Can't set permissions for temp dir");

    for (fname, fcontents) in input_files {
        std::fs::write(tdir.path().join(&fname), fcontents).expect("Cannot write input file");
    }

    let src_path = tdir.path().join("_______script");
    std::fs::write(&src_path, input).expect("Cannot write tmp script");

    let status = Command::new("sh")
        .current_dir(tdir.path())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .arg("-C")
        .arg(src_path.to_str().unwrap())
        .status()
        .expect("Cannot run shell command");

    if !status.success() {
        panic!("Running shell command failed");
    }

    for of in output_files {
        res.push(
            std::fs::read_to_string(tdir.path().join(of))
                .expect("Cannot read expected output file"),
        );
    }

    res
}

#[test]
fn test_tmp_script_output_generation() {
    let res = temp_script_with_outputs(
        vec![],
        &["a", "d"],
        r#"
            echo -n henlo > a
            echo -n world > d
        "#,
    );

    assert_eq!(res, vec!["henlo", "world"]);
}

#[test]
fn test_tmp_script_output_generation_with_inputs() {
    let res = temp_script_with_outputs(
        vec![
            ("a".to_string(), "henlo".to_string()),
            ("d".to_string(), "world".to_string()),
        ],
        &["c"],
        r#"
      cat a d > c
    "#,
    );

    assert_eq!(res, vec!["henloworld"]);
}
