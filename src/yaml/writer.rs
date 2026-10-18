use {
  super::*,
  json::{Field, Registry, Schema, Variant},
  std::fmt::Write,
};

const WIDTH: usize = 80;

pub(crate) struct Writer<'a> {
  indent: usize,
  item: bool,
  out: String,
  registry: &'a Registry,
}

impl<'a> Writer<'a> {
  pub(crate) fn blank(&mut self) {
    if !self.out.is_empty() {
      self.out.push('\n');
    }
  }

  fn comment(&mut self, description: &str) {
    for (i, paragraph) in description.split("\n\n").enumerate() {
      if i > 0 {
        self.line("#");
      }

      for line in Self::wrap(paragraph, WIDTH - self.indent - 2) {
        self.line(format_args!("# {line}"));
      }
    }
  }

  pub(crate) fn field(&mut self, field: &Field) {
    match &self.registry[&field.schema] {
      Schema::Array { element } => {
        let Schema::Struct { fields, .. } = &self.registry[element] else {
          unreachable!();
        };

        self.comment(&field.description);

        self.line(format_args!("{}:", field.name));

        self.nested(|writer| writer.item(|writer| writer.fields(fields)));
      }
      Schema::Map { .. } | Schema::Set { .. } | Schema::UntaggedEnum { .. } => {
        unreachable!()
      }
      Schema::Struct { fields, .. } => {
        self.comment(&field.description);

        self.line(format_args!("{}:", field.name));

        self.nested(|writer| writer.fields(fields));
      }
      Schema::TaggedEnum { tag, variants, .. } => {
        self.comment(&field.description);

        self.tagged(field, tag, &variants[0]);
      }
      Schema::Boolean { .. }
      | Schema::Bytes { .. }
      | Schema::Hex { .. }
      | Schema::String { .. }
      | Schema::StringEnum { .. }
      | Schema::UnsignedInteger { .. }
      | Schema::Wrapper { .. } => {
        let example = field
          .example
          .as_ref()
          .unwrap_or_else(|| panic!("field `{}` has no example", field.name));

        let scalar = serde_yaml::to_string(example).unwrap();

        let scalar = scalar.trim_end();

        self.comment(&field.description);

        if self.indent + field.name.len() + 2 + scalar.len() <= WIDTH {
          self.line(format_args!("{}: {scalar}", field.name));
        } else {
          self.line(format_args!("{}: >-", field.name));

          self.nested(|writer| {
            for line in Self::wrap(example, WIDTH - writer.indent) {
              writer.line(line);
            }
          });
        }
      }
    }
  }

  fn fields(&mut self, fields: &[Field]) {
    for field in fields {
      self.field(field);
    }
  }

  pub(crate) fn finish(self) -> String {
    self.out.trim_end().into()
  }

  fn item(&mut self, f: impl FnOnce(&mut Self)) {
    self.indent += 2;
    self.item = true;
    f(self);
    self.indent -= 2;
  }

  fn line(&mut self, text: impl Display) {
    if self.item {
      self.item = false;
      write!(self.out, "{}- ", " ".repeat(self.indent - 2)).unwrap();
    } else {
      write!(self.out, "{}", " ".repeat(self.indent)).unwrap();
    }

    writeln!(self.out, "{text}").unwrap();
  }

  fn nested(&mut self, f: impl FnOnce(&mut Self)) {
    self.indent += 2;
    f(self);
    self.indent -= 2;
  }

  pub(crate) fn new(registry: &'a Registry) -> Self {
    Self {
      indent: 0,
      item: false,
      out: String::new(),
      registry,
    }
  }

  pub(crate) fn placeholder(&mut self, field: &Field) {
    self.comment(&field.description);

    self.line(format_args!("{}: ...", field.name));
  }

  fn tagged(&mut self, field: &Field, tag: &str, variant: &Variant) {
    self.line(format_args!("{}:", field.name));

    self.nested(|writer| {
      writer.comment(&variant.description);
      writer.line(format_args!("{tag}: {}", variant.name));
      writer.fields(&variant.fields);
    });
  }

  pub(crate) fn variant(&mut self, field: &Field, name: &str) {
    let registry = self.registry;

    let Schema::TaggedEnum { tag, variants, .. } = &registry[&field.schema] else {
      unreachable!();
    };

    let variant = variants
      .iter()
      .find(|variant| variant.name == name)
      .unwrap();

    self.tagged(field, tag, variant);
  }

  fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();

    let mut line = String::new();

    for word in text.split_whitespace() {
      if !line.is_empty() && line.len() + 1 + word.len() > width {
        lines.push(std::mem::take(&mut line));
      }

      if !line.is_empty() {
        line.push(' ');
      }

      line.push_str(word);
    }

    if !line.is_empty() {
      lines.push(line);
    }

    lines
  }
}
