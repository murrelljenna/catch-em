/*
    This controls how the server and client decide initial network details once
    the server receives the initial connection. These details are:
        - PlayerId for the newly connected client

    The client cannot receive any other server communication until this handshake
    is completed.
 */

#[derive(Resource)]
pub enum ConnectionStatus {
    Initial, // Client has just sent connection to server
    Acknowledged, // Client has received server acknowledgement
    Complete // Client has sent server acknowledgement
}

#[derive(Resource)]
pub struct NetworkInfo {
    server_handle: &SocketAddr
}


pub fn serverHandshake(handle: &SocketAddr, mut transport: ResMut<Transport>) {
    // Generate player id for client
    let player_id: PlayerId = Players::generate_id();
    // Send client this id
    let message: ServerAcknowledgement = ServerAcknowledgment(player_id);

    transport.send(*handle, &serialize(message));
}

pub fn clientHandshake(assigned_player_id: &PlayerId, mut transport: ResMut<Transport>, mut local_player_id: ResMut<PlayerId>) {
    *local_player_id = *assigned_player_id;


}

pub fn serverCompleteConnection(mut players: ResMut<Players>)

