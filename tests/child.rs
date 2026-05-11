use super::*;

pub(crate) struct Child {
  child: Option<std::process::Child>,
  test: Option<Test>,
}

impl Child {
  pub(crate) fn new(child: std::process::Child, test: Test) -> Self {
    Self {
      child: Some(child),
      test: Some(test),
    }
  }

  #[track_caller]
  pub(crate) fn success(mut self) -> Test {
    let (child, test) = self.take();

    let output = child.wait_with_output().unwrap();

    test.foo(0, output)
  }

  pub(crate) fn take(&mut self) -> (std::process::Child, Test) {
    (self.child.take().unwrap(), self.test.take().unwrap())
  }
}

impl Drop for Child {
  fn drop(&mut self) {
    if let Some(child) = self.child.as_mut() {
      child.kill().unwrap();
      child.wait().unwrap();
    }
  }
}
