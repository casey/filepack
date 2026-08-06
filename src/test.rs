use super::*;

pub(crate) const FINGERPRINT: &str =
  "package1a4uf5nw04lxs6dgzqfh4rdhxffxdukfwf4hq39d7vn2fu4eqlxf3ql7ykr3";

pub(crate) const HASH: &str = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

pub(crate) const PRIVATE_KEY: &str = concat!(
  "private1a67dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyq24p4zsr2nh04f4pkgtxf",
  "zv5yle473x4jue7s6lkwg9tdkk73q59qxqurh4",
);

pub(crate) const PUBLIC_KEY: &str =
  "public1a67dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyqcxnd63";

pub(crate) const SIGNATURE: &str = concat!(
  "signature1a67dndhhmae7p6fsfnj0z37zf78cde6mwqgtms0y87h8ldlvvflyq4uf5nw04lxs6dgzqf",
  "h4rdhxffxdukfwf4hq39d7vn2fu4eqlxf3qqe5zmy0jwfe33a8rr70fk0zv8wgwuy7zqdmp6jdull0l6",
  "kjl9lcxsvmqjz2zqhn92j3enhg9r3gu922j84e54fthhz78anp6cg27wpcrcgx4r",
);

pub(crate) const WEAK_PUBLIC_KEY: &str =
  "public1aqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsqtuc8";

#[track_caller]
pub(crate) fn assert_cbor<T: Debug + Decode + Encode + PartialEq>(value: T, cbor: &str) {
  let buffer = value.encode_to_vec();
  assert_eq!(hex::encode(&buffer), cbor);
  let mut decoder = Decoder::new(&buffer);
  let decoded = T::decode(&mut decoder).unwrap();
  decoder.finish().unwrap();
  assert_eq!(decoded, value);
}

#[track_caller]
pub(crate) fn assert_cbor_eq<T: Debug + Decode + Encode + PartialEq>(
  value: T,
  expected: impl Encode,
) {
  assert_cbor(value, &hex::encode(expected.encode_to_vec()));
}

#[track_caller]
pub(crate) fn assert_encoding<T: Debug + Decode + Encode + PartialEq>(value: T) {
  let buffer = value.encode_to_vec();
  let mut decoder = Decoder::new(&buffer);
  let decoded = T::decode(&mut decoder).unwrap();
  decoder.finish().unwrap();
  assert_eq!(decoded, value);
}

#[track_caller]
pub(crate) fn assert_redb_impls<K>(values: &[K])
where
  K: for<'a> redb::Value<SelfType<'a> = K> + redb::Key + Ord,
{
  for value in values {
    let bytes = K::as_bytes(value);

    if let Some(width) = K::fixed_width() {
      assert_eq!(bytes.as_ref().len(), width);
    }

    assert_eq!(K::from_bytes(bytes.as_ref()), *value);
  }

  for a in values {
    for b in values {
      assert_eq!(
        K::compare(K::as_bytes(a).as_ref(), K::as_bytes(b).as_ref()),
        a.cmp(b),
        "{a:?} vs {b:?}",
      );
    }
  }
}

pub(crate) fn checksum(s: &str) -> String {
  let checked_hrpstring = CheckedHrpstring::new::<bech32::NoChecksum>(s).unwrap();
  checked_hrpstring
    .fe32_iter::<std::vec::IntoIter<u8>>()
    .with_checksum::<bech32::Bech32m>(&checked_hrpstring.hrp())
    .chars()
    .collect()
}

pub(crate) fn exif(orientation: u16) -> Vec<u8> {
  let mut bytes = b"II".to_vec();

  bytes.extend_from_slice(&42u16.to_le_bytes());
  bytes.extend_from_slice(&8u32.to_le_bytes());
  bytes.extend_from_slice(&1u16.to_le_bytes());
  bytes.extend_from_slice(&0x0112u16.to_le_bytes());
  bytes.extend_from_slice(&3u16.to_le_bytes());
  bytes.extend_from_slice(&1u32.to_le_bytes());
  bytes.extend_from_slice(&orientation.to_le_bytes());
  bytes.extend_from_slice(&[0; 2]);
  bytes.extend_from_slice(&0u32.to_le_bytes());

  bytes
}

pub(crate) fn flac(comments: &[&str], samples: u32) -> Vec<u8> {
  let mut bytes = b"fLaC".to_vec();

  bytes.push(if comments.is_empty() { 0x80 } else { 0x00 });
  bytes.extend_from_slice(&34u32.to_be_bytes()[1..]);
  bytes.extend_from_slice(&4096u16.to_be_bytes());
  bytes.extend_from_slice(&4096u16.to_be_bytes());
  bytes.extend_from_slice(&[0; 6]);
  bytes.extend_from_slice(&[0x0a, 0xc4, 0x42, 0xf0]);
  bytes.extend_from_slice(&samples.to_be_bytes());
  bytes.extend_from_slice(&[0; 16]);

  if !comments.is_empty() {
    let mut body = Vec::new();
    body.extend_from_slice(&0u32.to_le_bytes());
    body.extend_from_slice(&u32::try_from(comments.len()).unwrap().to_le_bytes());

    for comment in comments {
      body.extend_from_slice(&u32::try_from(comment.len()).unwrap().to_le_bytes());
      body.extend_from_slice(comment.as_bytes());
    }

    bytes.push(0x84);
    bytes.extend_from_slice(&u32::try_from(body.len()).unwrap().to_be_bytes()[1..]);
    bytes.extend(body);
  }

  bytes.extend_from_slice(&[0; 1024]);

  bytes
}

pub(crate) fn jpeg(width: u32, height: u32) -> Vec<u8> {
  let mut buffer = io::Cursor::new(Vec::new());
  ::image::DynamicImage::new_rgb8(width, height)
    .write_to(&mut buffer, ::image::ImageFormat::Jpeg)
    .unwrap();
  buffer.into_inner()
}

pub(crate) fn jpeg_grayscale(width: u32, height: u32) -> Vec<u8> {
  let mut buffer = io::Cursor::new(Vec::new());
  ::image::DynamicImage::new_luma8(width, height)
    .write_to(&mut buffer, ::image::ImageFormat::Jpeg)
    .unwrap();
  buffer.into_inner()
}

pub(crate) fn jpeg_with_exif(width: u32, height: u32, exif: &[u8]) -> Vec<u8> {
  let buffer = jpeg(width, height);

  let mut app1 = b"Exif\0\0".to_vec();
  app1.extend_from_slice(exif);

  let mut spliced = buffer[..2].to_vec();
  spliced.extend_from_slice(&[0xFF, 0xE1]);
  spliced.extend_from_slice(&u16::try_from(app1.len() + 2).unwrap().to_be_bytes());
  spliced.extend_from_slice(&app1);
  spliced.extend_from_slice(&buffer[2..]);
  spliced
}

pub(crate) fn jpeg_with_sampling(width: u32, height: u32, sampling: u8) -> Vec<u8> {
  let mut bytes = jpeg(width, height);
  let sof = bytes.windows(2).position(|w| w == [0xFF, 0xC0]).unwrap();
  bytes[sof + 11] = sampling;
  bytes
}

pub(crate) fn mp3(tags: &[&str], frames: u32) -> Vec<u8> {
  fn syncsafe(n: usize) -> [u8; 4] {
    let n = u32::try_from(n).unwrap();
    [
      u8::try_from((n >> 21) & 0x7F).unwrap(),
      u8::try_from((n >> 14) & 0x7F).unwrap(),
      u8::try_from((n >> 7) & 0x7F).unwrap(),
      u8::try_from(n & 0x7F).unwrap(),
    ]
  }

  let mut body = Vec::new();

  for tag in tags {
    let (id, value) = tag.split_once('=').unwrap();
    body.extend_from_slice(id.as_bytes());
    body.extend_from_slice(&syncsafe(value.len() + 1));
    body.extend_from_slice(&[0; 2]);
    body.push(3);
    body.extend_from_slice(value.as_bytes());
  }

  let mut bytes = b"ID3".to_vec();
  bytes.extend_from_slice(&[4, 0, 0]);
  bytes.extend_from_slice(&syncsafe(body.len()));
  bytes.extend(body);

  for _ in 0..frames {
    bytes.extend_from_slice(&mp3_frame());
  }

  bytes
}

pub(crate) fn mp3_frame() -> Vec<u8> {
  let mut bytes = vec![0xFF, 0xFB, 0x90, 0x00];
  bytes.resize(417, 0);
  bytes
}

pub(crate) fn png(
  width: u32,
  height: u32,
  color_type: png::ColorType,
  bit_depth: png::BitDepth,
  trns: Option<&[u8]>,
  exif: Option<&[u8]>,
) -> Vec<u8> {
  let mut buffer = Vec::new();

  let mut encoder = png::Encoder::new(&mut buffer, width, height);
  encoder.set_color(color_type);
  encoder.set_depth(bit_depth);

  if color_type == png::ColorType::Indexed {
    encoder.set_palette(vec![0; 3]);
  }

  if let Some(trns) = trns {
    encoder.set_trns(trns.to_vec());
  }

  let mut writer = encoder.write_header().unwrap();

  if let Some(exif) = exif {
    writer.write_chunk(png::chunk::eXIf, exif).unwrap();
  }

  let samples = u32::try_from(color_type.samples()).unwrap();
  let row = (width * samples * u32::from(bit_depth as u8)).div_ceil(8);

  writer
    .write_image_data(&vec![0; usize::try_from(row * height).unwrap()])
    .unwrap();
  writer.finish().unwrap();

  buffer
}

pub(crate) fn png_with_exif(width: u32, height: u32, exif: &[u8]) -> Vec<u8> {
  png(
    width,
    height,
    png::ColorType::Rgb,
    png::BitDepth::Eight,
    None,
    Some(exif),
  )
}

pub(crate) fn tempdir() -> (TempDir, Utf8PathBuf) {
  let tempdir = tempfile::Builder::new()
    .prefix("filepack-test-tempdir")
    .tempdir()
    .unwrap();

  let path = Utf8Path::from_path(tempdir.path()).unwrap().into();

  (tempdir, path)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hash_is_valid() {
    HASH.parse::<Hash>().unwrap();
  }

  #[test]
  fn private_key_is_valid() {
    assert_eq!(
      test::PRIVATE_KEY
        .parse::<PrivateKey>()
        .unwrap()
        .display_secret()
        .to_string(),
      test::PRIVATE_KEY,
    );
  }

  #[test]
  fn signature_matches() {
    let private_key = PRIVATE_KEY.parse::<PrivateKey>().unwrap();
    let statement = Statement {
      fingerprint: FINGERPRINT.parse().unwrap(),
      timestamp: None,
    };
    let signature = private_key.sign(&statement);
    assert_eq!(signature.to_string(), SIGNATURE);
  }
}
