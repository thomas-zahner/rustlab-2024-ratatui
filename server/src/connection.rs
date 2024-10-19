use crate::{
    b,
    room::{RoomMsg, DEFAULT_ROOM},
    SERVER_COMMANDS,
};
use std::{io::ErrorKind, net::SocketAddr, sync::Arc};

use futures::SinkExt;
use tokio::{net::TcpStream, sync::broadcast::error::RecvError};
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec, LinesCodecError};

use crate::{room::Rooms, user::Users};

const MAX_MSG_LEN: usize = 400;

pub struct Connection {
    tcp: TcpStream,
    users: Users,
    rooms: Rooms,
    username: String,
    addr: SocketAddr,
}

impl Connection {
    pub fn new(tcp: TcpStream, users: Users, rooms: Rooms, addr: SocketAddr) -> Self {
        let username = petname::petname(2, "").expect("failed to generate username");
        tracing::debug!("{addr} connected with the name: {username}");
        Self {
            tcp,
            users,
            rooms,
            username,
            addr,
        }
    }

    pub async fn handle(&mut self) {
        let (reader, writer) = self.tcp.split();
        let mut stream = FramedRead::new(reader, LinesCodec::new_with_max_length(MAX_MSG_LEN));
        let mut sink = FramedWrite::new(writer, LinesCodec::new_with_max_length(MAX_MSG_LEN + 100));

        let mut exit_result = sink
            .send(format!("{SERVER_COMMANDS}\nYou are {}", self.username))
            .await;
        if should_exit(exit_result) {
            self.users.remove(&self.username);
            return;
        }

        let mut room_name = DEFAULT_ROOM.to_string();
        let mut room_tx = self.rooms.join(&room_name, &self.username);
        let mut room_rx = room_tx.subscribe();
        let _ = room_tx.send(RoomMsg::Joined(self.username.clone()));

        let mut discarding_long_msg = false;

        exit_result = loop {
            tokio::select! {
                user_msg = stream.next() => {
                    let user_msg = match user_msg {
                        Some(Ok(msg)) => msg,
                        Some(Err(LinesCodecError::MaxLineLengthExceeded)) => {
                            b!(sink.send(format!("Messages can only be {MAX_MSG_LEN} chars long")).await);
                            discarding_long_msg = true;
                            continue;
                        }
                        Some(Err(LinesCodecError::Io(io_err))) => match io_err.kind() {
                            ErrorKind::InvalidData | ErrorKind::InvalidInput
                            | ErrorKind::BrokenPipe | ErrorKind::ConnectionReset => break Ok(()),
                            _ => break Err(LinesCodecError::Io(io_err)),
                        },
                        None if !discarding_long_msg => break Ok(()),
                        None => {
                            discarding_long_msg = false;
                            continue;
                        }
                    };
                    if user_msg.starts_with("/help") {
                        b!(sink.send(SERVER_COMMANDS).await);
                    } else if user_msg.starts_with("/name") {
                        let new_name = user_msg
                            .split_ascii_whitespace()
                            .nth(1);
                        let new_name = String::from(new_name.unwrap());
                        let changed_name = self.users.insert(new_name.clone());
                        if changed_name {
                            self.rooms.change_name(&room_name, &self.username, &new_name);
                            let msg = format!("{} is now {new_name}", self.username);
                            let msg: Arc<str> = Arc::from(msg.as_str());
                            let _ = room_tx.send(RoomMsg::Msg(msg));
                            self.username = new_name;
                        } else {
                            b!(sink.send(format!("{new_name} is already taken")).await);
                        }
                    } else if user_msg.starts_with("/join") {
                        let new_room = user_msg
                            .split_ascii_whitespace()
                            .nth(1);
                        let new_room = String::from(new_room.unwrap());
                        if new_room == room_name {
                            b!(sink.send(format!("You are in {room_name}")).await);
                            continue;
                        }
                        let _ = room_tx.send(RoomMsg::Left(self.username.clone()));
                        room_tx = self.rooms.change(&room_name, &new_room, &self.username);
                        room_rx = room_tx.subscribe();
                        room_name = new_room;
                        let _ = room_tx.send(RoomMsg::Joined(self.username.clone()));
                    } else if user_msg.starts_with("/rooms") {
                        let rooms_list = self.rooms.list();
                        let mut rooms_msg = String::with_capacity(rooms_list.len() * 15);
                        rooms_msg.push_str("Rooms - ");
                        for room in rooms_list {
                            rooms_msg.push_str(&room.0);
                            rooms_msg.push_str(" (");
                            rooms_msg.push_str(&room.1.to_string());
                            rooms_msg.push_str("), ");
                        }
                        // pop off trailing comma + space
                        rooms_msg.pop();
                        rooms_msg.pop();
                        b!(sink.send(rooms_msg).await);
                    } else if user_msg.starts_with("/users") {
                        let users_list = self.rooms.list_users(&room_name).unwrap();
                        let mut users_msg = String::with_capacity(users_list.len() * 15);
                        users_msg.push_str("Users - ");
                        for user in users_list {
                            users_msg.push_str(&user);
                            users_msg.push_str(", ");
                        }
                        // pop off trailing comma + space
                        users_msg.pop();
                        users_msg.pop();
                        b!(sink.send(users_msg).await);
                    } else if user_msg.starts_with("/quit") {
                        break Ok(());
                    } else if user_msg.starts_with("/") {
                        let unrecognized = user_msg
                            .split_ascii_whitespace()
                            .next()
                            .unwrap();
                        b!(sink.send(format!("Unrecognized command {unrecognized}, try /help")).await);
                    } else {
                        let msg = format!("{}: {user_msg}", self.username);
                        let msg: Arc<str> = Arc::from(msg.as_str());
                        let _ = room_tx.send(RoomMsg::Msg(msg));
                    }
                }
                peer_msg = room_rx.recv() => {
                    let peer_msg = match peer_msg {
                        Ok(ok) => ok,
                        // we would get this error if all tx
                        // were dropped for this rx, which is not
                        // possible since we're holding a tx,
                        // but if this were to somehow ever happen
                        // we just put the user back into the main
                        // room
                        Err(RecvError::Closed) => {
                            let _ = room_tx.send(RoomMsg::Left(self.username.clone()));
                            room_tx = self.rooms.change(&room_name, DEFAULT_ROOM, &self.username);
                            room_rx = room_tx.subscribe();
                            room_name = DEFAULT_ROOM.into();
                            let _ = room_tx.send(RoomMsg::Joined(self.username.clone()));
                            continue;
                        },
                        // under high load we might not deliver all msgs
                        // to all users in a room, in which case we let
                        // them know that we dropped some msgs
                        Err(RecvError::Lagged(n)) => {
                            tracing::warn!("Server dropped {n} messages for {room_name} with {} users", room_tx.receiver_count());
                            b!(sink.send(format!("Server is very busy and dropped {n} messages, sorry!")).await);
                            continue;
                        }
                    };
                    match peer_msg {
                        RoomMsg::Joined(peer_name) => {
                            let msg = if self.username == peer_name {
                                format!("You joined {room_name}")
                            } else {
                                format!("{peer_name} joined")
                            };
                            b!(sink.send(msg).await);
                        },
                        RoomMsg::Left(peer_name) => {
                            let msg = if self.username == peer_name {
                                format!("You left {room_name}")
                            } else {
                                format!("{peer_name} left")
                            };
                            b!(sink.send(msg).await);
                        },
                        RoomMsg::Msg(msg) => {
                            b!(sink.send(msg).await);
                        },
                    };

                }
            }
        };

        let _ = room_tx.send(RoomMsg::Left(self.username.clone()));
        tracing::debug!("{} disconnected (name: {})", self.addr, self.username);
        self.rooms.leave(&room_name, &self.username);
        self.users.remove(&self.username);
        should_exit(exit_result);
    }
}

fn should_exit(result: Result<(), LinesCodecError>) -> bool {
    match result {
        Ok(_) => false,
        Err(LinesCodecError::MaxLineLengthExceeded) => true,
        Err(LinesCodecError::Io(err))
            if matches!(
                err.kind(),
                ErrorKind::BrokenPipe | ErrorKind::ConnectionReset
            ) =>
        {
            true
        }
        Err(LinesCodecError::Io(err)) => {
            tracing::error!("unexpected error: {err}");
            true
        }
    }
}
