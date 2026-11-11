use super::{commands::Command, queries::Query};

/// Both kinds use one ordered queue without making queries into write commands.
pub enum Request {
    Command(Command),
    Query(Query),
}
impl From<Command> for Request {
    fn from(command: Command) -> Self {
        Self::Command(command)
    }
}
impl From<Query> for Request {
    fn from(query: Query) -> Self {
        Self::Query(query)
    }
}
