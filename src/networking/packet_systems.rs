use std::io::{Error, ErrorKind};
use std::{
    io,
    net::{SocketAddr, UdpSocket},
};
use std::collections::{HashMap, VecDeque};

use crate::networking::message::deserialize;
use crate::networking::HeartbeatTimer;
use bevy::prelude::*;
use bytes::Bytes;
use crate::networking::packet_systems::SocketError::NoInput;

use super::{events::NetworkEvent, transport::Transport, NetworkResource};

#[derive(Debug)]
pub enum SocketError {
    ConnectionReset(),
    NoInput(),
    Other(ErrorKind)
}

pub trait SocketLike {
    fn peer_addr(&self) -> Result<SocketAddr, SocketError>;
    fn recv_from(&self, buf: &mut [u8]) ->  Result<(usize, SocketAddr), SocketError>;

    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, SocketError>;
}

#[derive(Resource)]
pub struct SocketLive(
    pub UdpSocket
);

impl SocketLike for SocketLive {
    fn peer_addr(&self) -> Result<SocketAddr, SocketError> {
        return self.0.peer_addr().map_err(|err| match err.kind() {
            ErrorKind::ConnectionReset => NoInput(),
            kind => SocketError::Other(kind)
        });
    }

    fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr), SocketError> {
        return self.0.recv_from(buf).map_err(|err| match err.kind() {
            ErrorKind::WouldBlock => SocketError::NoInput(),
            ErrorKind::ConnectionReset => SocketError::ConnectionReset(),
            _ => SocketError::Other(err.kind())
        });
    }

    fn send_to(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, SocketError> {
        return self.0.send_to(buf, addr).map_err(|err| SocketError::Other(err.kind()));
    }
}

pub struct SocketTest(SocketAddr, HashMap<SocketAddr, VecDeque<(usize, Box<[u8]>, SocketAddr)>>);

impl SocketTest {
    fn peer_addr(&self) -> Result<SocketAddr, SocketError> {
        return Ok(self.0);
    }

    fn recv_from(&mut self, buf: &mut [u8]) -> Result<(usize, SocketAddr), SocketError> {
        let mut data: &mut VecDeque<(usize, Box<[u8]>, SocketAddr)> = self.1.get_mut(&self.0).expect("No queue initialized for in memory socket at: {}");
        match VecDeque::pop_front(data) {
            Some((size, bytes, addr)) => {
                buf[..size].copy_from_slice(bytes.as_ref());
                return Ok((size, addr));
            }

            None => Err(SocketError::NoInput())
        }
    }
}

#[derive(Resource)]
pub struct Socket(pub Box<dyn SocketLike + Send + Sync>);

impl Socket {
    pub fn peer_addr(&self) -> Result<SocketAddr, SocketError> {
        return self.0.peer_addr();
    }
    pub fn recv_from(&self, buf: &mut [u8]) ->  Result<(usize, SocketAddr), SocketError> {
        return self.0.recv_from(buf);
    }

    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> Result<usize, SocketError> {
        return self.0.send_to(buf, addr);
    }
}

#[derive(Resource)]
pub struct SocketAddress(pub SocketAddr);

pub fn client_recv_packet_system(socket: Res<Socket>, mut events: EventWriter<NetworkEvent>) {
    loop {
        let mut buf = [0; 512];
        match socket.recv_from(&mut buf) {
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
                match e {
                    SocketError::NoInput() => (),
                    SocketError::ConnectionReset() => events.send(NetworkEvent::Disconnected(
                        socket
                            .peer_addr()
                            .expect("No peer address for some reason"),
                    )),
                    _ => events.send(NetworkEvent::RecvError(e))
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
        match socket.recv_from(&mut buf) {
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
                match e {
                    SocketError::NoInput() => (),
                    SocketError::ConnectionReset() => events.send(NetworkEvent::Disconnected(
                        socket
                            .peer_addr()
                            .expect("No peer address for some reason"),
                    )),
                    _ => events.send(NetworkEvent::RecvError(e))
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
        if let Err(e) = socket.send_to(&message.payload, message.destination) {
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
