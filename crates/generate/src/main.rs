use {
  self::templates::{CommandMd, SchemaMd, SummaryMd},
  boilerplate::Boilerplate,
  camino::{Utf8Path, Utf8PathBuf},
  filepack::{Reference, Schema},
  std::{collections::BTreeMap, env, fs, iter},
};

mod templates;

fn main() {
  for (key, _) in env::vars() {
    if key.starts_with("FILEPACK_") {
      unsafe {
        env::remove_var(key);
      }
    }
  }

  let reference = Reference::new();

  let src = Utf8Path::new("book/src");

  let mut stack = vec![(vec![reference.commands.name.as_str()], &reference.commands)];

  let mut commands = BTreeMap::new();

  while let Some((path, command)) = stack.pop() {
    let name = path.join(" ");

    let markdown = CommandMd {
      help: &command.help,
      name: &name,
    }
    .to_string();

    let relative = Utf8PathBuf::from(format!("commands/{}.md", path.join("-")));

    write(&src.join(&relative), &markdown);

    commands.insert(name, relative);

    for (name, command) in &command.subcommands {
      if name == "help" {
        continue;
      }

      stack.push((
        path
          .iter()
          .copied()
          .chain(iter::once(name.as_str()))
          .collect(),
        command,
      ));
    }
  }

  let mut schemas = BTreeMap::new();

  for (name, &schema) in &reference.schemas {
    let relative = Utf8PathBuf::from(format!("schemas/{name}.md"));

    write(&src.join(&relative), &SchemaMd { schema }.to_string());

    schemas.insert(name.clone(), relative);
  }

  write(
    &src.join("SUMMARY.md"),
    &SummaryMd { commands, schemas }.to_string(),
  );
}

fn write(path: &Utf8PathBuf, contents: &str) {
  fs::create_dir_all(path.parent().unwrap()).unwrap();
  fs::write(path, contents).unwrap();
}
