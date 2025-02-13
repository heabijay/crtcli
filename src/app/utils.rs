use thiserror::Error;

#[derive(Error, Debug)]
#[error("response cookie has invalid format, parsing failed")]
pub struct CookieParsingError;

pub fn iter_set_cookies(
    response: &reqwest::Response,
) -> impl Iterator<Item = Result<(&str, &str), CookieParsingError>> {
    response
        .headers()
        .iter()
        .filter(|(name, _)| *name == "set-cookie")
        .map(|(_, value)| {
            value
                .to_str()
                .map_err(|_| CookieParsingError)?
                .split_once(";")
                .ok_or(CookieParsingError)?
                .0
                .split_once("=")
                .ok_or(CookieParsingError)
        })
}

pub fn collect_set_cookies(
    response: &reqwest::Response,
) -> Result<Vec<(String, String)>, CookieParsingError> {
    iter_set_cookies(response)
        .map(|x| x.map(|(name, value)| (name.to_owned(), value.to_owned())))
        .collect::<Result<Vec<_>, CookieParsingError>>()
}

pub fn find_cookie_by_name(set_cookies: &[(String, String)], name: &str) -> Option<String> {
    set_cookies
        .iter()
        .find(|(cookie_name, _)| *cookie_name == name)
        .map(|(_, value)| value.to_owned())
}
