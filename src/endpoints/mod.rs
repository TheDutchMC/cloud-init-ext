use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::http::StatusCode;
use paperclip::actix::OperationModifier;
use futures_util::future::{Ready, ok};
use paperclip::v2::models::{DefaultOperationRaw, Response, Either};
use paperclip::v2::schema::Apiv2Schema;

pub mod crud;
pub mod cloud_init;

pub struct Empty;

impl Responder for Empty {
    type Error = ();
    type Future = Ready<Result<HttpResponse, Self::Error>>;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        ok(HttpResponse::Ok().finish())
    }
}

impl Apiv2Schema for Empty {}

impl OperationModifier for Empty {
    fn update_response(op: &mut DefaultOperationRaw) {
        let status = StatusCode::OK;
        op.responses.insert(
            status.as_str().into(),
            Either::Right(Response {
                description: status.canonical_reason().map(ToString::to_string),
                schema: None,
                ..Default::default()
            }),
        );
    }
}