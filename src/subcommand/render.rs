use super::*;

#[derive(Parser)]
pub(crate) struct Render {
  #[arg(
    help = "Render <ROOT>. May be a path to a manifest or a directory containing a manifest named \
    `filepack.json`. If omitted, the manifest `filepack.json` in the current directory is signed."
  )]
  root: Option<Utf8PathBuf>,
}

impl Render {
  pub(crate) fn run(self) -> Result {
    let path = if let Some(path) = self.root {
      if filesystem::metadata(&path)?.is_dir() {
        path.join(Manifest::FILENAME)
      } else {
        path
      }
    } else {
      current_dir()?.join(Manifest::FILENAME)
    };

    let json = filesystem::read_to_string(&path)?;

    let manifest = serde_json::from_str::<Manifest>(&json)
      .context(error::DeserializeManifest { path: &path })?;

    let root = path.parent().unwrap();

    let metadata_path = root.join(Metadata::FILENAME);

    let metadata = if filesystem::exists(&metadata_path)? {
      let json = filesystem::read_to_string(&metadata_path)?;

      Some(
        serde_json::from_str::<Metadata>(&json).context(error::DeserializeMetadata {
          path: &metadata_path,
        })?,
      )
    } else {
      None
    };

    let mut present = HashSet::new();

    for path in manifest.files.keys() {
      if filesystem::exists(&root.join(path))? {
        present.insert(path.clone());
      }
    }

    let page = Page {
      manifest,
      metadata,
      present,
    };

    print!("{page}");

    Ok(())
  }
}
