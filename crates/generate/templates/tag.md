# {{ self.tag.prefix() }}

A tagged hex [{{ escape(&self.hex.schema) }}](../deco/{{ self.hex.schema.to_kebab_case() }}.md).

Example:

```
{{ self.hex.example }}
```
