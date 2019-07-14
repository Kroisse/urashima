use std::sync::{atomic::AtomicBool, Arc};

use bytes::Bytes;
use chashmap::CHashMap;
use failure::Fallible;
use futures::{channel::mpsc, select};
use lazy_static::lazy_static;
use lsp_types::{MessageType, TextDocumentContentChangeEvent, Url};
use tokio::{
    codec::{FramedRead, FramedWrite},
    io::{AsyncRead, WriteHalf},
    net::TcpStream,
};
use urashima::Runtime;
use urashima_ast::{
    expr::ExprArena,
    program::ScriptProgram,
    span::{Position, Span},
    Find,
};

use crate::{
    codec::Codec,
    command::{Command, Mailbox},
    handler::Handler,
    prelude::*,
    TaskExecutor,
};

pub(crate) type Writer = Compat01As03Sink<FramedWrite<WriteHalf<TcpStream>, Codec>, Bytes>;

pub(crate) struct Workspace {
    handler: Handler,
    buf: Vec<u8>,
    writer: Writer,
    state: Arc<WorkspaceState>,
    executor: TaskExecutor,
}

pub(crate) struct WorkspaceState {
    pub initialized: AtomicBool,
    pub rt: Runtime,
    pub sources: CHashMap<Url, Source>,
    mailbox: Mailbox,
}

#[derive(Debug)]
pub(crate) struct Source {
    version: u64,
    arena: ExprArena,
    ast: Option<ScriptProgram>,
    text: String,
}

lazy_static! {
    static ref DEFAULT_HANDLER: Handler = Handler::new();
}

impl Workspace {
    pub fn new(writer: Writer, mailbox: Mailbox, executor: TaskExecutor) -> Self {
        let state = WorkspaceState {
            initialized: AtomicBool::new(false),
            rt: Runtime::new(),
            sources: CHashMap::new(),
            mailbox,
        };
        Workspace {
            handler: DEFAULT_HANDLER.clone(),
            buf: Vec::new(),
            writer,
            state: Arc::new(state),
            executor,
        }
    }

    pub async fn serve(stream: TcpStream, executor: TaskExecutor) -> Fallible<()> {
        log::debug!("Listen!");
        let (reader, writer) = stream.split();
        let mut reader = FramedRead::new(reader, Codec::default()).compat().fuse();
        let writer = FramedWrite::new(writer, Codec::default()).sink_compat();
        let (tx, mut rx) = mpsc::channel(8);
        let mailbox = Mailbox::new(tx.clone());
        let mut workspace = Workspace::new(writer, mailbox, executor);
        'outer: loop {
            select! {
                x = reader.try_next() => if let Some(request) = x? {
                    let request = std::str::from_utf8(&request)?;
                    workspace.handle_request(request).await?;
                } else {
                    break 'outer Ok(());
                },
                noti = rx.next() => if let Some(command) = noti {
                    workspace.handle_command(command).await?;
                }
            }
        }
    }

    async fn handle_request<'a>(&'a mut self, request: &'a str) -> Fallible<()> {
        if let Some(response) = self
            .handler
            .handle_request(Arc::clone(&self.state), request)
            .await
        {
            self.send_message(&response).await?;
        }
        Ok(())
    }

    async fn handle_command(&mut self, command: Command) -> Fallible<()> {
        use Command::*;
        match command {
            Initialize => {
                // self.mailbox.initialized = true;
            }
            LogMessage(msg) => {
                self.notify::<lsp_notification!("window/logMessage")>(msg)
                    .await?
            }
            FileChanged {
                uri,
                version,
                changes,
            } => {
                // if let Err(e) = self.apply_file_changes(uri, version, changes) {
                // self.mailbox
                //     .log_message(MessageType::Warning, e.to_string())
                //     .await;
                // }
            }
            DocumentHighlight {
                uri,
                position,
                reply,
            } => {
                // let _ = reply.send(self.find_span(uri, position));
            }
        }
        Ok(())
    }

    pub async fn notify<T>(&mut self, params: T::Params) -> Fallible<()>
    where
        T: LspNotification,
        T::Params: Serialize,
    {
        let noti = jsonrpc_core::Notification {
            jsonrpc: Some(jsonrpc_core::Version::V2),
            method: <T as LspNotification>::METHOD.to_owned(),
            params: to_params(&params).ok_or_else(|| RpcError::parse_error())?,
        };
        self.send_message(&noti).await?;
        Ok(())
    }

    async fn send_message<'a, T>(&'a mut self, payload: &'a T) -> Fallible<()>
    where
        T: Serialize,
    {
        self.buf.clear();
        serde_json::to_writer(&mut self.buf, &payload)?;
        self.writer.send(Bytes::from(&self.buf[..])).await?;
        Ok(())
    }
}

impl WorkspaceState {
    pub async fn log_message(&self, typ: MessageType, msg: impl Into<String>) {
        let mut mailbox = self.mailbox.clone();
        mailbox.log_message(typ, msg).await;
    }

    pub fn apply_file_changes(
        &self,
        uri: Url,
        version: Option<u64>,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> urashima::Fallible<()> {
        let mut result = Ok(());
        self.sources.alter(uri.clone(), |entry| {
            let mut entry = if let Some(mut e) = entry {
                if let Some(v) = version {
                    if v <= e.version {
                        return Some(e);
                    }
                    e.version = v;
                }
                e
            } else {
                let text = String::new();
                Source {
                    version: version.unwrap_or(0),
                    arena: ExprArena::new(),
                    ast: None,
                    text,
                }
            };
            for i in changes {
                entry.text = i.text;
            }
            match urashima_ast::parse(&mut entry.arena, &entry.text) {
                Ok(ast) => {
                    entry.ast = Some(ast);
                }
                Err(e) => {
                    result = Err(e.into());
                }
            }
            Some(entry)
        });
        result
    }

    pub fn find_span(&self, uri: Url, pos: Position) -> Option<Span> {
        let src = self.sources.get(&uri)?;
        dbg!(src.ast.as_ref()?.find_span(pos, &src.arena))
    }
}

fn to_params<T>(payload: &T) -> Option<RpcParams>
where
    T: Serialize,
{
    let value = jsonrpc_core::to_value(payload).ok()?;
    Some(RpcParams::deserialize(value).ok()?)
}
