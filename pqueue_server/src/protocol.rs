use std::fmt;

use concurrent_pqueue::PQueueStats;

/// Represents client commands that can be sent to the priority queue server.
#[derive(Clone, Debug)]
pub enum Command {
    /// Updates an item's score (additive) or adds it with the given score
    Update { item_id: String, value: i64 },
    /// Removes and returns the highest-scoring item
    Next,
    /// Returns the highest-scoring item without removing it
    Peek,
    /// Gets the current score for a specific item
    Score { item_id: String },
    /// Returns server statistics
    Info,
    /// Invalid command with error message
    Error { msg: String },
    /// Returns help text
    Help,
}

impl From<&str> for Command {
    /// Parses a string command into a Command enum.
    /// Commands are case-insensitive, but identifiers are case-sensitive.
    fn from(s: &str) -> Self {
        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts.as_slice() {
            [command, item_id, value] if command.eq_ignore_ascii_case("UPDATE") => value
                .parse()
                .map(|val| Command::Update {
                    item_id: item_id.to_string(),
                    value: val,
                })
                .unwrap_or(Command::Error {
                    msg: "Invalid value for UPDATE".to_string(),
                }),
            [command] if command.eq_ignore_ascii_case("NEXT") => Command::Next,
            [command] if command.eq_ignore_ascii_case("PEEK") => Command::Peek,
            [command, item_id] if command.eq_ignore_ascii_case("SCORE") => Command::Score {
                item_id: item_id.to_string(),
            },
            [command] if command.eq_ignore_ascii_case("INFO") => Command::Info,
            [command] if command.eq_ignore_ascii_case("HELP") => Command::Help,
            _ => Command::Error {
                msg: "Invalid command or arguments".to_string(),
            },
        }
    }
}

/// Represents server responses sent back to clients.
#[derive(Clone, Debug)]
pub enum Response {
    /// Successful operation confirmation
    Ok,
    /// Numeric score response
    Score(i64),
    /// Item identifier response
    Item(String),
    /// Error message response
    Error(String),
    /// Server statistics response
    Stats(PQueueStats),
    /// Help text response
    Help,
}

impl fmt::Display for Response {
    /// Formats responses according to the line-based protocol.
    /// All responses are terminated with CRLF and prefixed with + or - for status.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::Ok => write!(f, "+OK\r\n"),
            Response::Score(score) => write!(f, "+{}\r\n", score),
            Response::Item(item) => write!(f, "+{}\r\n", item),
            Response::Error(msg) => write!(f, "-{}\r\n", msg),
            Response::Stats(stats) => write!(f,
                // Multi-line INFO response with key:value pairs
                "+INFO\r\n+uptime:{}\r\n+version:{}\r\n+updates:{}\r\n+items:{}\r\n+pools:{}\r\n",
                stats.uptime.num_seconds(),
                stats.version,
                stats.updates,
                stats.items,
                stats.pools),
            Response::Help => write!(f,
                // Multi-line help text explaining protocol commands
                "USAGE (note: commands are case insensitive, identifiers are case sensitive): \r\n\
                 +UPDATE <identifier> <score> [Updates the priority of <identifier> by adding <score> to its priority or inserts it with priority of <score>]\r\n \
                 +NEXT                        [Pops the highest priority item (item that has had that priority the longest if multiple) off the queue]\r\n \
                 +SCORE <identifier>          [Fetch the current priority score for <identifier>]\r\n \
                 +INFO                        [Fetch statistics about the server]\r\n \
                 +HELP                        [Get this help]\r\n"
            )
        }
    }
}
