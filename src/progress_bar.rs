use super::*;

const PROGRESS_CHARS: &str = "█▉▊▋▌▍▎▏ ";

const TEMPLATE: &str = "{spinner:.green} ⟪{elapsed_precise}⟫ ⟦{wide_bar:.cyan}⟧ \
                        {binary_bytes}/{binary_total_bytes} \
                        ⟨{binary_bytes_per_sec}, {eta}⟩";

/// The tick chars are from the
/// [Braille Patterns unicode block](https://en.wikipedia.org/wiki/Braille_Patterns).
///
/// The chars are ordered to represent the 8-bit numbers in increasing order.
/// Individual braille cells represent bits, with empty cells representing `0`
/// and full cells representing `1`.
///
/// Digits are ordered from least significant to most significant from left to
/// right, and then from top to bottom:
///
/// ```text
/// ╔═════╗
/// ║ 0 1 ║
/// ║ 2 3 ║
/// ║ 4 5 ║
/// ║ 6 7 ║
/// ╚═════╝
/// ```
///
/// todo: fix these
const TICK_CHARS: &str = concat!(
  "⠀⠁⠂⠃⠄⠅⠆⠇⡀⡁⡂⡃⡄⡅⡆⡇", // 0b0000----
  "⠈⠉⠊⠋⠌⠍⠎⠏⡈⡉⡊⡋⡌⡍⡎⡏", // 0b0001----
  "⠐⠑⠒⠓⠔⠕⠖⠗⡐⡑⡒⡓⡔⡕⡖⡗", // 0b0010----
  "⠘⠙⠚⠛⠜⠝⠞⠟⡘⡙⡚⡛⡜⡝⡞⡟", // 0b0011----
  "⠠⠡⠢⠣⠤⠥⠦⠧⡠⡡⡢⡣⡤⡥⡦⡧", // 0b0100----
  "⠨⠩⠪⠫⠬⠭⠮⠯⡨⡩⡪⡫⡬⡭⡮⡯", // 0b0101----
  "⠰⠱⠲⠳⠴⠵⠶⠷⡰⡱⡲⡳⡴⡵⡶⡷", // 0b0110----
  "⠸⠹⠺⠻⠼⠽⠾⠿⡸⡹⡺⡻⡼⡽⡾⡿", // 0b0111----
  "⢀⢁⢂⢃⢄⢅⢆⢇⣀⣁⣂⣃⣄⣅⣆⣇", // 0b1000----
  "⢈⢉⢊⢋⢌⢍⢎⢏⣈⣉⣊⣋⣌⣍⣎⣏", // 0b1001----
  "⢐⢑⢒⢓⢔⢕⢖⢗⣐⣑⣒⣓⣔⣕⣖⣗", // 0b1010----
  "⢘⢙⢚⢛⢜⢝⢞⢟⣘⣙⣚⣛⣜⣝⣞⣟", // 0b1011----
  "⢠⢡⢢⢣⢤⢥⢦⢧⣠⣡⣢⣣⣤⣥⣦⣧", // 0b1100----
  "⢨⢩⢪⢫⢬⢭⢮⢯⣨⣩⣪⣫⣬⣭⣮⣯", // 0b1101----
  "⢰⢱⢲⢳⢴⢵⢶⢷⣰⣱⣲⣳⣴⣵⣶⣷", // 0b1110----
  "⢸⢹⢺⢻⢼⢽⢾⢿⣸⣹⣺⣻⣼⣽⣾⣿", // 0b1111----
);

pub(crate) fn new(bytes: u64) -> ProgressBar {
  ProgressBar::new(bytes).with_style(
    ProgressStyle::default_bar()
      .progress_chars(PROGRESS_CHARS)
      .template(TEMPLATE)
      .unwrap()
      .tick_chars(TICK_CHARS),
  )
}
