use std::{
    io,
    net::{SocketAddr, UdpSocket},
};

use crate::networking::message::{deserialize, Message};
use crate::networking::HeartbeatTimer;
use bevy::prelude::*;
use bytes::Bytes;

use super::{events::NetworkEvent, transport::Transport, NetworkResource};

#[derive(Resource)]
pub struct Socket(pub UdpSocket);

#[derive(Resource)]
pub struct SocketAddress(pub SocketAddr);

pub fn client_recv_packet_system(socket: Res<Socket>, mut events: EventWriter<NetworkEvent>) {
    loop {
        let mut buf = [0; 512];
        match socket.0.recv_from(&mut buf) {
            Ok((recv_len, address)) => {
                let payload = Bytes::copy_from_slice(&buf[..recv_len]);
                if payload.len() == 0 {
                    debug!("{}: received heartbeat packet", address);
                    // discard without sending a NetworkEvent
                    continue;
                }
                debug!("received payload {:?} from {}", payload, address);
                let message = deserialize(payload);
                events.send(NetworkEvent::RawMessage(address, message));
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    events.send(NetworkEvent::RecvError(e));
                }
                // break loop when no messages are left to read this frame
                break;
            }
        }
    }
}

pub fn server_recv_packet_system(
    time: Res<Time>,
    socket: Res<Socket>,
    mut events: EventWriter<NetworkEvent>,
    mut net: ResMut<NetworkResource>,
) {
    loop {
        let mut buf = [0; 512];
        match socket.0.recv_from(&mut buf) {
            Ok((recv_len, address)) => {
                let payload = Bytes::copy_from_slice(&buf[..recv_len]);
                if net.connections.insert(address, time.elapsed()).is_none() {
                    // connection established
                    events.send(NetworkEvent::Connected(address));
                }
                if payload.len() == 0 {
                    debug!("{}: received heartbeat packet", address);
                    // discard without sending a NetworkEvent
                    continue;
                }
                debug!("received payload {:?} from {}", payload, address);
                let message = deserialize(payload);
                events.send(NetworkEvent::RawMessage(address, message));
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::WouldBlock {
                    events.send(NetworkEvent::RecvError(e));
                }
                // break loop when no messages are left to read this frame
                break;
            }
        }
    }
}

pub fn send_packet_system(
    socket: Res<Socket>,
    mut events: EventWriter<NetworkEvent>,
    mut transport: ResMut<Transport>,
) {
    let messages = transport.drain_messages_to_send(|_| true);
    for message in messages {
        if let Err(e) = socket.0.send_to(&message.payload, message.destination) {
            events.send(NetworkEvent::SendError(e, message))
        }
    }
}

pub fn idle_timeout_system(
    time: Res<Time>,
    mut net: ResMut<NetworkResource>,
    mut events: EventWriter<NetworkEvent>,
) {
    let idle_timeout = net.idle_timeout.clone();
    net.connections.retain(|addr, last_update| {
        let reached_idle_timeout = time.elapsed() - *last_update > idle_timeout;
        if reached_idle_timeout {
            println!("Reached idle timeout");
            events.send(NetworkEvent::Disconnected(*addr));
        }
        !reached_idle_timeout
    });
}

pub fn auto_heartbeat_system(
    time: Res<Time>,
    mut timer: ResMut<HeartbeatTimer>,
    remote_addr: Res<SocketAddress>,
    mut transport: ResMut<Transport>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        transport.send(remote_addr.0, Default::default());
    }
}
