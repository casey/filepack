use super::*;

pub(crate) use self::{
  atom::Atom, atoms::Atoms, avc1::Avc1, avcc::Avcc, box_header::BoxHeader, container::Container,
  data::Data, esds::Esds, fourcc::Fourcc, ftyp::Ftyp, hdlr::Hdlr, ilst::Ilst, ilst_item::IlstItem,
  mdhd::Mdhd, mdia::Mdia, meta::Meta, minf::Minf, moov::Moov, mp4a::Mp4a, mvhd::Mvhd, parse::Parse,
  reader::Reader, sample_entry::SampleEntry, stbl::Stbl, stsd::Stsd, stsz::Stsz, stts::Stts,
  stts_entry::SttsEntry, tkhd::Tkhd, trak::Trak, udta::Udta,
};

mod atom;
mod atoms;
mod avc1;
mod avcc;
mod box_header;
mod container;
mod data;
mod esds;
mod fourcc;
mod ftyp;
mod hdlr;
mod ilst;
mod ilst_item;
mod mdhd;
mod mdia;
mod meta;
mod minf;
mod moov;
mod mp4a;
mod mvhd;
mod parse;
mod reader;
mod sample_entry;
mod stbl;
mod stsd;
mod stsz;
mod stts;
mod stts_entry;
mod tkhd;
mod trak;
mod udta;

#[derive(Debug, PartialEq)]
pub(crate) struct Mp4 {
  pub(crate) ftyp: Ftyp,
  pub(crate) moov: Moov,
}

impl Mp4 {
  pub(crate) fn parse(bytes: &[u8]) -> Result<Self, Mp4Error> {
    Self::read(io::Cursor::new(bytes), bytes.len().into_u64())
  }

  pub(crate) fn read<R: Read + Seek>(mut reader: R, len: u64) -> Result<Self, Mp4Error> {
    fn read<R: Read, const N: usize>(
      reader: &mut R,
      offset: u64,
      len: u64,
    ) -> Result<[u8; N], Mp4Error> {
      ensure!(len - offset >= N.into_u64(), mp4_error::Truncated);
      let mut bytes = [0; N];
      reader.read_exact(&mut bytes).context(mp4_error::Io)?;
      Ok(bytes)
    }

    let mut bodies = Vec::<(Fourcc, Vec<u8>)>::new();

    let mut offset = 0;

    while offset < len {
      let mut bytes = read::<R, 8>(&mut reader, offset, len)?.to_vec();

      if bytes[..4] == [0, 0, 0, 1] {
        bytes.extend_from_slice(&read::<R, 8>(&mut reader, offset + 8, len)?);
      }

      let header = BoxHeader::parse(&mut Reader::new(&bytes))?;

      let start = offset + header.len;

      let end = match header.size {
        None => len,
        Some(size) => offset.checked_add(size).context(mp4_error::Truncated)?,
      };

      ensure!(end <= len, mp4_error::Truncated);

      if header.ty == Ftyp::TYPE || header.ty == Moov::TYPE {
        let mut body = vec![0; usize::try_from(end - start).unwrap()];
        reader.read_exact(&mut body).context(mp4_error::Io)?;
        bodies.push((header.ty, body));
      } else {
        reader.seek(SeekFrom::Start(end)).context(mp4_error::Io)?;
      }

      offset = end;
    }

    let container = Container::new(
      bodies
        .iter()
        .map(|(ty, body)| Atom { body, ty: *ty })
        .collect(),
    );

    Ok(Self {
      ftyp: container.one()?,
      moov: container.one()?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse() {
    #[track_caller]
    fn error(bytes: &[u8], expected: &str) {
      assert_eq!(
        Mp4::parse(bytes)
          .unwrap_err()
          .iter_chain()
          .map(ToString::to_string)
          .collect::<Vec<String>>()
          .join(": "),
        expected,
      );
    }

    let mp4 = Mp4::parse(&Mp4Builder::new().build()).unwrap();

    assert_eq!(mp4.ftyp, Ftyp);
    assert_eq!(mp4.moov.mvhd.timescale, 1000);

    let bytes = [
      Mp4Builder::atom(*b"ftyp", &[0; 8]),
      Mp4Builder::atom(*b"free", b"foo"),
      Mp4Builder::atom(*b"mdat", b"bar"),
      Mp4Builder::new().build()[20..].to_vec(),
    ]
    .concat();

    assert_eq!(Mp4::parse(&bytes).unwrap().moov.mvhd.timescale, 1000);

    let mut moov = Mp4Builder::new().build()[20..].to_vec();
    moov[..4].copy_from_slice(&[0; 4]);

    assert_eq!(
      Mp4::parse(&[Mp4Builder::atom(*b"ftyp", &[0; 8]), moov].concat())
        .unwrap()
        .moov
        .mvhd
        .timescale,
      1000,
    );

    error(b"foo", "truncated");

    error(b"fLaC\0\0\0\x22", "truncated");

    error(&Mp4Builder::atom(*b"ftyp", &[0; 8]), "missing `moov` box");

    error(&Mp4Builder::new().build()[20..], "missing `ftyp` box");

    error(
      &[
        Mp4Builder::new().build(),
        Mp4Builder::atom(*b"ftyp", &[0; 8]),
      ]
      .concat(),
      "duplicate `ftyp` box",
    );

    error(
      &[
        Mp4Builder::atom(*b"ftyp", &[0; 8]),
        Mp4Builder::atom(*b"moov", b""),
      ]
      .concat(),
      "invalid `moov` box: missing `mvhd` box",
    );

    error(
      &[
        Mp4Builder::atom(*b"ftyp", &[0; 8]),
        vec![
          0, 0, 0, 1, b'm', b'o', b'o', b'v', 0, 0, 0, 0, 0, 0, 0, 0xff,
        ],
      ]
      .concat(),
      "truncated",
    );
  }
}
