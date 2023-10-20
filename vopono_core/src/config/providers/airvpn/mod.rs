mod openvpn;
mod wireguard;

use std::env;

use super::{ConfigurationChoice, OpenVpnProvider, Provider};
use crate::config::providers::UiClient;
use crate::config::vpn::Protocol;
use anyhow::anyhow;

pub struct AirVPN {}

impl AirVPN {
    fn get_api_key(&self, ui: &dyn UiClient) -> anyhow::Result<String> {
        Ok(env::var("AIRVPN_API_KEY").or_else(|_|
                ui.get_input(crate::config::providers::Input{prompt: "Enter your AirVPN API key (see https://airvpn.org/apisettings/ )".to_string(), validator: None})
                  ).map_err(|_| {
                    anyhow!("Cannot generate AirVPN OpenVPN config files: AIRVPN_API_KEY is not defined in your environment variables. Get your key by activating API access in the Client Area at https://airvpn.org/apisettings/")
                })?.trim().to_string())
    }
}

impl Provider for AirVPN {
    fn alias(&self) -> String {
        "air".to_string()
    }

    fn alias_2char(&self) -> String {
        "ar".to_string()
    }

    fn default_protocol(&self) -> Protocol {
        Protocol::Wireguard
    }
}
