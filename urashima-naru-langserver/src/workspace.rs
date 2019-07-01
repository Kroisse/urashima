use std::collections::HashMap;

use bytes::Bytes;
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
use urashima_ast::program::ScriptProgram;

use crate::{
    codec::Codec,
    command::{Command, Mailbox},
    handler::Handler,
    prelude::*,
};

pub(crate) type Writer = Compat01As03Sink<FramedWrite<WriteHalf<TcpStream>, Codec>, Bytes>;

pub(crate) struct Workspace {
    rt: Runtime,
    sources: HashMap<Url, Source>,
    handler: Handler,
    buf: Vec<u8>,
    mailbox: Mailbox,
    writer: Writer,
}

#[derive(Debug)]
struct Source {
    version: u64,
    ast: Option<ScriptProgram>,
    text: String,
}

lazy_static! {
    static ref DEFAULT_HANDLER: Handler = Handler::new();
}

impl Workspace {
    pub fn new(writer: Writer, mailbox: Mailbox) -> Self {
        Workspace {
            rt: Runtime::default(),
            sources: HashMap::new(),
            handler: DEFAULT_HANDLER.clone(),
            buf: Vec::new(),
            mailbox,
            writer,
        }
    }

    pub async fn serve(stream: TcpStream) -> Fallible<()> {
        log::debug!("Listen!");
        let (reader, writer) = stream.split();
        let mut reader = FramedRead::new(reader, Codec::default()).compat().fuse();
        let writer = FramedWrite::new(writer, Codec::default()).sink_compat();
        let (tx, mut rx) = mpsc::channel(8);
        let mailbox = Mailbox::new(tx.clone());
        let mut workspace = Workspace::new(writer, mailbox);
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
            .handle_request(self.mailbox.clone(), request)
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
                self.mailbox.initialized = true;
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
                if let Err(e) = self.apply_file_changes(uri, version, changes) {
                    self.mailbox
                        .log_message(MessageType::Warning, e.to_string())
                        .await;
                }
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

    fn apply_file_changes(
        &mut self,
        uri: Url,
        version: Option<u64>,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> urashima::Fallible<()> {
        use std::collections::hash_map::Entry;
        let entry = match self.sources.entry(uri.clone()) {
            Entry::Occupied(e) => {
                let e = e.into_mut();
                if let Some(v) = version {
                    if v <= e.version {
                        return Ok(());
                    }
                    e.version = v;
                }
                for i in changes {
                    e.text = i.text;
                }
                e
            }
            Entry::Vacant(e) => {
                let mut text = String::new();
                for i in changes {
                    text = i.text;
                }
                e.insert(Source {
                    version: version.unwrap_or(0),
                    ast: None,
                    text,
                })
            }
        };
        let mut cap = self.rt.root_capsule();
        let ast = cap.parse_sourcecode::<ScriptProgram>(&entry.text)?;
        entry.ast = Some(ast);
        log::debug!("[{}] {:#?}", uri, entry);
        Ok(())
    }
}

fn to_params<T>(payload: &T) -> Option<RpcParams>
where
    T: Serialize,
{
    let value = jsonrpc_core::to_value(payload).ok()?;
    Some(RpcParams::deserialize(value).ok()?)
}
