use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{digest::FixedOutput, Hmac, Mac};

pub fn base32_encode(secret: &[u8]) -> String {
  base32::encode(base32::Alphabet::RFC4648 { padding: true }, secret)
}

pub fn time_tick() -> u64 {
  match SystemTime::now().duration_since(UNIX_EPOCH) {
    Ok(n) => n.as_secs() / 30,
    Err(_) => 0,
  }
}

pub fn generate_otp(secret: &[u8], counter: u64) -> Result<String, &'static str> {
  let mut mac = Hmac::<sha1::Sha1>::new_from_slice(secret).unwrap();
  let counter_bytes = counter.to_be_bytes();

  // Create a message consisting of counter bytes
  let message = &counter_bytes;

  // Sign the message using the HMAC-SHA1 key
  mac.update(message);
  let signature = mac.finalize_fixed().to_vec();

  // Extract the relevant bytes for the OTP
  let offset = (signature[signature.len() - 1] & 0x0f) as usize;
  let code = ((signature[offset] as u32 & 0x7f) << 24) |
        ((signature[offset + 1] as u32 & 0xff) << 16) |
        ((signature[offset + 2] as u32 & 0xff) << 8) |
        (signature[offset + 3] as u32 & 0xff);

  // Truncate the code to 6 digits
  let code = code % 1000000;

  Ok(format!("{:06}", code))
}

#[test]
fn test_base32_encode() {
  assert_eq!(base32_encode(b"hello"), "NBSWY3DP");

  let secret = b"your_secret_key";
  let counter = 123456;

  assert_eq!(base32_encode(secret), "PFXXK4S7ONSWG4TFORPWWZLZ");

  assert_eq!(generate_otp(secret, counter), Ok("811986".to_string()));
}
