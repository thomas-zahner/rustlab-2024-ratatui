use std::fmt;

use crate::{RoomName, Username};

#[derive(Debug)]
pub enum Command {
    Help,
    ChangeUsername(Username),
    ListRooms,
    Join(RoomName),
    ListUsers,
    SendFile(String, String),
    Nudge(Username),
    Quit,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Command::Help => write!(f, "/help"),
            Command::ChangeUsername(name) => write!(f, "/name {}", name),
            Command::ListRooms => write!(f, "/rooms"),
            Command::Join(room) => write!(f, "/join {}", room),
            Command::ListUsers => write!(f, "/users"),
            Command::SendFile(filename, encoded) => {
                write!(f, "/file {} {}", filename, encoded)
            }
            Command::Nudge(username) => write!(f, "/nudge {}", username),
            Command::Quit => write!(f, "/quit"),
        }
    }
}

impl TryFrom<String> for Command {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut parts = value.split_whitespace();
        match parts.next() {
            Some("/help") => Ok(Command::Help),
            Some("/name") => {
                let name = parts.next().ok_or("Name is required")?.into();
                Ok(Command::ChangeUsername(name))
            }
            Some("/rooms") => Ok(Command::ListRooms),
            Some("/join" | "/j") => {
                let room = parts.next().ok_or("Room name is required")?.into();
                Ok(Command::Join(room))
            }
            Some("/users") => Ok(Command::ListUsers),
            Some("/file") => {
                let filename = parts.next().ok_or("File name is required")?.to_string();
                let encoded = parts.next().ok_or("File content is required")?.to_string();
                Ok(Command::SendFile(filename, encoded))
            }
            Some("/nudge") => {
                let username = parts.next().ok_or("Username is required")?.into();
                Ok(Command::Nudge(username))
            }
            Some("/quit") => Ok(Command::Quit),
            _ => Err(format!("Invalid command: {}", value)),
        }
    }
}
