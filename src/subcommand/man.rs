use {super::*, clap::CommandFactory, clap_mangen::Man};

pub(crate) type Input = ();

pub(crate) type Output = ();

pub(crate) fn run() -> Result {
  let mut man = Vec::<u8>::new();

  Man::new(Arguments::command()).render(&mut man).unwrap();

  print!("{}", str::from_utf8(&man).unwrap());

  Ok(())
}
