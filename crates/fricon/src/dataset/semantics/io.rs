use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;

use crate::dataset::{
    semantics::{DatasetSemanticManifest, ManifestError},
    storage::layout::manifest_path,
};

pub fn read_manifest(
    path_or_dataset_dir: impl AsRef<Path>,
) -> Result<DatasetSemanticManifest, ManifestError> {
    let path = resolve_manifest_path(path_or_dataset_dir.as_ref());
    let file = File::open(path)?;
    let manifest: DatasetSemanticManifest = serde_json::from_reader(BufReader::new(file))?;
    manifest.validate()?;
    Ok(manifest)
}

pub fn write_manifest(
    dataset_dir: impl AsRef<Path>,
    manifest: &DatasetSemanticManifest,
) -> Result<(), ManifestError> {
    manifest.validate()?;
    let path = manifest_path(dataset_dir.as_ref());
    let mut file = NamedTempFile::new_in(
        path.parent()
            .expect("manifest path should always have a dataset directory parent"),
    )?;
    serde_json::to_writer_pretty(&mut file, manifest)?;
    file.persist(path)?;
    Ok(())
}

fn resolve_manifest_path(path_or_dataset_dir: &Path) -> PathBuf {
    if path_or_dataset_dir.is_dir() {
        manifest_path(path_or_dataset_dir)
    } else {
        path_or_dataset_dir.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use tempfile::tempdir;

    use crate::dataset::{
        semantics::{
            DatasetDType, DatasetSemanticManifest, ManifestColumn, read_manifest, write_manifest,
        },
        storage::layout::manifest_path,
    };

    fn manifest() -> DatasetSemanticManifest {
        DatasetSemanticManifest::minimal(BTreeMap::from([(
            "signal".to_string(),
            ManifestColumn::new(DatasetDType::Float64),
        )]))
    }

    #[test]
    fn write_and_read_manifest_round_trips_pretty_json() {
        let dir = tempdir().expect("temp dir");
        let manifest = manifest();

        write_manifest(dir.path(), &manifest).expect("write manifest");
        let json = std::fs::read_to_string(manifest_path(dir.path())).expect("read json");
        assert!(
            json.contains("\n  \"manifest_version\""),
            "manifest should be written as pretty JSON"
        );

        let from_dir = read_manifest(dir.path()).expect("read manifest from dataset dir");
        let from_file =
            read_manifest(manifest_path(dir.path())).expect("read manifest from explicit path");
        assert_eq!(from_dir, manifest);
        assert_eq!(from_file, manifest);
    }
}
