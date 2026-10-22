Configuration
=============

Data Directory
--------------

Filepack stores local data, including private keys, in the filepack data
directory, located in `~/.filepack`, or `$XDG_DATA_HOME/filepack` if
`$XDG_DATA_HOME` is set and non-empty.

Filepack stores private keys in the `keychain` subdirectory of the filepack
data directory, by default in `~/.filepack/keychain`.

The location of the filepack data directory used by a command can be overridden
with the `--data-dir` option or the `FILEPACK_DATA_DIR` environment variable.

Logging
-------

Logging can be enabled by setting the `FILEPACK_LOG` environment variable:

```
FILEPACK_LOG=info filepack serve
```

On Linux, when run as a systemd service, logs are written to the systemd
journal instead of standard error.

Log levels, in increasing verbosity, are `off`, `error`, `warn`, `info`,
`debug`, and `trace`:
