use {
  self::{
    functions::{deco, escape, list, parameter, write, yaml},
    templates::{
      CommandMd, CommandsMd, DecoMd, DecoSchemaMd, HexMd, LanguagesMd, LintsMd, MediaFormatsMd,
      RouteMd, RoutesMd, SummaryMd, TagMd, UnstableFeaturesMd, YamlMd, YamlSchemaMd,
    },
  },
  boilerplate::Boilerplate,
  camino::{Utf8Path, Utf8PathBuf},
  filepack::{
    Count, Language, Lint, LintGroup, MediaType, Reference, Tag, UnstableFeature, deco,
    reference::{Command, Hex, MediaFormat, Parameter, Request, Response, Route},
    yaml,
  },
  heck::ToKebabCase,
  std::{
    collections::{BTreeMap, BTreeSet},
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

      commands.insert(name, (command, relative));

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

  let deco = {
    let mut deco = BTreeMap::new();

    for (name, schema) in reference.deco.iter() {
      let relative = Utf8PathBuf::from(format!("deco/{}.md", name.to_kebab_case()));

      write(src.join(&relative), DecoSchemaMd { name, schema });

      deco.insert(name.as_str(), relative);
    }

    deco
  };

  let hex = {
    let mut pages = BTreeMap::new();

    for (tag, hex) in &reference.hex {
      let relative = Utf8PathBuf::from(format!("hex/{}.md", tag.prefix()));

      write(src.join(&relative), TagMd { hex, tag: *tag });

      pages.insert(tag.prefix(), relative);
    }

    pages
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

  let yaml = {
    let mut pages = BTreeMap::new();

    for (name, schema) in reference.yaml.iter() {
      let relative = Utf8PathBuf::from(format!("yaml/{}.md", name.to_kebab_case()));

      write(src.join(&relative), YamlSchemaMd { name, schema });

      pages.insert(name.as_str(), relative);
    }

    pages
  };

  write(src.join("commands.md"), CommandsMd { pages: &commands });

  write(
    src.join("hex.md"),
    HexMd {
      hex: &reference.hex,
    },
  );

  write(
    src.join("languages.md"),
    LanguagesMd {
      languages: &reference.languages,
    },
  );

  write(
    src.join("lints.md"),
    LintsMd {
      groups: &reference.lint_groups,
      lints: &reference.lints,
    },
  );

  write(
    src.join("media-formats.md"),
    MediaFormatsMd {
      media_formats: &reference.media_formats,
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
    src.join("deco.md"),
    DecoMd {
      deco: &reference.deco,
    },
  );

  write(
    src.join("unstable-features.md"),
    UnstableFeaturesMd {
      features: &reference.unstable_features,
    },
  );

  write(
    src.join("yaml.md"),
    YamlMd {
      yaml: &reference.yaml,
    },
  );

  write(
    src.join("SUMMARY.md"),
    SummaryMd {
      commands,
      deco,
      hex,
      routes,
      yaml,
    },
  );
}
