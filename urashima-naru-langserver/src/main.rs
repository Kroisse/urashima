#![feature(async_await)]
#![recursion_limit = "256"]

mod codec;
mod command;
mod handler;
mod prelude;
mod workspace;

use std::net::{Ipv4Addr, SocketAddr};

use failure::Fallible;
use futures::{compat::*, prelude::*};
use listenfd::ListenFd;
use tokio::{net::TcpListener, runtime::Runtime};

use crate::workspace::Workspace;

fn main() -> Fallible<()> {
    env_logger::init();
    let mut rt = Runtime::new()?;
    rt.block_on(run().boxed().compat())?;
    Ok(())
}

async fn run() -> Fallible<()> {
    let mut listenfd = ListenFd::from_env();
    let listener = if let Some(listener) = listenfd.take_tcp_listener(0)? {
        TcpListener::from_std(listener, &Default::default())?
    } else {
        let port = std::env::args()
            .skip(1)
            .next()
            .and_then(|a| a.parse().ok())
            .unwrap_or(6464);
        println!("port: {}", port);
        let addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), port);
        TcpListener::bind(&addr)?
    };
    let mut incoming = listener.incoming().compat();
    while let Some(stream) = incoming.try_next().await? {
        tokio::spawn(
            (async move {
                if let Err(e) = Workspace::serve(stream).await {
                    log::error!("{}", e);
                }
            })
                .unit_error()
                .boxed()
                .compat(),
        );
    }
    Ok(())
}
