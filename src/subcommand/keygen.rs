use super::*;

#[derive(Parser)]
pub(crate) struct Keygen {
  #[arg(default_value = "master", help = "Generate key named <NAME>", long)]
  name: KeyName,
}

impl Keygen {
  pub(crate) fn run(self, option: Options) -> Result {
    let dir = option.key_dir()?;

    if filesystem::exists(&dir)? {
      let mode = filesystem::mode(&dir)?;

      ensure! {
        mode.is_secure(),
        error::KeyDirPermissions { path: &dir, mode },
      }
    } else {
      filesystem::create_dir_all_with_mode(&dir, 0o700)?;
    }

    let private_path = dir.join(self.name.private_key_filename());

    ensure! {
      !filesystem::exists(&private_path)?,
      error::PrivateKeyAlreadyExists { path: private_path },
    }

    let public_path = dir.join(self.name.public_key_filename());

    ensure! {
      !filesystem::exists(&public_path)?,
      error::PublicKeyAlreadyExists { path: public_path },
    }

    let private_key = PrivateKey::generate();

    filesystem::write_with_mode(
      &private_path,
      format!("{}\n", private_key.display_secret()),
      0o600,
    )?;

    let public_key = private_key.public_key();

    filesystem::write(&public_path, format!("{public_key}\n"))?;

    Ok(())
  }
}
