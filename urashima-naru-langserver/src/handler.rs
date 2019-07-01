use std::marker::PhantomData;
use std::sync::Arc;

use futures::future::BoxFuture;
use jsonrpc_core::{
    Compatibility, MetaIoHandler, Metadata, Response, RpcMethod, RpcNotification, Value,
};
use lsp_types::TextDocumentContentChangeEvent;

use crate::{
    command::{Command, Mailbox},
    prelude::*,
};

type IoHandler = MetaIoHandler<Mailbox, inspect::Inspect>;

#[derive(Clone)]
pub(crate) struct Handler {
    handler: Arc<IoHandler>,
}

impl Handler {
    pub fn new() -> Self {
        let mut h = MetaIoHandler::new(Compatibility::V2, inspect::Inspect);

        add_requests(&mut h);
        add_notifications(&mut h);

        Handler {
            handler: Arc::new(h),
        }
    }

    pub async fn handle_request(&self, m: Mailbox, req: impl AsRef<[u8]>) -> Option<Response> {
        let req: jsonrpc_core::Request = match serde_json::from_slice(req.as_ref()) {
            Ok(v) => v,
            Err(_) => {
                return Some(Response::from(
                    RpcError::parse_error(),
                    Some(jsonrpc_core::Version::V2),
                ));
            }
        };
        self.handler
            .handle_rpc_request(req, m)
            .compat()
            .await
            .expect("should not fail")
    }
}

struct Delegate<T>(PhantomData<T>);

impl<R> Delegate<R> {
    fn new() -> Self {
        Delegate(PhantomData)
    }
}

impl<R, S> RpcMethod<S> for Delegate<R>
where
    R: LspRequest + Send + Sync + 'static,
    R::Params: for<'de> serde::Deserialize<'de> + Send + Sync,
    R::Result: serde::Serialize + Send + Sync,
    S: Metadata + Req<R>,
{
    fn call(&self, params: RpcParams, mut meta: S) -> jsonrpc_core::BoxFuture<Value> {
        let params = params.parse::<R::Params>();
        let fut = async move {
            let params = params?;
            let result = meta.call(params).await?;
            match jsonrpc_core::to_value(result) {
                Ok(v) => Ok(v),
                Err(_) => Err(RpcError::parse_error()),
            }
        };
        Box::new(fut.boxed().compat())
    }
}

impl<R, S> RpcNotification<S> for Delegate<R>
where
    R: LspNotification + Send + Sync + 'static,
    R::Params: for<'de> serde::Deserialize<'de> + Send + Sync,
    S: Metadata + Noti<R>,
{
    fn execute(&self, params: RpcParams, mut meta: S) {
        let params = params.parse::<R::Params>();
        let fut = async move {
            let params = params?;
            meta.call(params).await;
            Ok(())
        };
        tokio::spawn(Box::new(
            fut.map_err(|err: RpcError| {
                log::warn!("{}", err);
                ()
            })
            .boxed()
            .compat(),
        ));
    }
}

trait Req<T: LspRequest> {
    fn call(&mut self, params: T::Params) -> BoxFuture<RpcFallible<T::Result>>;
}

trait Noti<T: LspNotification> {
    fn call(&mut self, params: T::Params) -> BoxFuture<()>;
}

macro_rules! impl_requests {
    (@impl_req $method:tt ($self:ident, $params:ident) $body:block) => {
        impl Req<lsp_request!($method)> for Mailbox {
            fn call(
                &mut $self,
                $params: <lsp_request!($method) as LspRequest>::Params,
            ) -> BoxFuture<RpcFallible<<lsp_request!($method) as LspRequest>::Result>> {
                Box::pin(async move {
                    $body
                })
            }
        }
    };

    (@add_requests $($method:tt),*) => {
        fn add_requests(h: &mut IoHandler) {
            $(
            h.add_method_with_meta($method, Delegate::<lsp_request!($method)>::new());
            )*
        }
    };

    ($($method:tt ($self:ident, $params:ident) $body:block)*) => {
        $(
            impl_requests!(@impl_req $method ($self, $params) $body);
        )*
        impl_requests!(@add_requests $($method),*);
    };
}

macro_rules! impl_notifications {
    (@impl_noti $method:tt ($self:ident, $params:ident) $body:block) => {
        impl Noti<lsp_notification!($method)> for Mailbox {
            fn call(
                &mut $self,
                $params: <lsp_notification!($method) as LspNotification>::Params,
            ) -> BoxFuture<()> {
                Box::pin(async move {
                    $body
                })
            }
        }
    };

    (@add_notifications $($method:tt),*) => {
        fn add_notifications(h: &mut IoHandler) {
            $(
            h.add_notification_with_meta($method, Delegate::<lsp_notification!($method)>::new());
            )*
        }
    };

    ($($method:tt ($self:ident, $params:ident) $body:block)*) => {
        $(
            impl_notifications!(@impl_noti $method ($self, $params) $body);
        )*
        impl_notifications!(@add_notifications $($method),*);
    };
}

impl_requests! {

    "initialize"(self, params) {
        self.send(Command::Initialize).await;
        let capabilities = params.capabilities;
        let text_document_sync = Some(lsp_types::TextDocumentSyncCapability::Kind(lsp_types::TextDocumentSyncKind::Full));
        let workspace = Some(lsp_types::WorkspaceCapability {
            workspace_folders: Some(lsp_types::WorkspaceFolderCapability {
                supported: Some(true),
                ..Default::default()
            }),
        });
        Ok(lsp_types::InitializeResult {
            capabilities: lsp_types::ServerCapabilities {
                document_highlight_provider: Some(true),
                text_document_sync,
                workspace,
                ..Default::default()
            },
        })
    }

    "textDocument/documentHighlight"(self, params) {
        Ok(None)
    }

}

impl_notifications! {

    "initialized"(self, _params) {
        self.log_message(lsp_types::MessageType::Log, "Initialized!").await;
    }

    "exit"(self, _params) {
        log::info!("Exit");
    }

    "textDocument/didOpen"(self, params) {
        let doc = params.text_document;
        self.send(Command::FileChanged{
            uri: doc.uri,
            version: Some(doc.version),
            changes: vec![ TextDocumentContentChangeEvent { text: doc.text, range: None, range_length: None }]
        }).await;
    }

    "textDocument/didChange"(self, params) {
        let doc = params.text_document;
        self.send(Command::FileChanged{
            uri: doc.uri,
            version: doc.version,
            changes: params.content_changes,
        }).await;
    }
}

mod inspect {
    use jsonrpc_core::{
        middleware::{NoopCallFuture, NoopFuture},
        Call as RpcCall, Metadata, Middleware,
    };
    use tokio::prelude::future as future01;

    pub(super) struct Inspect;

    impl<M> Middleware<M> for Inspect
    where
        M: Metadata,
    {
        type Future = NoopFuture;
        type CallFuture = NoopCallFuture;

        fn on_call<F, X>(
            &self,
            call: RpcCall,
            meta: M,
            next: F,
        ) -> future01::Either<Self::CallFuture, X>
        where
            F: Fn(RpcCall, M) -> X + Send + Sync,
            X: future01::Future<Item = Option<jsonrpc_core::Output>, Error = ()> + Send + 'static,
        {
            log::debug!("on_call: {:?}", call);
            future01::Either::B(next(call, meta))
        }
    }
}
