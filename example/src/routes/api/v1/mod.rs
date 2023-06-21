use crate::{Request, Response};

pub fn handler(req: Request) -> Response {
    Response {
        code: 200,
        body: Some(String::from(format!("Hi from {}", req.path))),
    }
}
