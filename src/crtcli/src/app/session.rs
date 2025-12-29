use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub enum CrtSession {
    Cookie(CrtSessionCookie),
    OAuthSession(CrtSessionOAuth),
}

#[derive(Debug, Clone, Eq, PartialEq, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct CrtSessionCookie {
    aspxauth: String,
    bpmcsrf: String,
    csrftoken: Option<String>,
    bpmsessionid: Option<String>,
}

#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
)]
pub struct CrtSessionOAuth {
    access_token: String,
    expires_in: i64,
    token_type: String,
}

impl CrtSessionCookie {
    pub fn new(
        aspxauth: String,
        bpmcsrf: String,
        csrftoken: Option<String>,
        bpmsessionid: Option<String>,
    ) -> CrtSessionCookie {
        CrtSessionCookie {
            aspxauth,
            bpmcsrf,
            csrftoken,
            bpmsessionid,
        }
    }
}

impl CrtSessionCookie {
    pub fn to_cookie_value(&self) -> String {
        let mut cookie_value =
            format!(".ASPXAUTH={};BPMCSRF={};", self.aspxauth, self.bpmcsrf).to_owned();

        if let Some(csrftoken) = &self.csrftoken {
            cookie_value = format!("{cookie_value}CsrfToken={csrftoken};");
        }

        if let Some(bpmsessionid) = &self.bpmsessionid {
            cookie_value = format!("{cookie_value}BPMSESSIONID={bpmsessionid};");
        }

        cookie_value
    }

    pub fn bpmcsrf(&self) -> &str {
        &self.bpmcsrf
    }

    pub fn bpmsessionid(&self) -> Option<&str> {
        self.bpmsessionid.as_deref()
    }

    pub fn set_bpmsessionid(&mut self, bpmsessionid: Option<String>) {
        self.bpmsessionid = bpmsessionid;
    }
}

impl CrtSessionOAuth {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn token_type(&self) -> &str {
        &self.token_type
    }

    pub fn expires_in(&self) -> i64 {
        self.expires_in
    }
}
