use chrono::Local;
use std::collections::{HashMap, HashSet};
use std::env;
use stripmargin::StripMargin;
use url::Url as UrlType;
use webpage::{Webpage, WebpageOptions};

struct ArgumentParser;

impl ArgumentParser {
    /// Accepts 1 or 2 command line arguments representing title and body of `OrgEntry`
    /// respectively
    fn parse(args: &[String]) -> Option<OrgEntry> {
        match args {
            [_, title] => Some(OrgEntry::new(title.clone(), "".to_string())),
            [_, title, body] => Some(OrgEntry::new(title.clone(), body.clone())),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct OrgEntry {
    pub title: String,
    pub body: String,
}

impl OrgEntry {
    fn new(title: String, body: String) -> Self {
        Self { title, body }
    }

    /// Applies given function to transform both title and body
    fn apply_to_text<F>(&self, f: F) -> Self
    where
        F: Fn(&str) -> String,
    {
        Self::new(f(&self.title), f(&self.body))
    }

    /// Renders OrgEntry as org mode task with created property containing current local time
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
    /// Formats link in org mode format
    fn link(url: &str, title: &str) -> String {
        format!("[[{}][{}]]", url, title)
    }
}

type UrlString = String;

type FormatterFn = Box<dyn Fn(&str, Option<&str>) -> Option<String>>;

pub struct Url;

impl Url {
    /// Collects all the http/https urls
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

    /// Collects titles to all the urls, if no title retrieved stores it as None
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

    /// Replaces all the urls in text with the first not None result of formatters (applied
    /// sequentially) or default_formatter if all of the formatters returned None
    pub fn apply_all_formatters(
        text: &str,
        formatters: &Vec<FormatterFn>,
        default_formatter: Box<dyn Fn(&str, Option<&str>) -> String>,
    ) -> String {
        let urls = Self::collect(text);
        let urls_with_titles = Self::collect_titles(urls);
        let mut new_text = text.to_string();
        for (url, maybe_title) in urls_with_titles {
            let mut formatted: Option<String> = None;
            for f in formatters {
                formatted = f(&url, maybe_title.as_deref());
                if formatted.is_some() {
                    break;
                }
            }
            if formatted.is_none() {
                formatted = Some(default_formatter(&url, maybe_title.as_deref()));
            }
            let formatted_link = formatted.expect("Link should be already formatted");
            new_text = new_text.replace(&url, &formatted_link);
        }
        new_text
    }
}

pub trait LinkFormatter {
    fn condition(url: &str) -> bool;
    fn format(url: &str, title: Option<&str>) -> String;
    fn apply(url: &str, title: Option<&str>) -> Option<String> {
        if Self::condition(url) {
            Some(Self::format(url, title))
        } else {
            None
        }
    }
}

// struct YoutubeMusicLinkFormatter;
//
// impl LinkFormatter for YoutubeMusicLinkFormatter {
//     fn condition(url: &str) -> bool {
//         url.contains("music.youtube.com")
//     }
//
//     fn format(url: &str, title: Option<&str>) -> String {
//         format!("MUSIC: {}, {}", url, title.unwrap_or_default())
//     }
// }

fn default_formatter(url: &str, maybe_title: Option<&str>) -> String {
    OrgModeFormatter::link(url, maybe_title.unwrap_or(url))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let entry = ArgumentParser::parse(&args).expect("Exactly 1 or 2 arguments should be provided");
    let formatters: Vec<FormatterFn> = vec![
        // Box::new(YoutubeMusicLinkFormatter::apply)
    ];
    let new_entry = entry
        .apply_to_text(|t| Url::apply_all_formatters(t, &formatters, Box::new(default_formatter)));
    print!("{}", new_entry.to_orgmode());
}
