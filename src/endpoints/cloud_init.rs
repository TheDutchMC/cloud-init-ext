use std::sync::Arc;
use paperclip::actix::{web, post, Apiv2Schema, api_v2_operation};
use serde::Deserialize;
use crate::AppData;
use crate::endpoints::Empty;
use crate::error::Result;

#[derive(Deserialize, Apiv2Schema)]
pub struct CloudInitRequest {
    fqdn:       String,
    hostname:   String,
}

#[post("/cloud-init")]
#[api_v2_operation]
pub async fn cloud_init(data: web::Data<Arc<AppData>>, payload: web::Json<CloudInitRequest>) -> Result<Empty> {
    unimplemented!()
}
