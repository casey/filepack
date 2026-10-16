# `{{ self.name }}`

```
{{ self.help }}
```
%% if let Some(input) = self.input {
%%   if self.name == "filepack archive" {

Reads {{ link(Chapter::Json, input) }} as JSON from `<INPUT>`.
%%   } else {
%%     unreachable!()
%%   }
%% }
%% if let Some(output) = self.output {
%%   if self.name == "filepack verify" {

Prints {{ link(Chapter::Json, output) }} as JSON to standard output if `--print` is passed.
%%   } else {

Prints {{ link(Chapter::Json, output) }} as JSON to standard output.
%%   }
%% }
