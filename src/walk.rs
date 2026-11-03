use super::*;

pub(crate) struct Walk {
  pub(crate) empty: Vec<RelativePath>,
  pub(crate) entries: Vec<(RelativePath, EntryType)>,
  pub(crate) subpackages: Vec<RelativePath>,
}

impl Walk {
  pub(crate) fn new(root: &Utf8Path, manifest: &Utf8Path, ignore: &[RelativePath]) -> Result<Self> {
    let current_dir = current_dir()?;

    let manifest = current_dir.join(manifest).lexiclean();

    let mut empty = Vec::new();

    let mut entries = Vec::new();

    let mut subpackages = Vec::new();

    for entry in WalkDir::new(root).sort_by_file_name() {
      let entry = entry?;

      if entry.depth() == 0 {
        continue;
      }

      let path = decode_path(entry.path())?;

      if current_dir.join(path).lexiclean() == manifest {
        continue;
      }

      let path = path.strip_prefix(root).unwrap();

      let path = RelativePath::try_from(path).context(error::Path { path })?;

      if path.components().any(|component| component == State::DIR)
        || ignore.iter().any(|ignore| path.starts_with(ignore))
      {
        continue;
      }

      if entry.depth() > 1 && entry.file_type().is_file() && path.filename() == Manifest::FILENAME {
        subpackages.push(path.parent().unwrap());
        continue;
      }

      ensure! {
        !path.filename().is_reserved(),
        error::ReservedFilename { path },
      }

      ensure! {
        !entry.file_type().is_symlink(),
        error::Symlink { path },
      }

      empty.pop_if(|dir| path.starts_with(dir));

      if entry.file_type().is_dir() {
        empty.push(path.clone());
        entries.push((path, EntryType::Directory));
      } else {
        entries.push((path, EntryType::File));
      }
    }

    Ok(Self {
      empty,
      entries,
      subpackages,
    })
  }
}
