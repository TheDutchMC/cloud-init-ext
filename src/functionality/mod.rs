use std::sync::Arc;
use dwbhk::{
    WebhookRequestBuilder,
    WebhookBuilder,
    EmbedBuilder,
    EmbedFieldBuilder
};
use crate::{AppData, RT};
use crate::error::ServiceError;
use crate::functionality::ip::IpService;
use log::{debug, error};
use crate::functionality::ansible::AnsibleService;

pub mod ip;
pub mod ansible;

pub struct Client {
    hostname:   String,
    current_ip: String,
}

pub fn provision(data: Arc<AppData>, client: Client) {
    std::thread::spawn(move || {
        match do_provision(data.clone(), &client) {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to privision client: {:?}", e);
                let webhook = WebhookRequestBuilder::new()
                    .set_data(WebhookBuilder::new()
                        .set_embeds(vec![
                            EmbedBuilder::new()
                                .set_title("Provision failed")
                                .set_description(format!("Failed to provision client '{}'", client.hostname))
                                .set_fields(vec![
                                    EmbedFieldBuilder::new()
                                        .set_name("Error")
                                        .set_value(format!("{}", e))
                                        .build()
                                ])
                                .build()
                        ])
                        .build())
                    .build();
                let _guard = RT.enter();
                match RT.block_on(webhook.execute_url(&data.config.discord_webhook)) {
                    Ok(_) => {},
                    Err(e) => {
                        error!("Failed to trigger Discord webhook: {:?}", e);
                    }
                }
            }
        }
    });
}

fn do_provision(data: Arc<AppData>, client: &Client) -> Result<(), ServiceError> {
    let mut conn = data.get_conn()?;

    let mut ip_service = IpService::new(&mut conn);
    let avail_ip = ip_service.get_next_available_ip()?;
    ip_service.register(&avail_ip, &client.hostname)?;

    let ansible_service = AnsibleService::new(&data.config.playbooks, &data.config.ansible_ssh_key);
    ansible_service.provision_ip(&avail_ip)?;
    ansible_service.node_exporter(&avail_ip)?;

    Ok(())
}