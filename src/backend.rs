use crate::ServeConfigBuilder;
use dioxus::prelude::*;
use tokio::net::TcpListener;

thread_local! {
    pub static DB: rusqlite::Connection = {
        let conn = rusqlite::Connection::open("./dooT.db3").unwrap();

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS dooT ( id INTEGER PRIMARY KEY, name TEXT NOT NULL, checked BOOLEAN)",
        )
        .unwrap();

        conn
    }
}

pub async fn launch() {
    dioxus::logger::initialize_default();

    let socket_addr = dioxus_cli_config::fullstack_address_or_localhost();

    let router = axum::Router::new()
        .serve_dioxus_application(ServeConfigBuilder::new(), super::dooT)
        .into_make_service();

    let listener = TcpListener::bind(socket_addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
