use {
  self::{
    chapter::Chapter,
    description::Description,
    functions::{escape, list, parameter, route_link, route_path, write},
    row::Row,
    templates::{
      CommandMd, CommandsMd, DecoMd, DecoSchemaMd, HexMd, JsonMd, JsonSchemaMd, LanguagesMd,
      LintsMd, MediaFormatsMd, MetadataMd, RouteMd, RoutesMd, SummaryMd, TagMd, UnstableFeaturesMd,
      YamlMd,
    },
  },
  boilerplate::Boilerplate,
  camino::{Utf8Path, Utf8PathBuf},
  filepack::{
    Count, Language, Lint, LintGroup, MediaType, Reference, Tag, UnstableFeature, deco, json,
    reference::{self, Command, Hex, MediaFormat, Parameter, Request, Response, Route},
    yaml,
  },
  heck::ToKebabCase,
  std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fmt::{self, Display, Formatter},
    fs, iter,
  },
  strum::Display,
};

mod chapter;
mod description;
mod functions;
mod row;
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

  let commands = {
    let mut commands = BTreeMap::new();

    let mut stack = vec![(vec![reference.commands.name.as_str()], &reference.commands)];

    while let Some((components, command)) = stack.pop() {
      let name = components.join(" ");

      let path = Utf8PathBuf::from(format!("commands/{}.md", components.join("-")));

      write(
        &path,
        CommandMd {
          help: &command.help,
          input: command.input.as_deref(),
          name: &name,
          output: command.output.as_deref(),
          registry: &reference.json,
        },
      );

      commands.insert(name, (command, path));

      for (name, command) in &command.subcommands {
        stack.push((
          components
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
      if schema.description().is_none() {
        continue;
      }

      let path = Utf8PathBuf::from(format!("deco/{}.md", name.to_kebab_case()));

      write(
        &path,
        DecoSchemaMd {
          name,
          registry: &reference.deco,
          schema,
        },
      );

      deco.insert(name.as_str(), path);
    }

    deco
  };

  let hex = {
    let mut pages = BTreeMap::new();

    for (tag, hex) in &reference.hex {
      let path = Utf8PathBuf::from(format!("hex/{}.md", tag.prefix()));

      write(&path, TagMd { hex, tag: *tag });

      pages.insert(tag.prefix(), path);
    }

    pages
  };

  let routes = {
    let mut routes = BTreeMap::new();

    for (route_path, route) in &reference.routes {
      let slug = if route_path.ends_with('/') {
        format!("{route_path}index").to_kebab_case()
      } else {
        route_path.to_kebab_case()
      };

      let path = Utf8PathBuf::from(format!("routes/{slug}.md"));

      write(
        &path,
        RouteMd {
          path: route_path,
          registry: &reference.deco,
          route,
        },
      );

      routes.insert(route_path.as_str(), path);
    }

    routes
  };

  let json = {
    let mut pages = BTreeMap::new();

    for (name, schema) in &reference.json {
      if schema.description().is_none() {
        continue;
      }

      let path = Utf8PathBuf::from(format!("json/{}.md", name.to_kebab_case()));

      write(
        &path,
        JsonSchemaMd {
          chapter: Chapter::Json,
          name,
          registry: &reference.json,
          schema,
        },
      );

      pages.insert(name.as_str(), path);
    }

    pages
  };

  let yaml = {
    let mut pages = BTreeMap::new();

    for (name, schema) in &reference.yaml {
      if schema.description().is_none() {
        continue;
      }

      let path = Utf8PathBuf::from(format!("yaml/{}.md", name.to_kebab_case()));

      write(
        &path,
        JsonSchemaMd {
          chapter: Chapter::Yaml,
          name,
          registry: &reference.yaml,
          schema,
        },
      );

      pages.insert(name.as_str(), path);
    }

    pages
  };

  write("commands.md", CommandsMd { pages: &commands });

  write(
    "hex.md",
    HexMd {
      hex: &reference.hex,
    },
  );

  write(
    "json.md",
    JsonMd {
      json: &reference.json,
    },
  );

  write(
    "languages.md",
    LanguagesMd {
      languages: &reference.languages,
    },
  );

  write(
    "lints.md",
    LintsMd {
      groups: &reference.lint_groups,
      lints: &reference.lints,
    },
  );

  write(
    "metadata.md",
    MetadataMd {
      example: &reference.metadata,
    },
  );

  write(
    "media-formats.md",
    MediaFormatsMd {
      media_formats: &reference.media_formats,
    },
  );

  write(
    "routes.md",
    RoutesMd {
      pages: &routes,
      routes: &reference.routes,
    },
  );

  write(
    "deco.md",
    DecoMd {
      deco: &reference.deco,
    },
  );

  write(
    "unstable-features.md",
    UnstableFeaturesMd {
      features: &reference.unstable_features,
    },
  );

  write(
    "yaml.md",
    YamlMd {
      yaml: &reference.yaml,
    },
  );

  write(
    "SUMMARY.md",
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
