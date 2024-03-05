use crate::{json_response, ReqMethod, Request, Response};

pub async fn handler<'a>(req: Request<'a>) -> Result<Response, Response> {
    req.allow_methods(vec![ReqMethod::Get])?;

    Ok(json_response!(200, {"message": "Welcome"}))
}
