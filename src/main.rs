pub mod rfc6238;

pub mod bin {
  pub mod server;
}

fn main() {
  bin::server::main();
}
