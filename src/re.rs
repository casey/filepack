use super::*;

macro_rules! re {
  { $name:ident, $pattern:literal } => {
    pub(crate) static $name: LazyLock<Regex> =
      LazyLock::new(|| concat!("^", $pattern, "$").parse().unwrap());
  }
}

re! { DATE,       r"\d\d\d\d-\d\d-\d\d"      }
re! { KEY_NAME,   r"[0-9a-z]+(-[0-9a-z]+)*"  }
re! { PUBLIC_KEY, r"public1[a-z0-9]+"        }
re! { TAG,        r"[0-9A-Z]+(\.[0-9A-Z]+)*" }
re! { YEAR,       r"[1-9][0-9]{0,3}"         }
