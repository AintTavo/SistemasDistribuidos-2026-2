// Librerias std
use std::net::SocketAddr;

// Crates
use axum::{
    routing::{get, post, put, delete},
    Json,
    Router,
};

use serde::{
    Deserialize, 
    Serialize,
};

use colored::Colorize;

/*
#[derive(Serialize, Deserialize)]
struct PetitionGet {
    id : u64,
    username : String,

}

struct PetitionPut {
    id : u64,
    username : String,

}
*/


#[tokio::main]
async fn main() {

    // Definicion de comandos de API
    let app = Router::new()
        .route("/", get(|| async {"Petición get"}))
        .route("/", post(|| async {"Petición post"}))
        .route("/", put(|| async {"Petición put"}))
        .route("/", delete(|| async {"Petición delete"}));

    // Definicion de dirección del servidor
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let _tmp_addr = addr.to_string().yellow();
    println!("Servidor corriendo en {}{}","https://".yellow().bold(), _tmp_addr.yellow().bold());

    // Lanazr servidor
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await
        .unwrap();
}

async fn shutdown(){
    tokio::signal::ctrl_c()
    .await
    .expect("No se pudo instalar el manejador de Ctrl+C");
}