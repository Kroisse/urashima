use futures::{channel::mpsc, prelude::*};
use jsonrpc_core::Metadata;
use lsp_types::{LogMessageParams, MessageType, TextDocumentContentChangeEvent, Url};

use crate::prelude::*;

#[derive(Clone)]
pub(crate) struct Mailbox {
    pub(crate) initialized: bool,
    tx: mpsc::Sender<Command>,
}

impl Mailbox {
    pub fn new(tx: mpsc::Sender<Command>) -> Mailbox {
        Mailbox {
            initialized: false,
            tx,
        }
    }

    pub async fn log_message(&mut self, typ: MessageType, msg: impl Into<String>) {
        let msg = Command::LogMessage(LogMessageParams {
            typ,
            message: msg.into(),
        });
        self.send(msg).await;
    }

    pub async fn send(&mut self, msg: Command) {
        match &msg {
            Command::Initialize => {}
            _ if !self.initialized => {
                return;
            }
            _ => {}
        }
        self.tx.send(msg).await;
    }
}

impl Metadata for Mailbox {}

pub(crate) enum Command {
    Initialize,
    FileChanged {
        uri: Url,
        version: Option<u64>,
        changes: Vec<TextDocumentContentChangeEvent>,
    },
    LogMessage(LogMessageParams),
}
