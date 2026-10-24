Tagged Hex
==========

Tagged hex strings begin with a tag identifying the type, followed by a `1`,
followed by the deco-encoded payload formatted as lowercase hexadecimal.

For example, the tagged hex string of the BLAKE3 hash of the empty string is:

```
hash1af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
```

Note that the encoded payload does not include the outer deco length framing.

The deco encoding of the above hash is:

```
a0af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262
```

Where `a0` encodes the length of the string. The tagged hex representation does
not include this outer framing.

A character in the range `g` through `z` after the `1` separator is a format
version. All format versions are reserved for future use.

Tags
----

| tag | name | type | description |
|-----|------|------|-------------|
%% for (tag, hex) in self.hex {
{{ Row((
  format!("[`{}`](hex/{}.md)", tag.tag(), tag.tag()),
  &hex.name,
  format!("[{}](deco/{}.md)", escape(&hex.schema), hex.schema.to_kebab_case()),
  &hex.description,
)) }}
%% }
