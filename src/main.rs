extern crate core;
extern crate futures;
extern crate grpcio;
extern crate protobuf;

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use futures::future::{Future, IntoFuture};
use grpcio::{ChannelBuilder, Environment, RpcContext, ServerBuilder, UnarySink};

use helloworld::{HelloReply, HelloRequest};
use helloworld_grpc::{Greeter, GreeterClient};

mod helloworld_grpc;
mod helloworld;

#[derive(Clone)]
struct GreeterService;

impl Greeter for GreeterService {
  fn say_hello(&mut self, ctx: RpcContext, req: HelloRequest, sink: UnarySink<HelloReply>) {
    let now = Instant::now();
    println!("Starting to serve {}", req.get_name());
    let msg = format!("Replying for {}", req.get_name());
    let mut resp = HelloReply::new();
    thread::sleep(Duration::from_millis(700));
    resp.set_message(msg);
    let f = sink
      .success(resp)
      .map_err(move |e| println!("failed to reply {:?}", e));
    ctx.spawn(f);
    println!("Done servicing {}. Took {}ms", req.get_name(), now.elapsed().as_millis());
  }
}

fn start_server() {
  let env = Arc::new(Environment::new(20));
  let service = helloworld_grpc::create_greeter(GreeterService);
  let mut server = ServerBuilder::new(env)
    .register_service(service)
    .bind("127.0.0.1", 50_051)
    .build()
    .unwrap();
  server.start();
  for &(ref host, port) in server.bind_addrs() {
    println!("listening on {}:{}", host, port);
  }
  loop {
    thread::park()
  }
}

fn start_client() {
  let now = Instant::now();
  let async_replies: Vec<_> = (1..20).collect::<Vec<i32>>().iter().map(|index| {
    let env = Arc::new(Environment::new(5));
    let channel = ChannelBuilder::new(env)
      //.primary_user_agent(format!("grpc/test-{}", index).as_str())
      .connect("127.0.0.1:50051");
    let client = GreeterClient::new(channel);
    let mut request = HelloRequest::new();
    request.name = format!("Request {}", index);
    let future = client.say_hello_async(&request)
      .into_future()
      .and_then(|client| client)
      .map(move |res| {
        let _ = client;
        println!("Reply from {}", res.message);
        res.message
      });
    Box::new(future)
  }).collect();
  let result = futures::future::join_all(async_replies)
    .wait()
    .unwrap();
  println!("res {:?}", result);
  println!("Took {}ms", now.elapsed().as_millis());
}

fn main() {
  let component = std::env::var("COMPONENT").unwrap_or(String::from("client"));
  match component.as_str() {
    "server" => start_server(),
    _ => start_client()
  }
}