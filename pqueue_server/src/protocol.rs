use std::fmt;

use pqueue::PQueueStats;


#[derive(Clone, Debug)]
pub enum Command {
    Update { item_id: String, value: i64 },
    Next,
    Peek,
    Score { item_id: String },
    Info,
    Error { msg: String },
    Help,
}


impl From<&str> for Command {
    fn from(s: &str) -> Self {
        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts.as_slice() {
            [command, item_id, value] if command.eq_ignore_ascii_case("UPDATE") => {
                value.parse().map(|val| Command::Update {
                    item_id: item_id.to_string(),
                    value: val,
                }).unwrap_or(Command::Error {
                    msg: "Invalid value for UPDATE".to_string(),
                })
            },
            [command] if command.eq_ignore_ascii_case("NEXT") => Command::Next,
            [command] if command.eq_ignore_ascii_case("PEEK") => Command::Peek,
            [command, item_id] if command.eq_ignore_ascii_case("SCORE") => Command::Score {
                item_id: item_id.to_string(),
            },
            [command] if command.eq_ignore_ascii_case("INFO") => Command::Info,
            [command] if command.eq_ignore_ascii_case("INFO") => Command::Help,
            _ => Command::Error { msg: "Invalid command or arguments".to_string() },
        }
    }
}

#[derive(Clone, Debug)]
pub enum Response {
    Ok,
    Score(i64),
    Item(String),
    Error(String),
    Stats(PQueueStats),
    Help,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::Ok => write!(f, "+OK\r\n"),
            Response::Score(score) => write!(f, "+{}\r\n", score),
            Response::Item(item) => write!(f, "+{}\r\n", item),
            Response::Error(msg) => write!(f, "-{}\r\n", msg),
            Response::Stats(stats) => write!(f,
                "+INFO\r\n+uptime:{}\r\n+version:{}\r\n+updates:{}\r\n+items:{}\r\n+pools:{}\r\n",
                stats.uptime.num_seconds(),
                stats.version,
                stats.updates,
                stats.items,
                stats.pools),
            Response::Help => write!(f,
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
