use super::*;

const PROGRESS_CHARS: &str = "█▉▊▋▌▍▎▏ ";

const TEMPLATE: &str = "{spinner:.green} ⟪{elapsed_precise}⟫ ⟦{wide_bar:.cyan}⟧ \
                        {binary_bytes}/{binary_total_bytes} \
                        ⟨{binary_bytes_per_sec}, {eta}⟩";

pub const TICK_CHARS: &str = concat!(
  "⠀⠁⠈⠉⠂⠃⠊⠋⠐⠑⠘⠙⠒⠓⠚⠛",
  "⠄⠅⠌⠍⠆⠇⠎⠏⠔⠕⠜⠝⠖⠗⠞⠟",
  "⠠⠡⠨⠩⠢⠣⠪⠫⠰⠱⠸⠹⠲⠳⠺⠻",
  "⠤⠥⠬⠭⠦⠧⠮⠯⠴⠵⠼⠽⠶⠷⠾⠿",
  "⡀⡁⡈⡉⡂⡃⡊⡋⡐⡑⡘⡙⡒⡓⡚⡛",
  "⡄⡅⡌⡍⡆⡇⡎⡏⡔⡕⡜⡝⡖⡗⡞⡟",
  "⡠⡡⡨⡩⡢⡣⡪⡫⡰⡱⡸⡹⡲⡳⡺⡻",
  "⡤⡥⡬⡭⡦⡧⡮⡯⡴⡵⡼⡽⡶⡷⡾⡿",
  "⢀⢁⢈⢉⢂⢃⢊⢋⢐⢑⢘⢙⢒⢓⢚⢛",
  "⢄⢅⢌⢍⢆⢇⢎⢏⢔⢕⢜⢝⢖⢗⢞⢟",
  "⢠⢡⢨⢩⢢⢣⢪⢫⢰⢱⢸⢹⢲⢳⢺⢻",
  "⢤⢥⢬⢭⢦⢧⢮⢯⢴⢵⢼⢽⢶⢷⢾⢿",
  "⣀⣁⣈⣉⣂⣃⣊⣋⣐⣑⣘⣙⣒⣓⣚⣛",
  "⣄⣅⣌⣍⣆⣇⣎⣏⣔⣕⣜⣝⣖⣗⣞⣟",
  "⣠⣡⣨⣩⣢⣣⣪⣫⣰⣱⣸⣹⣲⣳⣺⣻",
  "⣤⣥⣬⣭⣦⣧⣮⣯⣴⣵⣼⣽⣶⣷⣾⣿",
  "⣾⣷⣶⣽⣼⣵⣴⣯⣮⣧⣦⣭⣬⣥⣤",
  "⣻⣺⣳⣲⣹⣸⣱⣰⣫⣪⣣⣢⣩⣨⣡⣠",
  "⣟⣞⣗⣖⣝⣜⣕⣔⣏⣎⣇⣆⣍⣌⣅⣄",
  "⣛⣚⣓⣒⣙⣘⣑⣐⣋⣊⣃⣂⣉⣈⣁⣀",
  "⢿⢾⢷⢶⢽⢼⢵⢴⢯⢮⢧⢦⢭⢬⢥⢤",
  "⢻⢺⢳⢲⢹⢸⢱⢰⢫⢪⢣⢢⢩⢨⢡⢠",
  "⢟⢞⢗⢖⢝⢜⢕⢔⢏⢎⢇⢆⢍⢌⢅⢄",
  "⢛⢚⢓⢒⢙⢘⢑⢐⢋⢊⢃⢂⢉⢈⢁⢀",
  "⡿⡾⡷⡶⡽⡼⡵⡴⡯⡮⡧⡦⡭⡬⡥⡤",
  "⡻⡺⡳⡲⡹⡸⡱⡰⡫⡪⡣⡢⡩⡨⡡⡠",
  "⡟⡞⡗⡖⡝⡜⡕⡔⡏⡎⡇⡆⡍⡌⡅⡄",
  "⡛⡚⡓⡒⡙⡘⡑⡐⡋⡊⡃⡂⡉⡈⡁⡀",
  "⠿⠾⠷⠶⠽⠼⠵⠴⠯⠮⠧⠦⠭⠬⠥⠤",
  "⠻⠺⠳⠲⠹⠸⠱⠰⠫⠪⠣⠢⠩⠨⠡⠠",
  "⠟⠞⠗⠖⠝⠜⠕⠔⠏⠎⠇⠆⠍⠌⠅⠄",
  "⠛⠚⠓⠒⠙⠘⠑⠐⠋⠊⠃⠂⠉⠈⠁",
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