use std::{borrow::Cow, string::FromUtf8Error};

use rand::prelude::*;
use rand_regex::Regex;
use thiserror::Error;
use url::{ParseError, Url};

#[derive(Clone, Debug)]
pub enum UrlGenerator {
    Static(Url),
    Dynamic(Regex),
}

#[derive(Error, Debug)]
pub enum UrlGeneratorError {
    #[error("{0}, generated url: {1}")]
    ParseError(ParseError, String),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
}

impl UrlGenerator {
    pub fn new_static(url: Url) -> Self {
        Self::Static(url)
    }

    pub fn new_dynamic(regex: Regex) -> Self {
        Self::Dynamic(regex)
    }

    pub fn generate<R: Rng>(&self, rng: &mut R) -> Result<Cow<Url>, UrlGeneratorError> {
        match self {
            Self::Static(url) => Ok(Cow::Borrowed(url)),
            Self::Dynamic(regex) => {
                let generated = Distribution::<Result<String, FromUtf8Error>>::sample(regex, rng)?;
                Ok(Cow::Owned(Url::parse(generated.as_str()).map_err(|e| {
                    UrlGeneratorError::ParseError(e, generated)
                })?))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_regex::Regex as RandRegex;
    use regex::Regex;
    use std::net::Ipv4Addr;
    use url::{Host, Url};

    #[test]
    fn test_url_generator_static() {
        let url_generator = UrlGenerator::new_static(Url::parse("http://127.0.0.1/test").unwrap());
        let url = url_generator.generate(&mut thread_rng()).unwrap();
        assert_eq!(url.host(), Some(Host::Ipv4(Ipv4Addr::new(127, 0, 0, 1))));
        assert_eq!(url.path(), "/test");
    }

    #[test]
    fn test_url_generator_dynamic() {
        let path_regex = "/[a-z][a-z][0-9]";
        let url_generator = UrlGenerator::new_dynamic(
            RandRegex::compile(&format!(r"http://127\.0\.0\.1{}", path_regex), 4).unwrap(),
        );
        let url = url_generator.generate(&mut thread_rng()).unwrap();
        assert_eq!(url.host(), Some(Host::Ipv4(Ipv4Addr::new(127, 0, 0, 1))));
        assert!(Regex::new(path_regex)
            .unwrap()
            .captures(url.path())
            .is_some());
    }
}
