pub mod args;
pub mod connection;
pub mod logger;
pub mod room;
pub mod user;

const SERVER_COMMANDS: &str =
    "/help | /name {name} | /rooms | /join {room} | /users | /nudge {name} | /quit";

#[macro_export]
macro_rules! b {
    ($result:expr) => {
        match $result {
            Ok(ok) => ok,
            Err(err) => break Err(err.into()),
        }
    };
}
