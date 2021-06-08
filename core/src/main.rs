use std::{env, io::stdout};

#[tokio::main]
async fn main() {
    let s = {
        let mut args = env::args();
        args.next();
        args.next().unwrap_or_default()
    };
    let req = virtualsnip::deserialize_request(&s);
    let resp = virtualsnip::calc(&req).await;
    virtualsnip::write_response(stdout(), &resp);
}
