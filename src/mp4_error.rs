use {super::*, mp4::Fourcc};

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum Mp4Error {
  #[snafu(display("`{tag}` tag has unsupported data type {data_type}"))]
  DataType { data_type: u32, tag: Fourcc },
  #[snafu(display("invalid descriptor {tag:#04x}"))]
  Descriptor { tag: u8 },
  #[snafu(display("duplicate `{ty}` box"))]
  Duplicate { ty: Fourcc },
  #[snafu(display("invalid `{ty}` box"))]
  Invalid {
    #[snafu(source(from(Mp4Error, Box::new)))]
    source: Box<Mp4Error>,
    ty: Fourcc,
  },
  #[snafu(display("I/O error"))]
  Io { source: io::Error },
  #[snafu(display("missing `{ty}` box"))]
  Missing { ty: Fourcc },
  #[snafu(display("invalid `{tag}` tag"))]
  Pair { tag: Fourcc },
  #[snafu(display("invalid box size {size}"))]
  Size { size: u64 },
  #[snafu(display("truncated"))]
  Truncated,
  #[snafu(display("`{tag}` tag is not valid UTF-8"))]
  Utf8 { source: Utf8Error, tag: Fourcc },
  #[snafu(display("unsupported version {version}"))]
  Version { version: u8 },
}
