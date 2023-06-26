use crate::{Request, Response};

pub fn handler(req: Request) -> Response {
    Response::from_string(200, None, Some(&format!("Hi from ID {}", req.path)))
}
