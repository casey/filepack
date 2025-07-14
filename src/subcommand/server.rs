use super::*;

#[derive(Parser)]
pub(crate) struct Server {
  #[arg(help = "Archive directory containing filepack archives.")]
  archives: Utf8PathBuf,
  #[arg(help = "Port to run the server on.", default_value_t = 80)]
  port: u16,
}

impl Server {
  pub(crate) fn run(self) -> Result {
    Runtime::new()
      .context(error::ServerRuntime)?
      .block_on(async {
        let app = Router::new().route("/", get(|| async { "Hello, World!" }));

        let listener = tokio::net::TcpListener::bind(("0.0.0.0", self.port))
          .await
          .context(error::ServerBind { port: self.port })?;

        axum::serve(listener, app)
          .await
          .context(error::ServerServe)?;

        Ok(())
      })
  }
}
