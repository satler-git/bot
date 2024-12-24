use hmac::{Hmac, Mac};
use worker::*;

use subtle::ConstantTimeEq;

#[event(fetch, respond_with_errors)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    router.post_async("/webhook", webhook).run(req, env).await
}

async fn webhook(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let webhook_sec: String = ctx.secret("WEBHOOK_SEC")?.to_string();
    let signature: Option<String> = req.headers().get("X-Hub-Signature-256")?;

    let mut req = req;

    if let Some(sig) = signature {
        let body = req.text().await?;

        if !verify_signature(&body, &webhook_sec, &sig[7..] /* sha256= */) {
            return Response::error("Unauthorised (signature did not match)", 403);
        }

        Response::empty() // TODO:
    } else {
        Response::error("Unauthorised (signature does not exit)", 403)
    }
}

fn verify_signature(body: &str, sec: &str, sig: &str) -> bool {
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(sec.as_bytes()).unwrap();

    mac.update(body.as_bytes());

    let result = mac.finalize();

    hex::encode(result.into_bytes())
        .as_bytes()
        .ct_eq(&sig.as_bytes())
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
