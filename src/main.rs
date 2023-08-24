use chrono::Local;
use std::collections::{HashMap, HashSet};
use std::env;
use stripmargin::StripMargin;
use url::Url as UrlType;
use webpage::{Webpage, WebpageOptions};

#[derive(Debug)]
struct OrgEntry {
    pub title: String,
    pub body: String,
}

impl OrgEntry {
    fn new(title: String, body: String) -> Self {
        Self { title, body }
    }
    fn make(args: &[String]) -> Option<Self> {
        match args {
            [title] => Some(OrgEntry::new(title.clone(), "".to_string())),
            [title, body] => Some(OrgEntry::new(title.clone(), body.clone())),
            _ => None,
        }
    }
    fn apply_to_text<F>(&self, f: F) -> Self
    where
        F: Fn(&str) -> String,
    {
        Self::new(f(&self.title), f(&self.body))
    }

    fn to_orgmode(&self) -> String {
        let now = Local::now().format("%Y-%m-%d %a %H:%M").to_string();
        format!(
            "INBOX {}
           |:PROPERTIES:
           |:CREATED:  [{}]
           |:END:
           |{}",
            self.title, now, self.body
        )
        .strip_margin()
    }
}

struct OrgModeFormatter;

impl OrgModeFormatter {
    fn link(url: &str, title: &str) -> String {
        format!("[[{}][{}]]", url, title)
    }
}

type UrlString = String;

pub struct Url;

impl Url {
    fn collect(text: &str) -> HashSet<UrlString> {
        let mut urls = HashSet::new();
        for word in text.split_whitespace() {
            if let Ok(url) = UrlType::parse(word) {
                if url.scheme() == "http" || url.scheme() == "https" {
                    urls.insert(word.to_string());
                }
            }
        }
        urls
    }

    fn collect_titles(urls: HashSet<String>) -> HashMap<UrlString, Option<String>> {
        let mut result = HashMap::new();
        for url in urls {
            let title = Webpage::from_url(&url, WebpageOptions::default())
                .ok()
                .map(|info| info.html.title)
                .flatten();
            result.insert(url, title);
        }
        result
    }

    fn replace_with<F>(text: &str, format_link: F) -> String
    where
        F: Fn(&str, Option<&str>) -> String,
    {
        let urls = Self::collect(text);
        let urls_with_titles = Self::collect_titles(urls);
        let mut new_text = text.to_string();
        for (url, maybe_title) in urls_with_titles {
            new_text = new_text.replace(&url, &format_link(&url, maybe_title.as_deref()));
        }
        new_text
    }
}

fn youtube_music_url_formatter(url: &str, title: Option<&str>) -> String {
    if url.contains("music.youtube.com") {
        "MUSIC".to_string()
    } else {
        OrgModeFormatter::link(url, title.unwrap_or(url))
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let entry = OrgEntry::make(&args[1..]).expect("At least title should be provided");
    let new_entry = entry.apply_to_text(|t| {
        Url::replace_with(t, |url, maybe_title| {
            // youtube_music_url_formatter(url, maybe_title)
            OrgModeFormatter::link(url, maybe_title.unwrap_or(url))
        })
    });
    print!("{}", new_entry.to_orgmode());
}

// link1: https://music.youtube.com/watch?v=I-xfIz2kvkY&si=cyQzfNDY9GXvbSGI
// link2: https://docs.rs/chrono/latest/chrono/
// link3: https://music.youtube.com/watch?v=izKYbAowNQs&si=kMw3QEfS6C7BA9ja
