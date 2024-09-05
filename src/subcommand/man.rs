use {super::*, clap::CommandFactory, clap_mangen::Man};

pub(crate) fn run() -> Result {
  let mut man = Vec::<u8>::new();

  Man::new(Arguments::command())
    .render(&mut man)
    .expect("writing to buffer cannot fail");

  let man = String::from_utf8(man).expect("man page is UTF-8");

  print!("{man}");

  Ok(())
}
