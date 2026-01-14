use super::*;

#[derive(Clone, Copy, Debug)]
pub struct Mode(u32);

impl Mode {
  pub(crate) fn is_secure(self) -> bool {
    self.0.trailing_zeros() >= 6
  }
}

impl Display for Mode {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "0{:03o}", self.0 & 0o777)
  }
}

impl From<Permissions> for Mode {
  fn from(permissions: Permissions) -> Self {
    #[cfg(unix)]
    let mode = {
      use std::os::unix::fs::PermissionsExt;
      permissions.mode()
    };

    #[cfg(not(unix))]
    let mode = 0;

    Self(mode)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(Mode(0).to_string(), "0000");
    assert_eq!(Mode(0o777).to_string(), "0777");
    assert_eq!(Mode(0o7777).to_string(), "0777");
  }
}
