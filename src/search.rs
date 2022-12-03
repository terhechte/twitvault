use std::{collections::HashMap, ops::Range};

use crate::storage::{Data, TweetId, UserId};
use egg_mode::{tweet::Tweet, user::TwitterUser};
use regex::Regex;

/// Super simple search.
/// strings ("search term") are searched full phrase
/// individual words (search term) are searched first all then any

pub struct Description {
    pub field: &'static str,
    pub content: String,
    pub highlights: Vec<Range<usize>>,
    pub rank: usize,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Kind {
    Tweet(TweetId),
    Profile(UserId),
}

pub struct SearchResult {
    pub kind: Kind,
    pub desc: Vec<Description>,
    pub rank: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    pub tweets: bool,
    pub mentions: bool,
    pub responses: bool,
    pub profiles: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            tweets: true,
            mentions: true,
            responses: true,
            profiles: true,
        }
    }
}

pub fn search(term: String, data: &Data, options: Options) -> Vec<SearchResult> {
    let mut results = HashMap::new();
    let Some(regex) = make_regex(term) else {
        return Vec::new()
    };

    if options.tweets {
        search_tweets(&regex, &data.tweets, &mut results);
    }
    if options.mentions {
        search_tweets(&regex, &data.mentions, &mut results);
    }
    if options.responses {
        for i in data.responses.values() {
            search_tweets(&regex, i, &mut results);
        }
    }
    if options.profiles {
        for user in data.profiles.values() {
            let mut descriptions = Vec::new();
            search_profile(&regex, user, &mut descriptions);
            if !descriptions.is_empty() {
                descriptions.sort_by(|a, b| a.rank.cmp(&b.rank));
                let rank = descriptions.iter().map(|s| s.rank).sum();
                results.insert(
                    Kind::Profile(user.id),
                    SearchResult {
                        kind: Kind::Profile(user.id),
                        desc: descriptions,
                        rank,
                    },
                );
            }
        }
    }

    let mut values: Vec<_> = results.into_values().collect();
    values.sort_by(|a, b| a.rank.cmp(&b.rank));

    values
}

fn make_regex(term: String) -> Option<Regex> {
    let mut phrase = "(?i)".to_string();
    let refined = term.trim();
    if (refined.starts_with('"') && refined.ends_with('"'))
        || (refined.starts_with('\'') && refined.ends_with('\''))
    {
        phrase.push_str(&refined[1..(refined.len() - 1)]);
    } else {
        phrase.push('(');
        for component in term.split_ascii_whitespace() {
            phrase.push_str(component);
            phrase.push('|');
        }
        phrase.pop();
        phrase.push(')');
    }
    Regex::new(&phrase).ok()
}

fn search_tweets(regex: &Regex, tweets: &[Tweet], into: &mut HashMap<Kind, SearchResult>) {
    for tweet in tweets {
        let mut descriptions = Vec::new();
        if let Some(ref user) = tweet.user {
            search_profile(regex, user, &mut descriptions);
        }
        search_tweet(regex, tweet, &mut descriptions);
        if !descriptions.is_empty() {
            descriptions.sort_by(|a, b| a.rank.cmp(&b.rank));
            let rank = descriptions.iter().map(|s| s.rank).sum();
            into.insert(
                Kind::Tweet(tweet.id),
                SearchResult {
                    kind: Kind::Tweet(tweet.id),
                    desc: descriptions,
                    rank,
                },
            );
        }
    }
}

fn search_profile(regex: &Regex, user: &TwitterUser, descriptions: &mut Vec<Description>) {
    if let Some(m) = make_results(&user.screen_name, regex, "Screen Name", 2) {
        descriptions.push(m);
    }
    if let Some(m) = make_results(&user.name, regex, "Name", 2) {
        descriptions.push(m);
    }
    if let Some(ref d) = user.description {
        if let Some(m) = make_results(d, regex, "Profile Description", 1) {
            descriptions.push(m);
        }
    }
    if let Some(ref s) = user.status {
        if let Some(m) = make_results(&s.text, regex, "Profile Status Tweet", 0) {
            descriptions.push(m);
        }
    }
}

fn search_tweet(regex: &Regex, tweet: &Tweet, descriptions: &mut Vec<Description>) {
    if let Some(m) = make_results(&tweet.text, regex, "Tweet Text", 4) {
        descriptions.push(m);
    }
    if let Some(quoted) = tweet
        .quoted_status
        .as_ref()
        .map(|q| &q.text)
        .and_then(|s| make_results(s, regex, "Tweet Quoted Text", 1))
    {
        descriptions.push(quoted);
    }
}

fn make_results(
    content: &str,
    regex: &Regex,
    field: &'static str,
    rank: usize,
) -> Option<Description> {
    let r = regex.find_iter(content);
    let highlights: Vec<Range<usize>> = r.map(|f| f.range()).collect();
    if highlights.is_empty() {
        None
    } else {
        Some(Description {
            field,
            content: content.to_string(),
            highlights,
            rank,
        })
    }
}
