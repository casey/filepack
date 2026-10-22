Shell Completion
================

Shell completion scripts for Bash, Elvish, Fish, PowerShell, and Zsh are
available in [release archives](https://github.com/casey/filepack/releases).

The `filepack` binary can also generate the same completion scripts at runtime
using `filepack completions SHELL`:

```sh
filepack completions bash
```

Bash
----

The recommended approach is to use the `bash-completions` package to lazy-load
the completion script:

```bash
mkdir -p ~/.local/share/bash-completion/completions
filepack completions bash > ~/.local/share/bash-completion/completions/filepack
```

If `bash-completions` is not installed, you can source the completion script in
your `.bashrc`:

```bash
source <(filepack completions bash)
```

Elvish
------

In your `rc.elv`:

```elvish
set edit:completion:arg-completer[filepack] = { |@args|
  eval (filepack completions elvish | slurp)
  set @result = (edit:completion:arg-completer[filepack] $@args)
  put $@result
}
```

Fish
----

Save the completion script to the completions directory to lazy-load it:

```fish
mkdir -p ~/.config/fish/completions
filepack completions fish > ~/.config/fish/completions/filepack.fish
```

PowerShell
----------

In your PowerShell `$PROFILE`:

```powershell
filepack completions powershell | Out-String | Invoke-Expression
```

Zsh
---

First save the completion script:

```zsh
mkdir -p ~/.zsh/completions
filepack completions zsh > ~/.zsh/completions/_filepack
```

Then in your `.zshrc`:

```zsh
fpath=(~/.zsh/completions $fpath)
autoload -U compinit
compinit
```
