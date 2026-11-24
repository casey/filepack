use super::*;

#[test]
fn quiet() {
  let server = Test::new().serve().spawn();

  Test::new()
    .write("foo", "bar")
    .arg("create")
    .success()
    .args(["--quiet", "upload", "--server", &server.address()])
    .success()
    .args(["--quiet", "upload", "--server", &server.address()])
    .success();

  Test::new()
    .write("foo", "bar")
    .arg("create")
    .success()
    .args(["--quiet", "upload", "--server", &server.address()])
    .success();

  server.terminate().success();
}

#[test]
fn restricted_upload_succeeds_with_auth() {
  let server = Test::new()
    .write_key_pair("master")
    .assert_server_file("bar")
    .ready_address()
    .args([
      "serve",
      "--address",
      "127.0.0.1",
      "--http-port",
      "0",
      "--domain",
      "localhost",
      "--admin-key",
      "master",
    ])
    .spawn();

  Test::new()
    .write_key_pair("master")
    .write("foo", "bar")
    .args([
      "upload",
      "--server",
      &server.address(),
      "--auth",
      "master",
      "--file",
      "foo",
    ])
    .success();

  server.terminate().success();
}

#[test]
fn reupload_package_succeeds() {
  let server = Test::new()
    .serve()
    .assert_server_file("aaa")
    .assert_server_file("bbb")
    .assert_server_file("ccc")
    .assert_server_file("ddd")
    .spawn();

  Test::new()
    .write("foo", "aaa")
    .write("bar", "bbb")
    .create_dir("empty")
    .write("sub/baz", "ccc")
    .write("sub/qux", "ddd")
    .arg("create")
    .success()
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(
      "
        uploading 4 of 4 files
        created package number 0
      ",
    )
    .success()
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr("package number 0 is up to date\n")
    .success();

  server.terminate().success();
}

#[test]
fn reupload_succeeds() {
  let server = Test::new().serve().assert_server_file("bar").spawn();

  for _ in 0..2 {
    Test::new()
      .write("foo", "bar")
      .args(["upload", "--server", &server.address(), "--file", "foo"])
      .success();
  }

  server.terminate().success();
}

#[test]
fn serve_admin_key_by_name() {
  let server = Test::new()
    .write_key_pair("master")
    .ready_address()
    .args([
      "serve",
      "--address",
      "127.0.0.1",
      "--http-port",
      "0",
      "--domain",
      "localhost",
      "--admin-key",
      "master",
    ])
    .spawn();

  server
    .put(format!("/api/file/{}", Hash::bytes(b"bar")))
    .body("bar")
    .status(StatusCode::UNAUTHORIZED)
    .send();

  server.terminate().success();
}

#[test]
fn serve_admin_key_by_public_key() {
  let server = Test::new()
    .ready_address()
    .args([
      "serve",
      "--address",
      "127.0.0.1",
      "--http-port",
      "0",
      "--domain",
      "localhost",
      "--admin-key",
      PUBLIC_KEY,
    ])
    .spawn();

  server
    .put(format!("/api/file/{}", Hash::bytes(b"bar")))
    .body("bar")
    .status(StatusCode::UNAUTHORIZED)
    .send();

  server.terminate().success();
}

#[test]
fn upload_auth_requires_https() {
  Test::new()
    .args([
      "upload",
      "--server",
      "http://example.com",
      "--auth",
      "master",
      "--file",
      "foo",
    ])
    .stderr("error: authentication tokens may only be used over HTTPS or loopback\n")
    .failure();
}

#[test]
fn upload_creates_file() {
  let server = Test::new().serve().assert_server_file("bar").spawn();

  Test::new()
    .write("foo", "bar")
    .args(["upload", "--server", &server.address(), "--file", "foo"])
    .success();

  server.terminate().success();
}

#[test]
fn upload_ignores_unrecognized_signatures() {
  let server = Test::new().serve().spawn();

  let test = Test::new()
    .keygen("master")
    .create_dir("foo")
    .args(["create", "--sign", "foo"])
    .success();

  add_unknown_signature(&test.path().join("foo/manifest.filepack"));

  test
    .args(["upload", "--server", &server.address(), "foo"])
    .stderr(
      "
        uploading 0 of 0 files
        created package number 0
        ignored 1 unrecognized signature
        uploaded 1 signature
      ",
    )
    .success();

  server.terminate().success();
}

#[test]
fn upload_no_revision() {
  let server = Test::new().serve().spawn();

  let test = Test::new().write("foo", "bar").arg("create").success();

  let path = test.path().join("manifest.filepack");

  let fingerprint = fingerprint(&path);

  add_parent(&path);

  test
    .args(["upload", "--server", &server.address(), "--no-revision"])
    .stderr(&format!(
      "
        uploading 1 of 1 file
        uploaded {}/package/{fingerprint}
      ",
      server.address(),
    ))
    .assert_absent(".filepack")
    .success();

  server.get(format!("/package/{fingerprint}")).send();

  server
    .get("/package/0")
    .status(StatusCode::NOT_FOUND)
    .send();

  server.terminate().success();
}

#[test]
fn upload_package_accepts_directory() {
  let server = Test::new().serve().assert_server_file("bar").spawn();

  Test::new()
    .write("foo/bar", "bar")
    .args(["create", "foo"])
    .success()
    .args(["upload", "--server", &server.address(), "foo"])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 0
      ",
    )
    .success();

  server.terminate().success();
}

#[test]
fn upload_package_defaults_to_current_directory() {
  let server = Test::new().serve().assert_server_file("bar").spawn();

  Test::new()
    .write("foo", "bar")
    .arg("create")
    .success()
    .args(["upload", "--server", &server.address()])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 0
      ",
    )
    .success();

  server.terminate().success();
}

#[test]
fn upload_package_fails_when_manifest_decode_fails() {
  Test::new()
    .write("manifest.filepack", "not deco")
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "manifest.filepack",
    ])
    .stderr(
      "
        error: failed to decode manifest at `manifest.filepack`
               └─ expected magic bytes `filepack\\x00` but found `n`
      ",
    )
    .failure();
}

#[test]
fn upload_package_fails_when_manifest_missing() {
  Test::new()
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "manifest.filepack",
    ])
    .stderr("error: manifest `manifest.filepack` not found\n")
    .failure();
}

#[test]
fn upload_package_fails_when_package_is_not_directory() {
  let (deco, hash) = Directory::new().insert_file("package", b"foo").deco();

  let mut files = BTreeMap::new();
  files.insert(hash, deco);

  let mut encoder = Encoder::new();
  let mut archive = encoder.map::<u64>();
  archive.item(2, &files);
  archive.item(1, hash);
  archive.finish();
  encoder.magic(MagicType::Archive);

  Test::new()
    .write("manifest.filepack", encoder.finish())
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "manifest.filepack",
    ])
    .stderr(
      "
        error: failed to unarchive manifest
               └─ expected archive `package` entry to be directory but found file
      ",
    )
    .failure();
}

#[test]
fn upload_package_fails_when_package_missing() {
  let mut dir_encoder = Encoder::new();
  let mut map = dir_encoder.map::<u64>();
  map.item(1, BTreeMap::<String, u64>::new());
  map.finish();
  let dir_bytes = dir_encoder.finish();

  let root = Hash::bytes(&dir_bytes);

  let mut files = BTreeMap::new();
  files.insert(root, dir_bytes);

  let mut encoder = Encoder::new();
  let mut archive = encoder.map::<u64>();
  archive.item(2, &files);
  archive.item(1, root);
  archive.finish();
  encoder.magic(MagicType::Archive);

  Test::new()
    .write("manifest.filepack", encoder.finish())
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "manifest.filepack",
    ])
    .stderr(
      "
        error: failed to unarchive manifest
               └─ archive missing package directory
      ",
    )
    .failure();
}

#[test]
fn upload_package_fails_when_root_file_missing() {
  let missing = Hash::bytes(b"missing");

  let mut encoder = Encoder::new();
  let mut archive = encoder.map::<u64>();
  archive.item(2, BTreeMap::<Hash, Vec<u8>>::new());
  archive.item(1, missing);
  archive.finish();
  encoder.magic(MagicType::Archive);

  Test::new()
    .write("manifest.filepack", encoder.finish())
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "manifest.filepack",
    ])
    .stderr(&format!(
      "
        error: failed to unarchive manifest
               └─ archive missing entry for hash {missing}
      ",
    ))
    .failure();
}

#[test]
fn upload_package_fails_when_root_not_directory_deco() {
  let mut text_encoder = Encoder::new();
  text_encoder.text("not a directory");
  let junk = text_encoder.finish();
  let root = Hash::bytes(&junk);

  let mut files = BTreeMap::new();
  files.insert(root, junk);

  let mut encoder = Encoder::new();
  let mut archive = encoder.map::<u64>();
  archive.item(2, &files);
  archive.item(1, root);
  archive.finish();
  encoder.magic(MagicType::Archive);

  Test::new()
    .write("manifest.filepack", encoder.finish())
    .args([
      "upload",
      "--server",
      "http://127.0.0.1:1",
      "manifest.filepack",
    ])
    .stderr(
      "
        error: failed to unarchive manifest
               ├─ failed to decode directory
               └─ truncated
      ",
    )
    .failure();
}

#[test]
fn upload_package_serves_package_html() {
  let server = Test::new().serve().spawn();

  let test = Test::new()
    .write(
      "filepack.yaml",
      "
        title: Foo
        description: Bar
      ",
    )
    .arg("create")
    .success();

  let fingerprint = fingerprint(&test.path().join("manifest.filepack"));

  test
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(
      "
        uploading 2 of 2 files
        created package number 0
      ",
    )
    .success();

  let metadata = Metadata {
    description: Some("Bar".parse().unwrap()),
    title: Some("Foo".parse().unwrap()),
    ..Metadata::default()
  };

  let totals = Totals {
    directories: 0,
    directory_size: 0,
    file_size: metadata.encode_to_vec().len().into_u64() + 28,
    files: 2,
  };

  let root = Hash::from(fingerprint);

  let deco = server.get(format!("/file/{root}")).send();

  let directory = Directory::decode_strict(&deco).unwrap();

  server
    .get(format!("/package/{fingerprint}"))
    .assert_page(PackageHtml {
      directory,
      fingerprint,
      identifier: PackageIdentifier::Fingerprint(fingerprint),
      metadata: Some(metadata),
      mounted: false,
      next: None,
      number: None,
      prev: None,
      parents: Vec::new(),
      revision: None,
      revisions: None,
      signers: BTreeSet::new(),
      totals,
    })
    .send();

  server.terminate().success();
}

#[test]
fn upload_package_skips_directories_already_on_server() {
  let server = Test::new().serve().spawn();

  Test::new()
    .write("foo/bar", "baz")
    .arg("create")
    .success()
    .env("FILEPACK_LOG", "filepack=debug")
    .args(["upload", "--server", &server.address()])
    .stderr_regex_path(
      r"
        uploading 1 of 1 file
        \S+ DEBUG filepack::subcommand::upload: uploading directory `\.`
        \S+ DEBUG filepack::subcommand::upload: uploading directory `\./foo`
        \S+ DEBUG filepack::subcommand::upload: uploading file `\./foo/bar`
        created package number 0
      ",
    )
    .success()
    .write("qux", "quux")
    .arg("update")
    .success()
    .env("FILEPACK_LOG", "filepack=debug")
    .args(["upload", "--server", &server.address()])
    .stderr_regex_path(
      r"
        uploading 1 of 2 files
        \S+ DEBUG filepack::subcommand::upload: uploading directory `\.`
        \S+ DEBUG filepack::subcommand::upload: uploading file `\./qux`
        updated package number 0
      ",
    )
    .success();

  server.terminate().success();
}

#[test]
fn upload_package_skips_files_already_on_server() {
  let server = Test::new()
    .serve()
    .assert_server_file("aaa")
    .assert_server_file("bbb")
    .spawn();

  Test::new()
    .write("foo", "aaa")
    .args(["upload", "--server", &server.address(), "--file", "foo"])
    .success()
    .write("bar", "bbb")
    .arg("create")
    .success()
    .env("FILEPACK_LOG", "filepack=debug")
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr_regex_path(
      r"
        uploading 1 of 2 files
        \S+ DEBUG filepack::subcommand::upload: uploading directory `\.`
        \S+ DEBUG filepack::subcommand::upload: uploading file `\./bar`
        created package number 0
      ",
    )
    .success();

  server.terminate().success();
}

#[test]
fn upload_package_state_case_variant_error() {
  Test::new()
    .arg("create")
    .success()
    .create_dir(".FILEPACK")
    .args(["upload", "--server", "http://127.0.0.1:1"])
    .stderr("error: expected `.filepack` but found case mismatch `.FILEPACK`\n")
    .failure();
}

#[test]
fn upload_package_uploads_files() {
  let server = Test::new()
    .serve()
    .assert_server_file("aaa")
    .assert_server_file("bbb")
    .assert_server_file("ccc")
    .assert_server_file("ddd")
    .assert_file_count("files", 8)
    .spawn();

  let test = Test::new()
    .write("foo", "aaa")
    .write("bar", "bbb")
    .create_dir("empty")
    .write("sub/baz", "ccc")
    .write("sub/qux", "ddd")
    .create_dir("sub/empty")
    .arg("create")
    .success();

  let root = Hash::from(fingerprint(&test.path().join("manifest.filepack")));

  test
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(
      "
        uploading 4 of 4 files
        created package number 0
      ",
    )
    .success();

  let deco = server.get(format!("/file/{root}")).send();

  let directory = Directory::decode_strict(&deco).unwrap();

  server
    .get(format!("/directory/{root}"))
    .assert_page(DirectoryHtml {
      directory,
      hash: root,
    })
    .send();

  server.terminate().success();
}

#[test]
fn upload_records_state() {
  let server = Test::new().serve().spawn();

  Test::new()
    .arg("create")
    .success()
    .args(["--quiet", "upload", "--server", &server.address()])
    .success();

  let test = Test::new().write("foo", "bar").arg("create").success();

  let revision = fingerprint(&test.path().join("manifest.filepack"))
    .revision()
    .hash();

  let test = test
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 1
      ",
    )
    .success()
    .arg("verify")
    .stderr("successfully verified 1 file totaling 3 bytes\n")
    .success();

  assert_eq!(
    State::load(&test.path()).unwrap(),
    State {
      servers: BTreeMap::from([(
        format!("{}/", server.address()),
        ServerState {
          number: 1,
          revision,
        },
      )]),
    },
  );

  server.terminate().success();
}

#[test]
fn upload_recovers_number_from_parent() {
  let server = Test::new().serve().spawn();

  Test::new()
    .write("foo", "bar")
    .arg("create")
    .success()
    .args(["--quiet", "upload", "--server", &server.address()])
    .success()
    .write("foo", "baz")
    .arg("update")
    .success()
    .remove_file(".filepack/state.json")
    .args(["upload", "--server", &server.address()])
    .stderr(
      "
        uploading 1 of 1 file
        updated package number 0
      ",
    )
    .success();

  server.terminate().success();
}

#[test]
fn upload_rejects_ambiguous_number() {
  let server = Test::new().serve().spawn();

  let test = Test::new()
    .write("foo", "bar")
    .arg("create")
    .success()
    .args(["--quiet", "upload", "--server", &server.address()])
    .success();

  Test::new()
    .write("foo", "baz")
    .arg("create")
    .success()
    .args(["--quiet", "upload", "--server", &server.address()])
    .success();

  test
    .args([
      "--quiet",
      "upload",
      "--server",
      &server.address(),
      "--replace",
      "--number",
      "1",
    ])
    .success()
    .remove_file(".filepack/state.json")
    .args(["upload", "--server", &server.address()])
    .stderr(
      "error: server already has this revision as package numbers `0`, `1`, use `--number N` to \
       select one\n",
    )
    .failure();

  server.terminate().success();
}

#[test]
fn upload_rejects_ambiguous_parent() {
  let server = Test::new().serve().spawn();

  let test = Test::new().write("foo", "bar").arg("create").success();

  let parent = fingerprint(&test.path().join("manifest.filepack"))
    .revision()
    .hash();

  let test = test
    .args(["--quiet", "upload", "--server", &server.address()])
    .success();

  Test::new()
    .write("foo", "baz")
    .arg("create")
    .success()
    .args(["--quiet", "upload", "--server", &server.address()])
    .success();

  test
    .args([
      "--quiet",
      "upload",
      "--server",
      &server.address(),
      "--replace",
      "--number",
      "1",
    ])
    .success()
    .write("foo", "qux")
    .arg("update")
    .success()
    .remove_file(".filepack/state.json")
    .args(["upload", "--server", &server.address()])
    .stderr(&format!(
      "error: parent revision {parent} is the head of packages `0`, `1` on `{}/`, use `--number N` \
       to select one\n",
      server.address(),
    ))
    .failure();

  server.terminate().success();
}

#[test]
fn upload_rejects_invalid_signature() {
  let test = Test::new()
    .keygen("master")
    .create_dir("foo")
    .args(["create", "--sign", "foo"])
    .success();

  let path = test.path().join("foo/manifest.filepack");

  let signature = fingerprint(&path).revision().hash();

  let package = add_parent(&path).hash();

  test
    .args(["upload", "--server", "http://127.0.0.1:1", "foo"])
    .stderr(&format!(
      "error: signature revision `{signature}` does not match package revision `{package}`\n"
    ))
    .failure();
}

#[test]
fn upload_rejects_parent_that_is_not_head() {
  let server = Test::new().serve().spawn();

  let test = Test::new().write("foo", "bar").arg("create").success();

  let parent = fingerprint(&test.path().join("manifest.filepack"))
    .revision()
    .hash();

  test
    .args(["--quiet", "upload", "--server", &server.address()])
    .success()
    .args(["delete", "--server", &server.address(), "0"])
    .success()
    .write("foo", "baz")
    .arg("update")
    .success()
    .remove_file(".filepack/state.json")
    .args(["upload", "--server", &server.address()])
    .stderr(&format!(
      "error: parent revision {parent} is not the head of a package on `{}/`\n",
      server.address(),
    ))
    .failure();

  server.terminate().success();
}

#[test]
fn upload_rejects_stale_state() {
  let server = Test::new().serve().spawn();

  let a = Test::new()
    .write("foo", "bar")
    .arg("create")
    .success()
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 0
      ",
    )
    .success();

  let root = fingerprint(&a.path().join("manifest.filepack"))
    .revision()
    .hash();

  let b = Test::new()
    .write("foo", "bar")
    .arg("create")
    .success()
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr("server already has this revision as package number 0\n")
    .success()
    .write("foo", "baz")
    .arg("update")
    .success()
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(
      "
        uploading 1 of 1 file
        updated package number 0
      ",
    )
    .success();

  let head = Revision {
    package: fingerprint(&b.path().join("manifest.filepack")),
    parents: Some(NonEmptyOrderedSet::singleton(root)),
  }
  .hash();

  let a = a.write("foo", "qux").arg("update").success();

  let revision = Revision {
    package: fingerprint(&a.path().join("manifest.filepack")),
    parents: Some(NonEmptyOrderedSet::singleton(root)),
  }
  .hash();

  a.args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(&format!(
      "error: package number 0 is at revision {head} on the server, which is not the parent of \
      revision {revision}\n"
    ))
    .failure();

  server.terminate().success();
}

#[test]
fn upload_rejects_unpublished_parent() {
  let server = Test::new().serve().spawn();

  let test = Test::new().arg("create").success();

  let path = test.path().join("manifest.filepack");

  let parent = fingerprint(&path).revision().hash();

  add_parent(&path);

  let stderr = format!(
    "error: parent revision {parent} has not been uploaded to `{}/`\n",
    server.address(),
  );

  test
    .args(["upload", "--server", &server.address()])
    .stderr(&stderr)
    .failure()
    .args([
      "upload",
      "--server",
      &server.address(),
      "--replace",
      "--number",
      "0",
    ])
    .stderr(&stderr)
    .failure();

  server.terminate().success();
}

#[test]
fn upload_replace_requires_number() {
  Test::new()
    .arg("create")
    .success()
    .args(["upload", "--server", "http://127.0.0.1:1", "--replace"])
    .stderr("error: no package number to replace, use `--number N` to select one\n")
    .failure();
}

#[test]
fn upload_replaces_package() {
  let server = Test::new().serve().spawn();

  let test = Test::new().write("foo", "bar").arg("create").success();

  let old = fingerprint(&test.path().join("manifest.filepack"));

  let test = test
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 0
      ",
    )
    .success()
    .write("foo", "baz")
    .args(["create", "--force"])
    .success();

  let new = fingerprint(&test.path().join("manifest.filepack"));

  test
    .args([
      "upload",
      "--server",
      &server.address(),
      "--replace",
      "manifest.filepack",
    ])
    .stderr(
      "
        uploading 1 of 1 file
        replaced package number 0
      ",
    )
    .success();

  server.get(format!("/package/{new}")).send();

  server.get("/package/0").send();

  Test::new()
    .args(["gc", "--server", &server.address()])
    .stderr("removed 1 revision, 0 signatures, 1 package, 1 directory, and 3 files, freeing 84 B\n")
    .success();

  server
    .get(format!("/package/{old}"))
    .status(StatusCode::NOT_FOUND)
    .send();

  server.terminate().success();
}

#[test]
fn upload_reports_lagging_servers() {
  let server = Test::new().serve().spawn();

  let test = Test::new().write("foo", "bar").arg("create").success();

  let package = fingerprint(&test.path().join("manifest.filepack"));

  let revision = package.revision().hash();

  test
    .write(
      ".filepack/state.json",
      serde_json::to_string(&State {
        servers: BTreeMap::from([(
          "baz".into(),
          ServerState {
            number: 0,
            revision: Revision {
              package,
              parents: Some(NonEmptyOrderedSet::singleton(revision)),
            }
            .hash(),
          },
        )]),
      })
      .unwrap(),
    )
    .args(["upload", "--server", &server.address()])
    .stderr(&format!(
      "
        uploading 1 of 1 file
        created package number 0
        warning: revision {revision} has not yet been uploaded to `baz`
      "
    ))
    .success();

  server.terminate().success();
}

#[test]
fn upload_updates_implicitly() {
  let server = Test::new().serve().spawn();

  let test = Test::new().write("foo", "bar").arg("create").success();

  let root = fingerprint(&test.path().join("manifest.filepack"))
    .revision()
    .hash();

  let test = test
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 0
      ",
    )
    .success()
    .write("foo", "baz")
    .arg("update")
    .success()
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(
      "
        uploading 1 of 1 file
        updated package number 0
      ",
    )
    .success()
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr("package number 0 is up to date\n")
    .success();

  let head = Revision {
    package: fingerprint(&test.path().join("manifest.filepack")),
    parents: Some(NonEmptyOrderedSet::singleton(root)),
  }
  .hash();

  assert_eq!(
    State::load(&test.path()).unwrap(),
    State {
      servers: BTreeMap::from([(
        format!("{}/", server.address()),
        ServerState {
          number: 0,
          revision: head,
        },
      )]),
    },
  );

  server.terminate().success();
}

#[test]
fn upload_updates_package() {
  let server = Test::new().serve().spawn();

  let test = Test::new().write("foo", "bar").arg("create").success();

  let old = fingerprint(&test.path().join("manifest.filepack"));

  let test = test
    .args(["upload", "--server", &server.address(), "manifest.filepack"])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 0
      ",
    )
    .success()
    .args([
      "upload",
      "--server",
      &server.address(),
      "--number",
      "0",
      "manifest.filepack",
    ])
    .stderr("package number 0 is up to date\n")
    .success()
    .write("foo", "baz")
    .arg("update")
    .success();

  let new = fingerprint(&test.path().join("manifest.filepack"));

  test
    .args([
      "upload",
      "--server",
      &server.address(),
      "--number",
      "0",
      "manifest.filepack",
    ])
    .stderr(
      "
        uploading 1 of 1 file
        updated package number 0
      ",
    )
    .success();

  Test::new()
    .args(["gc", "--server", &server.address()])
    .stderr(
      "removed 0 revisions, 0 signatures, 0 packages, 0 directories, and 0 files, freeing 0 B\n",
    )
    .success();

  for identifier in [old.to_string(), new.to_string(), "0".into()] {
    server.get(format!("/package/{identifier}")).send();
  }

  server.terminate().success();
}

#[test]
fn upload_uploads_signatures() {
  let server = Test::new().serve().spawn();

  Test::new()
    .keygen("master")
    .write("foo/bar", "baz")
    .args(["create", "--sign", "foo"])
    .success()
    .args(["upload", "--server", &server.address(), "foo"])
    .stderr(
      "
        uploading 1 of 1 file
        created package number 0
        uploaded 1 signature
      ",
    )
    .success()
    .keygen("bar")
    .args(["sign", "--key", "bar", "foo"])
    .success()
    .args(["upload", "--server", &server.address(), "foo"])
    .stderr(
      "
        package number 0 is up to date
        uploaded 1 signature
      ",
    )
    .success()
    .keygen("baz")
    .args(["sign", "--key", "baz", "foo"])
    .success()
    .remove_file("foo/.filepack/state.json")
    .args(["upload", "--server", &server.address(), "foo"])
    .stderr(
      "
        server already has this revision as package number 0
        uploaded 1 signature
      ",
    )
    .success()
    .args(["upload", "--server", &server.address(), "foo"])
    .stderr("package number 0 is up to date\n")
    .success();

  server.terminate().success();
}
