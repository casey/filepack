use {
  super::*,
  indicatif::{ProgressBarIter, ProgressStyle},
};

const COUNT_TEMPLATE: &str = "{spinner:.green} ⟪{elapsed_precise}⟫ ⟦{wide_bar:.cyan}⟧ \
                              {pos}/{len} {msg} ⟨{eta}⟩";

const PROGRESS_CHARS: &str = "=>-";

const TEMPLATE: &str = "{spinner:.green} ⟪{elapsed_precise}⟫ ⟦{wide_bar:.cyan}⟧ \
                        {binary_bytes}/{binary_total_bytes} \
                        ⟨{binary_bytes_per_sec}, {eta}⟩";

const TEMPLATE_WITH_MESSAGE: &str = "{spinner:.green} ⟪{elapsed_precise}⟫ ⟦{wide_bar:.cyan}⟧ \
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

struct Items {
  done: u64,
  noun: &'static str,
  total: u64,
}

impl Display for Items {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}/{} {}", self.done, self.total, self.noun)
  }
}

pub(crate) struct ProgressBar {
  inner: indicatif::ProgressBar,
  items: Option<Items>,
}

impl ProgressBar {
  pub(crate) fn bytes(options: &Options, bytes: u64) -> Self {
    Self {
      inner: Self::inner(options.quiet, bytes, TEMPLATE),
      items: None,
    }
  }

  pub(crate) fn count(quiet: bool, len: u64, message: &'static str) -> Self {
    Self {
      inner: Self::inner(quiet, len, COUNT_TEMPLATE).with_message(message),
      items: None,
    }
  }

  pub(crate) fn inc(&self, delta: u64) {
    self.inner.inc(delta);
  }

  fn inner(quiet: bool, len: u64, template: &str) -> indicatif::ProgressBar {
    if quiet {
      indicatif::ProgressBar::hidden()
    } else {
      indicatif::ProgressBar::new(len).with_style(
        ProgressStyle::default_bar()
          .progress_chars(PROGRESS_CHARS)
          .template(template)
          .unwrap()
          .tick_chars(TICK_CHARS),
      )
    }
  }

  pub(crate) fn item_done(&mut self) {
    self.items.as_mut().unwrap().done += 1;
    self.update_message();
  }

  pub(crate) fn items(options: &Options, bytes: u64, items: u64, noun: &'static str) -> Self {
    let bar = Self {
      inner: Self::inner(options.quiet, bytes, TEMPLATE_WITH_MESSAGE),
      items: Some(Items {
        done: 0,
        noun,
        total: items,
      }),
    };
    bar.update_message();
    bar
  }

  pub(crate) fn set_totals(&mut self, bytes: u64, items: u64) {
    self.inner.set_length(bytes);
    self.items.as_mut().unwrap().total = items;
    self.update_message();
  }

  fn update_message(&self) {
    self
      .inner
      .set_message(self.items.as_ref().unwrap().to_string());
  }

  pub(crate) fn wrap_read<R: Read>(&self, read: R) -> ProgressBarIter<R> {
    self.inner.wrap_read(read)
  }

  pub(crate) fn wrap_write<W: Write>(&self, write: W) -> ProgressBarIter<W> {
    self.inner.wrap_write(write)
  }
}
