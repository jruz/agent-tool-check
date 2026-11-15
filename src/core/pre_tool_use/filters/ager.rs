use std::{env, fs, io, path};

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    #[serde(rename = "identity")]
    _identity: Option<String>,
    entries: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Ciphertext,
    Unlisted,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to read project ager.yaml")]
    Io(#[from] io::Error),
    #[error("Access to ager plaintext file blocked; use its .age file instead")]
    Plaintext,
    #[error("failed to parse project ager.yaml")]
    Yaml(#[from] serde_saphyr::Error),
}

pub fn check_path(value: &str) -> Result<Access, Error> {
    let root = env::current_dir()?;
    let Some(manifest) = read_manifest(&root)? else {
        let access = Access::Unlisted;
        return Ok(access);
    };
    let file = root.join(value);
    let file = normalize(&file);
    for entry in &manifest.entries {
        let plaintext = root.join(entry);
        let plaintext = normalize(&plaintext);
        if file == plaintext {
            return Err(Error::Plaintext);
        }
        let encrypted = format!("{entry}.age");
        let encrypted = root.join(encrypted);
        let encrypted = normalize(&encrypted);
        if file == encrypted {
            let access = Access::Ciphertext;
            return Ok(access);
        }
    }
    let access = Access::Unlisted;
    Ok(access)
}

fn read_manifest(root: &path::Path) -> Result<Option<Manifest>, Error> {
    let manifest_path = root.join("ager.yaml");
    let content = match fs::read_to_string(manifest_path) {
        Ok(content) => content,
        Err(error) => return missing_manifest(error),
    };
    let manifest = serde_saphyr::from_str(&content)?;
    let manifest = Some(manifest);
    Ok(manifest)
}

fn missing_manifest(error: io::Error) -> Result<Option<Manifest>, Error> {
    if error.kind() == io::ErrorKind::NotFound {
        Ok(None)
    } else {
        let error = Error::Io(error);
        Err(error)
    }
}

fn normalize(file: &path::Path) -> path::PathBuf {
    let empty = path::PathBuf::new();
    file.components().fold(empty, |mut result, component| {
        match component {
            path::Component::ParentDir => {
                result.pop();
            }
            path::Component::CurDir => {}
            path::Component::Prefix(_) | path::Component::RootDir | path::Component::Normal(_) => {
                result.push(component);
            }
        }
        result
    })
}
