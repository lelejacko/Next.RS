use crate::{ReqMethod, Request, Response};

pub async fn handler(req: Request) -> Result<Response, Response> {
    req.allow_methods(vec![ReqMethod::Get])?;

    Ok(Response::from_string(
        200,
        None,
        Some(&format!("Hi from {}", req.path)),
    ))
}
