use std::net::IpAddr;
use std::str::FromStr;
use mysql::{PooledConn, Row, params};
use mysql::prelude::Queryable;
use crate::error::ServiceError;
use log::trace;

pub struct IpService<'a> {
    conn:   &'a mut PooledConn
}

pub struct Ip(String);

impl std::ops::Deref for Ip {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> IpService<'a> {
    pub fn new(conn: &'a mut PooledConn) -> Self {
        Self {
            conn
        }
    }

    pub fn register(&mut self, ip: &Ip, hostname: &str) -> Result<(), ServiceError> {
        self.conn.exec_drop("INSEERT INTO registered_clients (ip, hostname) VALUES (:ip, :hostname)", params! {
            "ip" => &**ip,
            "hostname" => hostname
        })?;

        Ok(())
    }

    pub fn get_next_available_ip(&mut self) -> Result<Ip, ServiceError> {
        let rows: Vec<Row> = self.conn.exec("SELECT ip FROM registered_clients", mysql::Params::Empty)?;

        let mut addrs = Vec::new();
        for row in rows {
            let ip: String = row.get("ip").unwrap();
            let ip_addr = IpAddr::from_str(&ip)?;

            if ip_addr.is_ipv6() {
                return Err(ServiceError::Unsupported("Currently only IPv4 addresses are supported"));
            }

            addrs.push(ip_addr);
        }

        let next_free = Self::next_free_addr_v4(&addrs)?;
        Ok(Ip(next_free.to_string()))
    }

    fn next_free_addr_v4(addrs: &Vec<IpAddr>) -> Result<IpAddr, ServiceError> {
        let mut addrs = addrs.clone();
        addrs.sort_by(|a, b| {
            let a_str = a.to_string();
            let b_str = b.to_string();
            let octets_a: Vec<_> = a_str.split(".").collect();
            let octets_b: Vec<_> = b_str.split(".").collect();
            let last_a = octets_a.last().unwrap();
            let last_b = octets_b.last().unwrap();

            let a_int: i32 = last_a.parse().unwrap();
            let b_int: i32 = last_b.parse().unwrap();
            a_int.partial_cmp(&b_int).unwrap()
        });

        let mut prev = -1i32;
        let mut prev_octets = Vec::default();
        for addr in addrs {
            trace!("Current Addr: {:?}", &addr);
            let as_str = addr.to_string();
            let mut octets: Vec<_> = as_str.split(".").map(|f| f.to_string()).collect();
            let last_octet = octets.last().unwrap(); // This cannot be None because IpAddr::from_str validated the form of the address
            let as_int: i32 = last_octet.parse()?;

            trace!("Got last octet: {}", as_int);

            if prev == -1 {
                prev = as_int;
                prev_octets = octets;
                trace!("IP is first in sequence");
                continue;
            }

            if as_int - prev > 1 {
                trace!("Got free address: {}", prev + 1);
                break;
            }

            prev = as_int;
            prev_octets = octets;
        }

        if prev == 254 {
            panic!("No more free addresses available");
        }

        let new = prev + 1;
        prev_octets.pop();
        prev_octets.push(new.to_string());
        let ip = prev_octets.join(".");
        trace!("Built new IP {}", ip);

        Ok(IpAddr::from_str(&ip)?)
    }
}

#[cfg(test)]
mod test {
    use std::net::{IpAddr, Ipv4Addr};
    use super::*;
    use crate::test::setup;

    #[test]
    fn next_free_addr_v4() {
        setup();
        let mut addrs = vec![
            IpAddr::V4(Ipv4Addr::new(10, 10, 0, 1)),
            IpAddr::V4(Ipv4Addr::new(10, 10, 0, 2)),
            IpAddr::V4(Ipv4Addr::new(10, 10, 0, 4)),
            IpAddr::V4(Ipv4Addr::new(10, 10, 0, 5))
        ];

        let next_addr = IpService::next_free_addr_v4(&addrs).expect("Unable to compute next free address");
        assert_eq!("10.10.0.3", next_addr.to_string().as_str());
        addrs.push(IpAddr::V4(Ipv4Addr::new(10, 10, 0, 3)));

        let next_addr = IpService::next_free_addr_v4(&addrs).expect("Unable to compute next free address");
        assert_eq!("10.10.0.6", next_addr.to_string().as_str());
        addrs.push(IpAddr::V4(Ipv4Addr::new(10, 10, 0, 6)));
        addrs.push(IpAddr::V4(Ipv4Addr::new(10, 10, 0, 10)));

        let next_addr = IpService::next_free_addr_v4(&addrs).expect("Unable to compute next free address");
        assert_eq!("10.10.0.7", next_addr.to_string().as_str());
    }
}