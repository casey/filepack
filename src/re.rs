use super::*;

macro_rules! re {
  { $name:ident, $pattern:literal } => {
    pub(crate) static $name: LazyLock<Regex> =
      LazyLock::new(|| concat!("^(?:", $pattern, ")$").parse().unwrap());
  }
}

re! { DATE,       r"\d\d\d\d-\d\d-\d\d"      }
re! { KEY_NAME,   r"[0-9a-z]+(-[0-9a-z]+)*"  }
re! { NUMBER,     r"0|[1-9][0-9]*"           }
re! { PUBLIC_KEY, r"public1.*"               }
re! { YEAR,       r"[1-9][0-9]{0,3}"         }
