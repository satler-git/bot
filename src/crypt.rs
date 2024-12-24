use hmac::{Hmac, Mac};
use subtle::ConstantTimeEq;

const USER_AGENT: &str = "satler-bot";

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

pub struct GitHubApp {
    pub key: jwt_simple::prelude::RS256KeyPair,
    pub client_id: String,
}

impl GitHubApp {
    pub fn new(key: &str, client_id: &str) -> Self {
        GitHubApp {
            key: jwt_simple::prelude::RS256KeyPair::from_pem(key).unwrap(),
            client_id: client_id.into(),
        }
    }

    fn jwt(&self) -> String {
        use jwt_simple::prelude::*;

        let claim = Claims::create(Duration::from_mins(1)).with_issuer(&self.client_id);

        self.key.sign(claim).unwrap()
    }

    pub async fn token(&self, installation_id: usize) -> Result<String, worker::Error> {
        use reqwest::header;

        #[derive(Debug, serde::Deserialize)]
        struct AccessTokens {
            token: String,
        }

        let jwt = self.jwt();
        let client = reqwest::Client::new();
        let endpoint =
            format!("https://api.github.com/app/installations/{installation_id}/access_tokens");

        let res = client
            .post(endpoint)
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header(header::AUTHORIZATION, format!("Bearer {jwt}"))
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::USER_AGENT, USER_AGENT)
            .send()
            .await
            .map_err(|e| worker::Error::RustError(format!("Error in sending a request: {e}")))?
            .text()
            .await
            .map_err(|e| {
                worker::Error::RustError(format!("Error in reading text from the body: {e}"))
            })?;

        let token: AccessTokens =
            serde_json::from_str(&res).map_err(worker::Error::SerdeJsonError)?;

        Ok(token.token)
    }
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
