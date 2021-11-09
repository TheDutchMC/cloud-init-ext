use std::sync::Arc;
use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use futures_util::future::{err, ok, Ready};
use mysql::prelude::Queryable;
use mysql::{Row, params};
use crate::AppData;
use paperclip::actix::Apiv2Security;

pub mod registered_clients;

#[allow(unused)]
#[derive(Apiv2Security)]
#[openapi(
    apiKey,
    in = "header",
    name = "Authorization",
)]
pub struct Auth {
    user_id:    String
}

impl FromRequest for Auth {
    type Error = crate::error::HttpError;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth: String = match req.headers().get("authorization") {
            Some(authorization) => match authorization.to_str() {
                Ok(hv) => hv.to_string(),
                Err(_) => return err(Self::Error::Unauthorized)
            },
            None => return err(Self::Error::Unauthorized)
        };

        let data: &Arc<AppData> = req.app_data().unwrap();
        let mut conn = match data.get_conn() {
            Ok(c) => c,
            Err(e) => {
                return err(Self::Error::Mysql(e));
            }
        };

        let user: Row = match conn.exec_first("SELECT id FROM users WHERE password = :password", params! {
            "password" => &auth
        }) {
            Ok(Some(r)) => r,
            Ok(None) => return err(Self::Error::Unauthorized),
            Err(e) => return err(Self::Error::Mysql(e))
        };

        let user_id: String = user.get("id").unwrap();
        ok(Self { user_id })
    }
}