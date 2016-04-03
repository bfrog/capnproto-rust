extern crate capnp;
extern crate capnp_rpc;

#[macro_use]
extern crate gj;

use std::rc::Rc;
use std::cell::RefCell;

use capnp::capability::{Server};
use capnp_rpc::{RpcSystem, twoparty, rpc_twoparty_capnp};
use pubsub_capnp::publisher::listener;
use pubsub_capnp::publisher;


use gj::{EventLoop, Promise, TaskReaper, TaskSet};
use gj::io::tcp;

pub mod pubsub_capnp {
  include!(concat!(env!("OUT_DIR"), "/pubsub_capnp.rs"));
}

struct ListenerImpl;

impl publisher::listener::Server for ListenerImpl {
    fn push_values(&mut self,
                   params: publisher::listener::PushValuesParams,
                   _results: publisher::listener::PushValuesResults)
        -> Promise<(), ::capnp::Error>
    {
        println!("got: {}", pry!(params.get()).get_values());
        Promise::ok(())
    }
}

pub fn main() {

    ::gj::EventLoop::top_level(move |wait_scope| {
        use std::net::ToSocketAddrs;
        let addr = try!("127.0.0.1:22222".to_socket_addrs()).next().expect("could not parse address");
        let (reader, writer) = try!(::gj::io::tcp::Stream::connect(addr).wait(wait_scope)).split();
        let network =
            Box::new(twoparty::VatNetwork::new(reader, writer,
                                               rpc_twoparty_capnp::Side::Client,
                                               Default::default()));
        let mut rpc_system = RpcSystem::new(network, None);
        let publisher: publisher::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

        let listener = publisher::listener::ToClient::new(ListenerImpl).from_server::<::capnp_rpc::Server>();

        let mut request = publisher.register_listener_request();
        request.get().set_listener(listener);
        request.send().promise.wait(wait_scope).unwrap();

        Promise::<(),()>::never_done().wait(wait_scope).unwrap();
        Ok(())

    }).expect("top level error");
}