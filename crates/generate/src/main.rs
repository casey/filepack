use {
  boilerplate::Boilerplate,
  camino::Utf8PathBuf,
  filepack::Schema,
  std::{collections::BTreeMap, env, fs, iter},
};

#[derive(Boilerplate)]
struct SummaryMd {
  commands: BTreeMap<String, Utf8PathBuf>,
}

#[derive(Boilerplate)]
struct CommandMd<'a> {
  help: &'a str,
  name: &'a str,
}

fn main() {
  for (key, _) in env::vars() {
    if key.starts_with("FILEPACK_") {
      unsafe {
        env::remove_var(key);
      }
    }
  }

  let schema = Schema::new();

  let mut stack = vec![(vec![schema.commands.name.as_str()], &schema.commands)];

  let mut commands = BTreeMap::new();

  while let Some((path, command)) = stack.pop() {
    let name = path.join(" ");

    let markdown = CommandMd {
      help: &command.help,
      name: &name,
    }
    .to_string();

    let relative = Utf8PathBuf::from(format!("commands/{}.md", path.join("-")));

    let absolute = Utf8PathBuf::from("book/src").join(&relative);

    fs::create_dir_all(absolute.parent().unwrap()).unwrap();

    fs::write(&absolute, markdown).unwrap();

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

  fs::write("book/src/SUMMARY.md", SummaryMd { commands }.to_string()).unwrap();
}
