use axum::Router;
use std::net::{SocketAddr, ToSocketAddrs};
use tracing::info;

use crate::errors::ServerError;

pub mod errors;

pub async fn get_addr(host: &str, port: u16) -> Result<SocketAddr, ServerError> {
    let addrs = format!("{}:{}", host, port)
        .to_socket_addrs()
        .map_err(|e| ServerError::SocketResolveError(e.to_string()))?
        .collect::<Vec<SocketAddr>>();

    let socket = match addrs.first() {
        Some(addr) => *addr,
        None => {
            return Err(ServerError::SocketNotFound(
                "No socket adresses found".into(),
            ));
        }
    };

    Ok(socket)
}

pub async fn run_server(addr: SocketAddr, router: Router) {
    info!("listening on {addr}");

    axum_server::bind(addr)
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("Failed to start server");
}
