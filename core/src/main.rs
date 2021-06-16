use std::io::{stdin, stdout};

fn main() {
    let req = virtualsnip::read_request(stdin());
    let resp = virtualsnip::calc(&req);
    virtualsnip::write_response(stdout(), &resp);
}
