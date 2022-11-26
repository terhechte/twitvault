use eyre::Report;

use crate::storage::Storage;

#[derive(Debug)]
pub enum Message {
    Finished(Storage),
    Loading(String),
    Error(Report),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Finished(_) => f.write_str("Finished"),
            Message::Loading(e) => f.write_fmt(format_args!("{}", e)),
            Message::Error(e) => f.write_fmt(format_args!("{}", e)),
        }
    }
}
