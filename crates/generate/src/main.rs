use {
  self::{
    functions::{escape, parameter, schema, write},
    templates::{CommandMd, HexMd, RouteMd, RoutesMd, SchemaMd, SummaryMd, TagMd},
  },
  boilerplate::Boilerplate,
  camino::{Utf8Path, Utf8PathBuf},
  filepack::{
    Count, Reference, Schema,
    reference::{Parameter, Request, Route, Tag},
    schema::Field,
  },
  heck::ToKebabCase,
  std::{
    collections::BTreeMap,
    env,
    fmt::Display,
    fs::{self, File},
    io::{BufWriter, Write},
    iter,
  },
};

mod functions;
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

  let commands = {
    let mut commands = BTreeMap::new();

    let mut stack = vec![(vec![reference.commands.name.as_str()], &reference.commands)];

    while let Some((path, command)) = stack.pop() {
      let name = path.join(" ");

      let relative = Utf8PathBuf::from(format!("commands/{}.md", path.join("-")));

      write(
        src.join(&relative),
        CommandMd {
          help: &command.help,
          name: &name,
        },
      );

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

    commands
  };

  let schemas = {
    let mut schemas = BTreeMap::new();

    for (name, schema) in &reference.schemas {
      let relative = Utf8PathBuf::from(format!("schemas/{}.md", name.to_kebab_case()));

      write(src.join(&relative), SchemaMd { name, schema });

      schemas.insert(name.as_str(), relative);
    }

    schemas
  };

  let tags = {
    let mut tags = BTreeMap::new();

    for (prefix, tag) in &reference.tags {
      let relative = Utf8PathBuf::from(format!("hex/{prefix}.md"));

      write(src.join(&relative), TagMd { prefix, tag });

      tags.insert(prefix.as_str(), relative);
    }

    tags
  };

  let routes = {
    let mut routes = BTreeMap::new();

    for (path, route) in &reference.routes {
      let slug = if path.ends_with('/') {
        format!("{path}index").to_kebab_case()
      } else {
        path.to_kebab_case()
      };

      let relative = Utf8PathBuf::from(format!("routes/{slug}.md"));

      write(src.join(&relative), RouteMd { path, route });

      routes.insert(path.as_str(), relative);
    }

    routes
  };

  write(
    src.join("hex.md"),
    HexMd {
      tags: &reference.tags,
    },
  );

  write(
    src.join("routes.md"),
    RoutesMd {
      pages: &routes,
      routes: &reference.routes,
    },
  );

  write(
    src.join("SUMMARY.md"),
    SummaryMd {
      commands,
      routes,
      schemas,
      tags,
    },
  );
}
