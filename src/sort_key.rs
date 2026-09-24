use super::*;

#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SortKey<'a> {
  Count(u64),
  Media(MediaType),
  Text(UniCase<&'a str>, &'a str),
  Year(i64),
}

impl<'a> SortKey<'a> {
  pub(crate) fn compare(
    a: &'a PackageSummary,
    b: &'a PackageSummary,
    sort: Sort,
    order: Order,
  ) -> Ordering {
    let ordering = match (Self::new(a, sort), Self::new(b, sort)) {
      (Some(a), Some(b)) => match order {
        Order::Ascending => a.cmp(&b),
        Order::Descending => b.cmp(&a),
      },
      (Some(_), None) => Ordering::Less,
      (None, Some(_)) => Ordering::Greater,
      (None, None) => Ordering::Equal,
    };

    ordering.then_with(|| a.fingerprint.cmp(&b.fingerprint))
  }

  fn new(summary: &'a PackageSummary, sort: Sort) -> Option<Self> {
    let key = match sort {
      Sort::Creator => Self::text(summary.metadata.as_ref()?.creator.as_ref()?),
      Sort::Files => Self::Count(summary.totals.files),
      Sort::Media => Self::Media(summary.metadata.as_ref()?.media.as_ref()?.ty()),
      Sort::Size => Self::Count(summary.totals.file_size),
      Sort::Title => Self::text(summary.metadata.as_ref()?.title.as_ref()?),
      Sort::Year => Self::Year(summary.metadata.as_ref()?.time?.year()),
    };

    Some(key)
  }

  fn text(text: &'a str) -> Self {
    Self::Text(UniCase::new(text), text)
  }
}
