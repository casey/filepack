use super::*;

macro_rules! print {
  ($($arg:tt)*) => {
    $crate::macros::print(format_args!($($arg)*))?
  };
}

macro_rules! println {
  () => {
    print!("\n")
  };
  ($($arg:tt)*) => {
    print!("{}\n", format_args!($($arg)*))
  };
}

macro_rules! warn {
  ($($arg:tt)*) => {{
    let style = $crate::Style::stderr();
    eprintln!(
      "{}: {}",
      "warning".style(style.warning()),
      format_args!($($arg)*).style(style.message()),
    );
  }};
}

pub(crate) fn print(args: fmt::Arguments) -> Result {
  if let Err(source) = io::stdout().write_fmt(args)
    && source.kind() != io::ErrorKind::BrokenPipe
  {
    Err(error::StandardOutputIo.into_error(source))
  } else {
    Ok(())
  }
}
