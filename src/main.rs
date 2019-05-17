extern crate hyper;
extern crate rifling;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use std::env;

use hyper::rt::{self, Future};
use hyper::{Error, Server};
use rifling::{Constructor, Delivery, Hook};

lazy_static! {
    static ref PORT: u16 = env::var("PORT")
        .expect("Expected $PORT environment variable")
        .parse::<u16>()
        .expect("Expected $PORT to be integer");
    static ref SECRET: String = env::var("SECRET").expect("Expected $SECRET environment variable");
}

fn main() {
    env_logger::init();

    let addr = ([127, 0, 0, 1], *PORT).into();

    let mut handler = Constructor::new();
    let hook = Hook::new("*", Some(SECRET.to_string()), |delivery: &Delivery| {
        if let Some(payload) = &delivery.payload {
            if payload["repository"]["full_name"] == "peachcloud/peach-packages"
                && payload["ref"] == "refs/heads/release"
            {
                info!("Packages are released, running build process");
                return run();
            }
        }
        info!("Ignoring web hook");
    });
    handler.register(hook);

    let server = Server::bind(&addr)
        .serve(handler)
        .map_err(|e: Error| error!("Error: {:?}", e));

    rt::run(server);
}

fn run() {
    setup();
    update_repo();
    build_packages();
}

fn setup() {}

fn update_repo() {}

fn build_packages() {}
