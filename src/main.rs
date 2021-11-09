#[macro_use]
extern crate lazy_static;

mod error;
mod endpoints;
mod functionality;
mod apis;

use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use log::{info, debug, error};
use std::fs;
use std::process::{Command, exit};
use std::sync::Arc;
use actix_governor::GovernorConfigBuilder;
use actix_web::{App, HttpServer};
use actix_web::middleware::normalize::TrailingSlash;
use paperclip::actix::{OpenApiExt, web};
use crate::error::ServiceError;
use crate::functionality::ip::Ip;

lazy_static! {
    pub static ref RT: tokio::runtime::Runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("Starting {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    debug!("Reading configuration");
    let config = match Config::read() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to read configuration: {:}", e);
            exit(1);
        }
    };

    debug!("Creating AppData");
    let appdata = match AppData::new(config) {
        Ok(a) => a,
        Err(e) => {
            error!("Failed to create AppData: {:?}", e);
            exit(1);
        }
    };

    debug!("Applying migrations");
    match appdata.migrate() {
        Ok(_) => {},
        Err(e) => {
            error!("Unable to apply migrations: {:?}", e);
            exit(1);
        }
    }

    let appdata = Arc::new(appdata);

    let governor = GovernorConfigBuilder::default()
        .per_second(5)
        .burst_size(20)
        .finish()
        .unwrap();

    let server = HttpServer::new(move || App::new()
        .wrap_api()
        .data(appdata.clone())
        .wrap(actix_web::middleware::Logger::default())
        .wrap(actix_web::middleware::NormalizePath::new(TrailingSlash::Trim))
        .wrap(actix_governor::Governor::new(&governor))
        .wrap(actix_cors::Cors::permissive())
        .service(web::scope("/v1")
            .service(crate::endpoints::crud::registered_clients::registered_clients)
            .service(crate::endpoints::cloud_init::cloud_init))
        .with_json_spec_at("/spec")
        .build()
    )
        .bind("[::]:4333")?
        .run();

    info!("Starting Actix server");
    server.await
}

#[derive(Serialize, Deserialize, Default)]
struct Config {
    mysql_host:         String,
    mysql_database:     String,
    mysql_username:     String,
    mysql_password:     String,
    discord_webhook:    String,
    playbooks:          Vec<Playbook>,
    ansible_ssh_key:    String,
    dns_api:            DnsApi,
    cf_api_token:       Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Playbook {
    function:   PlaybookFunction,
    path:       String
}

impl Playbook {
    pub fn play(&self, target: &Ip, privkey: &str) -> Result<(), ServiceError> {
        let mut child = Command::new("ansible-playbook")
            .args(&[
                &self.path,
                "--extra-vars",
                &format!("target={}", &**target),
                "--private-key",
                privkey])
            .spawn()?;

        let exit = child.wait()?;
        if !exit.success() {
            return Err(ServiceError::UnsuccessfulPlaybook);
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum PlaybookFunction {
    Ip,
    NodeExporter,
}

#[derive(Deserialize, Serialize)]
pub enum DnsApi {
    CloudFlare
}


const PATH: &str = "/etc/cloud-init-ext";
const FILE: &str = "config.yaml";

impl Config {
    fn read() -> anyhow::Result<Self> {
        let path = PathBuf::from(PATH);
        if !path.exists() {
            debug!("Configuration directory '{}' does not exist. Creating it now", PATH);
            fs::create_dir_all(&path)?;

        }

        let file = path.join(FILE);
        if !file.exists() {
            debug!("Configuration file '{}/{}' does not exist, creating a default and exiting", PATH, FILE);
            let mut f = fs::File::create(&file)?;
            serde_yaml::to_writer(&mut f, &Self::default())?;
            info!("A new configuration file has been created at '{}/{}', please configure {} and restart.", PATH, FILE, env!("CARGO_PKG_NAME"));
            exit(0);
        }

        let f = fs::File::open(&file)?;
        let this: Self = serde_yaml::from_reader(f)?;
        Ok(this)
    }
}

pub struct AppData {
    pool:   mysql::Pool,
    config: Config
}

mod migrations {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

impl AppData {
    fn new(config: Config) -> anyhow::Result<Self> {
        let opts = mysql::OptsBuilder::new()
            .ip_or_hostname(Some(&config.mysql_host))
            .db_name(Some(&config.mysql_database))
            .user(Some(&config.mysql_username))
            .pass(Some(&config.mysql_password));
        let pool = mysql::Pool::new(opts)?;

        Ok(Self {
            pool,
            config,
        })
    }

    pub fn get_discord_webhook(&self) -> String {
        self.config.discord_webhook.clone()
    }

    pub fn get_conn(&self) -> Result<mysql::PooledConn, mysql::Error> {
        self.pool.get_conn()
    }

    fn migrate(&self) -> anyhow::Result<()> {
        let mut conn = self.get_conn()?;
        migrations::migrations::runner().run(&mut conn)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn setup() {
        INIT.call_once(|| {
            env_logger::init();
        })
    }
}