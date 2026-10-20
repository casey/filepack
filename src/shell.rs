use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Shell {
  Bash,
  Elvish,
  Fish,
  Powershell,
  Zsh,
}

impl Shell {
  pub(crate) fn completion_script(self) -> &'static str {
    match self {
      Self::Bash => include_str!("../completions/filepack.bash"),
      Self::Elvish => include_str!("../completions/filepack.elvish"),
      Self::Fish => include_str!("../completions/filepack.fish"),
      Self::Powershell => include_str!("../completions/filepack.powershell"),
      Self::Zsh => include_str!("../completions/filepack.zsh"),
    }
  }
}
