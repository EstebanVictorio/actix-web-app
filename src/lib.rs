#[macro_use]
extern crate actix_web;

use actix_web::{middleware, web, App, HttpRequest, HttpServer, Result};
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

static SERVER_COUNTER: AtomicUsize = AtomicUsize::new(0);

struct AppState {
  server_id: usize,
  request_count: Cell<usize>,
  messages: Arc<Mutex<Vec<String>>>,
}

#[derive(Deserialize)]
struct PostInput {
  message: String,
}

#[derive(Serialize)]
struct PostResponse {
  server_id: usize,
  request_count: usize,
  message: String,
}

pub struct MessageApp {
  port: u16,
}

#[derive(Serialize)]
struct IndexResponse {
  server_id: usize,
  request_count: usize,
  messages: Vec<String>,
}

fn post(msg: web::Json<PostInput>, state: web::Data<AppState>) -> Result<web::Json<PostResponse>> {
  let request_count = state.request_count.get() + 1;
  state.request_count.set(request_count);
  let mut ms = state.messages.lock().unwrap();
  ms.push(msg.message.clone());

  Ok(web::Json(PostResponse {
    request_count,
    server_id: state.server_id,
    message: msg.message.clone(),
  }))
}

#[delete("/clear")]
fn clear(state: web::Data<AppState>) -> Result<web::Json<IndexResponse>> {
  let request_count = state.request_count.get() + 1;
  state.request_count.set(request_count);
  let mut ms = state.messages.lock().unwrap();
  ms.clear();

  Ok(web::Json(IndexResponse {
    request_count,
    server_id: state.server_id,
    messages: vec![],
  }))
}

// NOTE: last page - 87 at "Receiving input"
#[get("/")]
fn index(state: web::Data<AppState>) -> Result<web::Json<IndexResponse>> {
  let request_count = state.request_count.get() + 1;
  state.request_count.set(request_count);
  let ms = state.messages.lock().unwrap(); // RAII (Resource Acquisition Is Initialization)
  Ok(web::Json(IndexResponse {
    server_id: state.server_id,
    request_count,
    messages: ms.clone(),
  }))
}

impl MessageApp {
  pub fn new(port: u16) -> Self {
    MessageApp { port }
  }

  pub fn run(&self) -> std::io::Result<()> {
    println!("Starting listening on: http://localhost:{}", self.port);
    let messages = Arc::new(Mutex::new(vec![]));
    HttpServer::new(move || {
      App::new()
        .data(AppState {
          server_id: SERVER_COUNTER.fetch_add(1, Ordering::SeqCst),
          request_count: Cell::new(0),
          messages: messages.clone(),
        })
        .wrap(middleware::Logger::default())
        .service(index)
        .service(
          web::resource("/send")
            .data(web::JsonConfig::default().limit(4096))
            .route(web::post().to(post)),
        )
        .service(clear)
    })
    .bind(("localhost", self.port))?
    .workers(8)
    .run()
  }
}
