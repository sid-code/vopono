use crate::config::providers::{UiClient, WireguardProvider};
use crate::util::delete_all_files_in_dir;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs::create_dir_all;
use std::vec::Vec;

use super::AirVPN;

impl WireguardProvider for AirVPN {
    fn create_wireguard_config(&self, ui: &dyn UiClient) -> anyhow::Result<()> {
        let wireguard_dir = self.wireguard_dir()?;
        create_dir_all(&wireguard_dir)?;
        delete_all_files_in_dir(&wireguard_dir)?;
        let client = Client::new();

        let status = client
            .get("https://airvpn.org/api/status/")
            .send()?
            .json::<AirVPNStatusResponse>()?;

        let unique_countries =
            BTreeSet::from_iter(status.servers.iter().map(|s| s.country_code.clone()));

        let api_key = self.get_api_key(ui)?;

        for country in unique_countries {
            // TODO: use DNS to get a list of actual servers and
            // generate configs for each.
            let path = wireguard_dir.join(format!("{country}.conf"));

            let mut config = client
                .get("https://airvpn.org/api/generator/")
                .header("API-KEY", &api_key)
                .query(&[
                    ("system", "linux"),
                    ("protocols", "wireguard_3_udp_1637"),
                    ("servers", &country),
                ])
                .send()?;

            let mut f = std::fs::File::create(path)?;
            std::io::copy(&mut config, &mut f)?;
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
