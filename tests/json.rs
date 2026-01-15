use {crate::EMPTY_HASH, pretty_assertions::assert_eq};

#[macro_export]
macro_rules! json {
  ($($parts:tt)*) => {
    {
      let mut s = String::new();
      parts!(s, {$($parts)*});
      s.push('\n');
      s
    }
  };
}

#[macro_export]
macro_rules! json_pretty {
  ($($parts:tt)*) => {
    {
      let json = json!($($parts)*);
      let value = serde_json::from_str::<serde_json::Value>(&json).unwrap();
      serde_json::to_string_pretty(&value).unwrap() + "\n"
    }
  }
}

#[macro_export]
macro_rules! json_regex {
  ($($parts:tt)*) => {
    {
      json_pretty!($($parts)*).replace('{', "\\{")
    }
  };
}

#[macro_export]
macro_rules! parts {
    ($s:ident, { $($parts:tt)* }) => {{
        $s.push('{');
        parts!($s, $($parts)*);
        $s.push('}');
    }};

    ($s:ident, $key:tt : $value:tt $(, $($rest:tt)*)? ) => {{
        quote!($s, $key);
        $s.push(':');
        parts!($s, $value);

        $(
            $s.push(',');
            let len = $s.len();
            parts!($s, $($rest)*);
            if $s.len() == len {
              $s.pop();
            }
        )?
    }};

    ($s:ident, $value:literal) => {{
        $s.push_str(stringify!($value));
    }};

    ($s:ident, $value:expr) => {{
        $s.push('"');
        $s.push_str(($value).as_ref());
        $s.push('"');
    }};

    ($s:ident,) => {};
}

#[macro_export]
macro_rules! quote {
  ($s:ident, $value:ident) => {
    $s.push('"');
    $s.push_str(stringify!($value));
    $s.push('"');
  };

  ($s:ident, $value:literal) => {
    $s.push_str(stringify!($value));
  };
}

#[test]
fn json() {
  #[track_caller]
  fn case(actual: String, expected: &'static str) {
    assert_eq!(actual, expected.to_owned() + "\n");
  }

  case(json! {}, "{}");

  case(
    json! {
      foo: "a"
    },
    r#"{"foo":"a"}"#,
  );

  case(
    json! {
      foo: "a",
    },
    r#"{"foo":"a"}"#,
  );

  case(
    json! {
      foo: "a",
      bar: "b"
    },
    r#"{"foo":"a","bar":"b"}"#,
  );

  case(
    json! {
      foo: "a",
      bar: "b",
      baz: {
        bob: "c"
      }
    },
    r#"{"foo":"a","bar":"b","baz":{"bob":"c"}}"#,
  );

  case(
    json! {
      files: {
        foo: {
          hash: EMPTY_HASH,
          size: 0
        }
      }
    },
    r#"{"files":{"foo":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0}}}"#,
  );

  case(
    json! {
      files: {
        bar: {
          hash: EMPTY_HASH,
          size: 0
        }
      },
      signatures: {
        "0": "0"
      }
    },
    r#"{"files":{"bar":{"hash":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","size":0}},"signatures":{"0":"0"}}"#,
  );
}
