mod rfc6238;

fn main() {
  let secret = b"your_secret_key";
  let counter = rfc6238::time_tick();

  println!("KEY: {}", rfc6238::base32_encode(secret));

  match rfc6238::generate_otp(secret, counter) {
    Ok(code) => println!("OTP: {} ({})", code, counter),
    Err(err) => println!("Error: {}", err),
  }
}
