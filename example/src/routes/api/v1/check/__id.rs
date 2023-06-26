use crate::{Request, Response};

pub fn handler(req: Request) -> Response {
    println!("{:?}", req.dyn_fields);
    Response::from_string(200, None, Some(&format!("Hi from ID {}", req.path)))
}
