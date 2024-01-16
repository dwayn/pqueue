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
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts.as_slice() {
            ["UPDATE", item_id, value] => value.parse().map(|val| Command::Update {
                item_id: item_id.to_string(),
                value: val,
            }).unwrap_or(Command::Error {
                msg: "Invalid value for UPDATE".to_string(),
            }),
            ["NEXT"] => Command::Next,
            ["PEEK"] => Command::Peek,
            ["SCORE", item_id] => Command::Score {
                item_id: item_id.to_string(),
            },
            ["INFO"] => Command::Info,
            _ => Command::Error { msg: "Unknown command".to_string() },
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
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::Ok => write!(f, "+OK\r\n"),
            Response::Score(score) => write!(f, "+{}\r\n", score),
            Response::Item(item) => write!(f, "+{}\r\n", item),
            Response::Error(msg) => write!(f, "-{}\r\n", msg),
            Response::Stats(stats) => write!(f,
                "+uptime:{}\r\n+version:{}\r\n+updates:{}\r\n+items:{}\r\n+pools:{}\r\n",
                stats.uptime,
                stats.version,
                stats.updates,
                stats.items,
                stats.pools),
        }
    }
}
