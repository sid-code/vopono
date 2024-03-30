use crate::config::providers::{UiClient, WireguardProvider};
use crate::util::delete_all_files_in_dir;
use regex::Regex;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs::create_dir_all;
use std::io::{Read, Write};
use std::vec::Vec;
use trust_dns_resolver::proto::rr::RecordType;
use trust_dns_resolver::Resolver;

use super::AirVPN;

impl WireguardProvider for AirVPN {
    fn create_wireguard_config(&self, ui: &dyn UiClient) -> anyhow::Result<()> {
        let wireguard_dir = self.wireguard_dir()?;
        create_dir_all(&wireguard_dir)?;
        delete_all_files_in_dir(&wireguard_dir)?;
        let client = Client::new();

        let resolver = Resolver::from_system_conf()?;

        let status = client
            .get("https://airvpn.org/api/status/")
            .send()?
            .json::<AirVPNStatusResponse>()?;

        let unique_countries =
            BTreeSet::from_iter(status.servers.iter().map(|s| s.country_code.clone()));

        let api_key = self.get_api_key(ui)?;

        for country in unique_countries {
            let mut config_str = String::new();
            client
                .get("https://airvpn.org/api/generator/")
                .header("API-KEY", &api_key)
                .query(&[
                    ("system", "linux"),
                    ("protocols", "wireguard_3_udp_1637"),
                    ("servers", &country),
                ])
                .send()?
                .read_to_string(&mut config_str)?;

            match resolver.lookup(format!("{country}.all.vpn.airdns.org."), RecordType::A) {
                Ok(entry_ips) => {
                    for (i, entry_ip) in entry_ips.iter().enumerate() {
                        let pat = Regex::new(r"Endpoint = .+")?;

                        let modified_config_str =
                            pat.replace(&config_str, format!("Endpoint = {entry_ip}:1637"));

                        let path = wireguard_dir.join(format!("{country}-{i}.conf"));

                        let mut f = std::fs::File::create(path)?;

                        write!(f, "{modified_config_str}")?;
                    }
                }
                Err(err) => {
                    println!("WHOOPS, {country} failed DNS lookup.");
                    println!("{err}");
                }
            }
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct AirVPNStatusResponse {
    servers: Vec<AirVPNServerEntry>,
}

#[derive(Deserialize, Debug)]
struct AirVPNServerEntry {
    country_code: String,
}
