use std::{net::UdpSocket, time::Duration};

use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin, prelude::*};
use bevy::log::Level;
use bevy::time::TimePlugin;
use crate::networking::{NetworkEvent, ServerPlugin, Transport};
use crate::networking::systems::Socket;

const LISTEN_ADDRESS: &str = "127.0.0.1:8080";

pub fn main() {
    println!("Ahhhh");
    let socket = UdpSocket::bind(LISTEN_ADDRESS).expect("could not bind socket");
    println!("Ahhhh");
    socket
        .set_nonblocking(true)
        .expect("could not set socket to be nonblocking");
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("could not set read timeout");
    println!("Ahhhh");

    info!("Server now listening on {}", LISTEN_ADDRESS);

    App::new()
        // run the server at a reduced tick rate (100 ticks per minute)
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f32(
            60. / 100.,
        )))
        .insert_resource(Socket(socket))
        .add_plugins(TimePlugin::default())
        .add_plugins(LogPlugin {
            filter: "".to_string(),
            level: Level::INFO,
        })
        .add_plugins(ServerPlugin)
        .add_systems(Update,connection_handler)
        .run();
}

fn connection_handler(mut events: EventReader<NetworkEvent>, mut transport: ResMut<Transport>) {
    println!("Hi");
    for event in events.iter() {
        match event {
            NetworkEvent::Connected(handle) => {
                info!("{}: connected!", handle);
                println!("AHH");
                transport.send(*handle, b"PONG");
            }
            NetworkEvent::Disconnected(handle) => {
                info!("{}: disconnected!", handle);
            }
            NetworkEvent::Message(handle, msg) => {
                info!("{} sent a message: {:?}", handle, msg);
            }
            NetworkEvent::SendError(err, msg) => {
                error!(
                    "NetworkEvent::SendError (payload [{:?}]): {:?}",
                    msg.payload, err
                );
            }
            NetworkEvent::RecvError(err) => {
                error!("NetworkEvent::RecvError: {:?}", err);
            }
        }
    }
}