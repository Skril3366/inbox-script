use linkify::{LinkFinder, LinkKind};
use url::Url as UrlType;
use webpage::{Webpage, WebpageOptions};

#[derive(Debug)]
struct Url(String);

impl Url {
    fn is_url(maybe_url: &String) -> bool {
        UrlType::parse(maybe_url).is_ok()
    }
    fn new(url: String) -> Option<Self> {
        if !Self::is_url(&url) {
            None
        } else {
            Some(Self(url))
        }
    }
    fn get_title(&self) -> Option<String> {
        Webpage::from_url(&self.0, WebpageOptions::default())
            .ok()
            .map(|info| info.html.title)
            .flatten()
    }
}

#[derive(Debug)]
struct OrgEntry {
    name: String,
    body: String,
}

impl OrgEntry {
    fn new(name: String, body: String) -> Self {
        Self { name, body }
    }
    // fn get_all_urls(&self) -> Vec<Url> {
    //     let mut urls = Vec::new();
    //     for line in self.body.lines() {
    //         if let Some(url) = Self::get_url(line) {
    //             urls.push(url);
    //         }
    //     }
    //     urls
    // }
}

fn main() {
    // let org_entry = OrgEntry::new("Hello".to_string(), "World".to_string());
    let url: Option<Url> = Url::new(
        "https://music.youtube.com/playlist?list=OLAK5uy_mFI5RfOkB5qx-eLvCgd198khWA-wdU1iI"
            .to_string(),
    );
    let title = url.map(|u| u.get_title()).flatten();
    dbg!(title);

    let input = "http://example.com and foo@example.com";
    let mut finder = LinkFinder::new();
    finder.kinds(&[LinkKind::Url]);
    let links: Vec<_> = finder.links(input).collect();
    links.iter().map(|l| l.as_str());

    // assert_eq!(1, links.len());
    // let link = &links[0];
    // assert_eq!("http://example.com", link.as_str());
    // assert_eq!(&LinkKind::Email, link.kind());
}
