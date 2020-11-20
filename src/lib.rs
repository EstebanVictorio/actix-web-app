#[macro_use]
extern crate actix_web;

use actix_web::{ middleware, web, App, HttpRequest, HttpServer, Result };
use serde::Serialize;
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

static SERVER_COUNTER: AtomicUsize = AtomicUsize::new(0);



struct AppState {
  server_id: usize,
  request_count: Cell<usize>,
  messages: Arc<Mutex<Vec<String>>>,
}
pub struct MessageApp {
  port: u16,
}

#[derive(Serialize)]
struct IndexResponse {
  message: String,
}


#[get("/")]
fn index(req: HttpRequest) -> Result<web::Json<IndexResponse>> {
  let hello =
    req
      .headers()
      .get("hello")
      .and_then(|v| v.to_str().ok())
      .unwrap_or_else(|| "world");
  
  Ok(web::Json(IndexResponse {
    message: hello.to_owned(),
  }))
}

impl MessageApp {
  pub fn new(port: u16) -> Self {
    MessageApp { port, }
  }

  pub fn run(&self) -> std::io::Result<()> {
    println!("Starting listening on: http://localhost:{}", self.port);
    HttpServer::new(move || {
      App::new()
        .wrap(middleware::Logger::default())
        .service(index)
    })
    .bind(("localhost", self.port))?
    .workers(8)
    .run()
  }
}