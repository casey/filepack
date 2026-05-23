use super::*;

const PROGRESS_CHARS: &str = "█▉▊▋▌▍▎▏ ";

const TEMPLATE: &str = "{spinner:.green} ⟪{elapsed_precise}⟫ ⟦{wide_bar:.cyan}⟧ \
                        {binary_bytes}/{binary_total_bytes} \
                        ⟨{binary_bytes_per_sec}, {eta}⟩";

const TEMPLATE_WITH_FILES: &str = "{spinner:.green} ⟪{elapsed_precise}⟫ ⟦{wide_bar:.cyan}⟧ \
                                   {binary_bytes}/{binary_total_bytes} ⟦{msg}⟧ \
                                   ⟨{binary_bytes_per_sec}, {eta}⟩";

const TICK_CHARS: &str = concat!(
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

pub(crate) fn new(options: &Options, bytes: u64) -> ProgressBar {
  if options.quiet {
    ProgressBar::hidden()
  } else {
    ProgressBar::new(bytes).with_style(
      ProgressStyle::default_bar()
        .progress_chars(PROGRESS_CHARS)
        .template(TEMPLATE)
        .unwrap()
        .tick_chars(TICK_CHARS),
    )
  }
}

pub(crate) fn with_files(options: &Options, bytes: u64, files: u64) -> ProgressBar {
  if options.quiet {
    ProgressBar::hidden()
  } else {
    ProgressBar::new(bytes)
      .with_style(
        ProgressStyle::default_bar()
          .progress_chars(PROGRESS_CHARS)
          .template(TEMPLATE_WITH_FILES)
          .unwrap()
          .tick_chars(TICK_CHARS),
      )
      .with_message(format!("0/{files} files"))
  }
}
