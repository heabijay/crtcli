use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CrtSession {
    aspxauth: String,
    bpmcsrf: String,
    csrftoken: Option<String>,
    bpmsessionid: Option<String>,
}

impl CrtSession {
    pub fn new(
        aspxauth: String,
        bpmcsrf: String,
        csrftoken: Option<String>,
        bpmsessionid: Option<String>,
    ) -> CrtSession {
        Self {
            aspxauth,
            bpmcsrf,
            csrftoken,
            bpmsessionid,
        }
    }

    pub fn to_cookie_value(&self) -> String {
        let mut cookie_value =
            format!(".ASPXAUTH={};BPMCSRF={};", self.aspxauth, self.bpmcsrf).to_owned();

        if let Some(csrftoken) = &self.csrftoken {
            cookie_value = format!("{}CsrfToken={};", cookie_value, csrftoken);
        }

        if let Some(bpmsessionid) = &self.bpmsessionid {
            cookie_value = format!("{}BPMSESSIONID={};", cookie_value, bpmsessionid);
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
