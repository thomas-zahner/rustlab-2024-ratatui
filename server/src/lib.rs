pub mod args;
pub mod connection;
pub mod logger;
pub mod room;
pub mod user;

pub const SERVER_COMMANDS: &str = "
Server commands
  /help - print this message
  /name {name} - change name
  /rooms - list rooms
  /join {room} - joins room
  /users - list users in room
  /quit - quit server
";

#[macro_export]
macro_rules! b {
    ($result:expr) => {
        match $result {
            Ok(ok) => ok,
            Err(err) => break Err(err.into()),
        }
    };
}
