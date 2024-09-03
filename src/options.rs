use super::*;

#[derive(Parser)]
pub(crate) struct Options {
  #[arg(long, help = "Memory-map files for hashing")]
  pub(crate) mmap: bool,
}
