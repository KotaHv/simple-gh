use rocket::{
    http::Status,
    response::{Responder, Response, Result},
    Request,
};

pub struct CustomError {
    reason: String,
    status: Status,
}

impl<'r> Responder<'r, 'static> for CustomError {
    fn respond_to(self, req: &'r Request<'_>) -> Result<'static> {
        Response::build_from(self.reason.respond_to(req)?)
            .status(self.status)
            .ok()
    }
}

impl CustomError {
    pub fn reason(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        CustomError {
            reason,
            status: Status::InternalServerError,
        }
    }

    pub fn new(reason: impl Into<String>, status: Status) -> Self {
        let reason = reason.into();
        CustomError { reason, status }
    }
}
