#[macro_use]
extern crate log;

pub mod onu;
pub mod rfc6238;

pub mod bin {
  pub mod server;
}

fn main() {
  bin::server::main();
}
