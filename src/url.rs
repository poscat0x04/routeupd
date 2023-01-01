use url::{ParseError, Url};

pub fn parse_url(url: &str) -> Result<Url, ParseError> {
    Url::parse(url).or_else(|_| {
        Url::parse(&("https://".to_owned() + url))
    })
}

#[cfg(test)]
mod test {
    use crate::url::parse_url;

    #[test]
    fn test_parse_url() {
        let gh_abbrev = parse_url("github.com");
        assert_eq!(gh_abbrev.unwrap().as_str(), "https://github.com/");

        let gh_full = parse_url("https://github.com/linus");
        assert_eq!(gh_full.unwrap().as_str(), "https://github.com/linus");
    }
}
