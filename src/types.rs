use eyre::Report;

use crate::storage::Storage;

#[derive(Debug)]
pub enum Message {
    Initial,
    Finished(Storage),
    Loading(String),
    Error(Report),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Initial => f.write_str("Initial"),
            Message::Finished(_) => f.write_str("Finished"),
            Message::Loading(e) => f.write_fmt(format_args!("{}", e)),
            Message::Error(e) => f.write_fmt(format_args!("{}", e)),
        }
    }
}
