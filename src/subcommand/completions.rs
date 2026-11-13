use super::*;

#[derive(Parser)]
pub(crate) struct Completions {
  #[arg(help = "Print completion script for <SHELL>")]
  shell: Shell,
}

pub(crate) type Input = ();

pub(crate) type Output = ();

impl Completions {
  #[expect(clippy::unnecessary_wraps)]
  pub(crate) fn run(self) -> Result {
    print!("{}", self.shell.completion_script());

    Ok(())
  }
}
