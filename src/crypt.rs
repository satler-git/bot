use hmac::{Hmac, Mac};
use subtle::ConstantTimeEq;

pub fn verify_signature(body: &str, sec: &str, sig: &str) -> bool {
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(sec.as_bytes()).unwrap();

    mac.update(body.as_bytes());

    let result = mac.finalize();

    hex::encode(result.into_bytes())
        .as_bytes()
        .ct_eq(sig.as_bytes())
        .unwrap_u8()
        == 1
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_verify_sig() -> Result<(), Box<dyn std::error::Error>> {
        assert!(super::verify_signature(
            "Hello, World!",
            "It's a Secret to Everybody",
            &"sha256=757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17"[7..]
        ));
        Ok(())
    }
}
