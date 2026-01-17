use super::*;

macro_rules! filename {
  { $alias:ident, $extension:ident, $literal:literal } => {
    #[derive(Clone, Debug, PartialEq)]
    pub(crate) struct $extension;

    impl Extension for $extension {
      const EXTENSION: &str = $literal;
    }

    pub(crate) type $alias = Filename<$extension>;
  }
}

filename! { Png, PngExtension, "png" }

filename! { Nfo, NfoExtension, "nfo" }

filename! { Md, MdExtension, "md" }

pub(crate) trait Extension {
  const EXTENSION: &str;
}

#[derive(Clone, Debug, DeserializeFromStr, PartialEq)]
pub(crate) struct Filename<T: Extension> {
  component: Component,
  phantom: PhantomData<T>,
}

impl<T: Extension> Filename<T> {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.component.as_path()
  }
}

impl<T: Extension> FromStr for Filename<T> {
  type Err = ComponentError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let component = s.parse::<Component>()?;

    if component.extension() != Some(T::EXTENSION) {
      return Err(ComponentError::Extension {
        extension: T::EXTENSION,
      });
    }

    Ok(Self {
      component,
      phantom: PhantomData,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn valid() {
    #[track_caller]
    fn case<T: FromStr<Err = ComponentError>>(input: &str) {
      input.parse::<T>().unwrap();
    }

    case::<Md>("README.md");
    case::<Nfo>("info.nfo");
    case::<Png>("cover.png");
  }

  #[test]
  fn invalid() {
    #[track_caller]
    fn case<T: FromStr<Err = ComponentError> + std::fmt::Debug>(
      input: &str,
      expected: ComponentError,
    ) {
      assert_eq!(input.parse::<T>().unwrap_err(), expected);
    }

    case::<Md>("README.txt", ComponentError::Extension { extension: "md" });
    case::<Nfo>("info.txt", ComponentError::Extension { extension: "nfo" });
    case::<Png>("", ComponentError::Empty);
    case::<Png>("cover.jpg", ComponentError::Extension { extension: "png" });
    case::<Png>("foo/bar.png", ComponentError::Separator { character: '/' });
  }
}
