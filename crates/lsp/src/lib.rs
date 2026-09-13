use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::{Client, LanguageServer, LspService, Server, ls_types};

#[derive(Debug)]
struct Backend {
    client: Client,
}

impl LanguageServer for Backend {
    async fn initialize(
        &self,
        _: ls_types::InitializeParams,
    ) -> Result<ls_types::InitializeResult> {
        Ok(ls_types::InitializeResult {
            capabilities: ls_types::ServerCapabilities {
                ..Default::default()
            },
            server_info: Some(ls_types::ServerInfo {
                name: "ream-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _: ls_types::InitializedParams) {
        self.client
            .log_message(ls_types::MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

pub fn start_lsp() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    else {
        log::error!("Failed to initialize Tokio runtime");

        return;
    };

    let (service, socket) = LspService::new(|client| Backend { client });

    runtime.block_on(Server::new(stdin, stdout, socket).serve(service));
}
