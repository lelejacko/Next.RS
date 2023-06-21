use crate::{Request, Response};

pub fn handler(req: Request) -> Response {
    Response {
        code: 200,
        headers: None,
        body: Some(format!("Hi from {}", req.path).clone()),
    }
}
