use std::fmt::Write;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use bytes::{BufMut, Bytes, BytesMut};
use failure::Error;
use lazy_static::lazy_static;
use tokio::codec::{Decoder, Encoder};

#[derive(Debug)]
enum State {
    Header { len: Option<usize> },
    Body { len: usize },
}

impl Default for State {
    fn default() -> Self {
        State::Header { len: None }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Codec {
    state: State,
}

const EOL_BYTES: &[&[u8]] = &[b"\r\n", b"\r", b"\n"];

lazy_static! {
    static ref EOL: AhoCorasick = AhoCorasickBuilder::new()
        .match_kind(aho_corasick::MatchKind::LeftmostLongest)
        .build(EOL_BYTES);
}

impl Decoder for Codec {
    type Item = Bytes;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match self.state {
                State::Header { len } => {
                    let buf = if let Some(m) = EOL.find(&src) {
                        src.split_to(m.end())
                    } else {
                        return Ok(None);
                    };
                    let line = std::str::from_utf8(&buf)?.trim_end();
                    if line.is_empty() {
                        let len = if let Some(i) = len {
                            i
                        } else {
                            failure::bail!("Unknown content length");
                        };
                        self.state = State::Body { len };
                    } else if let Some(i) = line.find(':') {
                        let (name, value) = line.split_at(i);
                        let value = &value[1..].trim_start();
                        match name {
                            "Content-Length" => {
                                let len = Some(value.parse()?);
                                self.state = State::Header { len };
                            }
                            _ => {
                                log::warn!("Unknown header name: {:?}", name);
                            }
                        }
                    } else {
                        log::warn!("Invalid header: {:?}", line);
                    }
                    continue;
                }
                State::Body { len } => {
                    if src.len() < len {
                        return Ok(None);
                    }
                    let buf = src.split_to(len);
                    self.state = State::default();
                    return Ok(Some(buf.into()));
                }
            }
        }
    }
}

impl Encoder for Codec {
    type Item = Bytes;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        write!(dst, "Content-Length: {}\r\n\r\n", item.len())?;
        dst.reserve(item.len());
        dst.put(item);
        Ok(())
    }
}
