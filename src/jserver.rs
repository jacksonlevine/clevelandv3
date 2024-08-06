use std::net::{IpAddr, Ipv4Addr};

use bevy::prelude::*;
use bevy_quinnet::client::certificate::CertificateVerificationMode;
use bevy_quinnet::shared::channels::ChannelType;
use bevy_quinnet::{client::*, server::*, shared::channels::ChannelsConfiguration};

use bevy_quinnet::server::certificate::CertificateRetrievalMode;
use connection::ClientEndpointConfiguration;


pub fn start_listening(mut server: ResMut<QuinnetServer>) {
    server
        .start_endpoint(
            ServerEndpointConfiguration::from_ip(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 6000),
            CertificateRetrievalMode::GenerateSelfSigned {
                server_hostname: String::from("Distant Garden Server"),
            },
            ChannelsConfiguration::from_types(vec![
                ChannelType::Unreliable, //Player updates
                ChannelType::Unreliable, //Mob updates
                ChannelType::OrderedReliable,  //Inventory updates
                ChannelType::OrderedReliable,  //
            ])
            .unwrap(),
        )
        .unwrap();
}
