pub mod controller;

pub mod utils;

use actix_web::{web, App, HttpServer};

use controller::accept_proposal_secondry::secondry_index;

use controller::accept_proposal::index;

use controller::receive_signal::receive_signal;

use controller::health::health_check;

use controller::receive_vote::vote;

use utils::ips::add_node;

use std::env;

use dotenv;

fn read_nodes_from_env() -> Vec<String> {
    match env::var("NODES") {
        Ok(nodes_str) => nodes_str
            .split(',') 
            .map(|node| node.trim().to_string())
            .collect(),
        Err(_) => Vec::new(), 
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let environment = env::var("ENV").unwrap_or_else(|_| "dev".to_string());
    if environment == "dev" {
        dotenv::from_filename("node.env").ok();
    }
    let port = env::var("PORT").expect("Failed to load the Port !!");
    let ips = read_nodes_from_env();
    for ip in ips {
        add_node(ip);
    }
    HttpServer::new(|| 
        App::new()
                .route("/proposal", web::post().to(index))
                .route("/running",web::head().to(health_check))
                .route("/node", web::post().to(secondry_index))
                .route("/vote", web::post().to(vote))
                .route("/signal", web::post().to(receive_signal))
    )
    .bind(("0.0.0.0", port.parse::<u16>().unwrap()))?
    .run()
    .await
}