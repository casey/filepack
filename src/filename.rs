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
  type Err = PathError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let component = s.parse::<Component>()?;

    if component.extension() != Some(T::EXTENSION) {
      todo!()
    }

    Ok(Self {
      component,
      phantom: PhantomData,
    })
  }
}
