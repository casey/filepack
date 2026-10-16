use {
  self::{
    chapter::Chapter,
    description::Description,
    functions::{escape, list, parameter, write},
    has_page::HasPage,
    templates::{
      CommandMd, CommandsMd, DecoMd, DecoSchemaMd, HexMd, JsonMd, JsonSchemaMd, LanguagesMd,
      LintsMd, MediaFormatsMd, RouteMd, RoutesMd, SummaryMd, TagMd, UnstableFeaturesMd, YamlMd,
    },
  },
  boilerplate::Boilerplate,
  camino::{Utf8Path, Utf8PathBuf},
  filepack::{
    Count, Language, Lint, LintGroup, MediaType, Reference, Tag, UnstableFeature, deco, json,
    reference::{Command, Hex, MediaFormat, Parameter, Request, Response, Route},
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
  strum::Display,
};

mod chapter;
mod description;
mod functions;
mod has_page;
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
          input: command.input.as_deref(),
          name: &name,
          output: command.output.as_deref(),
          registry: &reference.json,
        },
      );

      commands.insert(name, (command, relative));

      for (name, command) in &command.subcommands {
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

    for (name, schema) in &reference.deco {
      if !schema.has_page() {
        continue;
      }

      let relative = Utf8PathBuf::from(format!("deco/{}.md", name.to_kebab_case()));

      write(
        src.join(&relative),
        DecoSchemaMd {
          name,
          registry: &reference.deco,
          schema,
        },
      );

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

      write(
        src.join(&relative),
        RouteMd {
          path,
          registry: &reference.deco,
          route,
        },
      );

      routes.insert(path.as_str(), relative);
    }

    routes
  };

  let json = {
    let mut pages = BTreeMap::new();

    for (name, schema) in &reference.json {
      if !schema.has_page() {
        continue;
      }

      let relative = Utf8PathBuf::from(format!("json/{}.md", name.to_kebab_case()));

      write(
        src.join(&relative),
        JsonSchemaMd {
          chapter: Chapter::Json,
          name,
          registry: &reference.json,
          schema,
        },
      );

      pages.insert(name.as_str(), relative);
    }

    pages
  };

  let yaml = {
    let mut pages = BTreeMap::new();

    for (name, schema) in &reference.yaml {
      if !schema.has_page() {
        continue;
      }

      let relative = Utf8PathBuf::from(format!("yaml/{}.md", name.to_kebab_case()));

      write(
        src.join(&relative),
        JsonSchemaMd {
          chapter: Chapter::Yaml,
          name,
          registry: &reference.yaml,
          schema,
        },
      );

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
    src.join("json.md"),
    JsonMd {
      json: &reference.json,
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
      json,
      routes,
      yaml,
    },
  );
}
