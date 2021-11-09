use std::sync::Arc;
use serde::Serialize;
use mysql::prelude::Queryable;
use mysql::{Row, PooledConn};
use crate::AppData;
use crate::endpoints::crud::Auth;
use crate::error::Result;
use paperclip::actix::{api_v2_operation, web, get, Apiv2Schema};

#[derive(Serialize, Apiv2Schema)]
pub struct RegisteredClientResponse {
    clients:    Vec<Client>
}

#[derive(Serialize, Apiv2Schema)]
pub struct Client {
    id:         i32,
    ip:         String,
    hostname:   String,
}

/// Get all registered clients
#[get("/crud/registered-clients")]
#[api_v2_operation]
pub async fn registered_clients(_: Auth, data: web::Data<Arc<AppData>>) -> Result<web::Json<RegisteredClientResponse>> {
    let mut conn = data.get_conn()?;
    let clients = get_registered_clients(&mut conn)?;
    Ok(web::Json(RegisteredClientResponse {
        clients
    }))
}

fn get_registered_clients(conn: &mut PooledConn) -> Result<Vec<Client>> {
    let rows: Vec<Row> = conn.exec("SELECT id,ip,hostname FROM registered_clients", mysql::Params::Empty)?;
    let clients: Vec<_> = rows.into_iter()
        .map(|f| {
            Client {
                id: f.get("id").unwrap(),
                ip: f.get("ip").unwrap(),
                hostname: f.get("hostname").unwrap()
            }
        })
        .collect();

    Ok(clients)
}

