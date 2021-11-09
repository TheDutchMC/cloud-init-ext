use crate::error::ServiceError;
use crate::functionality::ip::Ip;
use crate::{Playbook, PlaybookFunction};

pub struct AnsibleService<'a> {
    playbooks:      &'a Vec<Playbook>,
    private_key:    &'a str,
}

impl<'a> AnsibleService<'a> {
    pub fn new(playbooks: &'a Vec<Playbook>, private_key: &'a str) -> Self {
        Self {
            playbooks,
            private_key
        }
    }

    pub fn provision_ip(&self, ip: &Ip) -> Result<(), ServiceError> {
        let playbook = self.get_playbook(PlaybookFunction::Ip)?;
        playbook.play(ip, &self.private_key)?;
        Ok(())
    }

    pub fn node_exporter(&self, ip: &Ip) -> Result<(), ServiceError> {
        let playbook = self.get_playbook(PlaybookFunction::NodeExporter)?;
        playbook.play(ip, &self.private_key)?;
        Ok(())
    }

    fn get_playbook(&self, function: PlaybookFunction) -> Result<&Playbook, ServiceError> {
        let playbook = self.playbooks.iter()
            .find(|f| f.function.eq(&function));

        let playbook = match playbook {
            Some(p) => p,
            None => return Err(ServiceError::MissingPlaybook(function))
        };

        Ok(playbook)
    }
}