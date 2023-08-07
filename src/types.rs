use serde::Deserialize;
use serde_with::serde_as;
use serde_with::NoneAsEmptyString;

use super::request::Parameters;

structstruck::strike! {
    #[strikethrough[serde_as]]
    #[strikethrough[derive(Debug, Deserialize)]]
    #[serde(rename_all = "snake_case")]
    #[serde(tag = "result")]
    pub enum PreauthResponse {
        Auth {
            devices: Vec<pub struct Device {
                pub capabilities: Option<Vec<pub enum DeviceCapability {
                    #![derive(PartialEq, Eq, PartialOrd, Ord)]
                    #![serde(rename_all = "snake_case")]

                    Auto,
                    Push,
                    Sms,
                    Phone,
                    MobileOtp,
                }>>,
                pub device: String,
                pub display_name: Option<String>,
                #[serde_as(as = "NoneAsEmptyString")]
                pub name: Option<String>,
                #[serde_as(as = "NoneAsEmptyString")]
                pub number: Option<String>,
                pub sms_nextcode: Option<String>,
                pub r#type: pub enum DeviceType {
                    #![derive(PartialEq, Eq, PartialOrd, Ord)]
                    #![serde(rename_all = "snake_case")]

                    Phone,
                    Token,
                },

            }>,
        },
        Enroll {
            enroll_portal_url: String,
        },
        Allow,
        Deny,
    }
}

structstruck::strike! {
    #[strikethrough[serde_as]]
    #[strikethrough[derive(Deserialize, Debug)]]
    pub struct AuthStatusResponse {
        pub result: pub enum AuthResult {
            #![serde(rename_all = "snake_case")]

            Allow,
            Deny,
            Waiting,
        },
        pub status: pub enum AuthStatus {
            #![serde(rename_all = "snake_case")]

            Calling,
            Answered,
            Pushed,
            PushFailed,
            Timeout,
            Fraud,
            Allow,
            Bypass,
            Deny,
            LockedOut,
            Sent,
        },
        pub status_msg: String,
        pub trusted_device_token: Option<String>,
    }
}

impl AuthStatusResponse {
    pub fn ready(&self) -> Option<bool> {
        match self.result {
            AuthResult::Allow => Some(true),
            AuthResult::Deny => Some(false),
            AuthResult::Waiting => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum User {
    UserId { id: String },
    Username { username: String },
}

impl User {
    pub(crate) fn apply(self, parameters: &mut Parameters) {
        match self {
            Self::UserId { id } => parameters.set("user_id", id),
            Self::Username { username } => parameters.set("username", username),
        };
    }

    pub fn user_id<S: Into<String>>(id: S) -> Self {
        Self::UserId { id: id.into() }
    }

    pub fn username<S: Into<String>>(username: S) -> Self {
        Self::Username {
            username: username.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreauthRequest {
    pub user: User,
    pub ipaddr: Option<String>,
    pub hostname: Option<String>,
    pub trusted_device_token: Option<String>,
}

impl PreauthRequest {
    pub fn new(user: User) -> Self {
        Self {
            user,
            ipaddr: None,
            hostname: None,
            trusted_device_token: None,
        }
    }

    pub(crate) fn apply(self, parameters: &mut Parameters) {
        self.user.apply(parameters);
        parameters.set_opt("ipaddr", self.ipaddr);
        parameters.set_opt("hostname", self.hostname);
        parameters.set_opt("trusted_device_token", self.trusted_device_token);
    }
}

structstruck::strike! {
    #[strikethrough[derive(Clone, Debug)]]
    pub struct AuthRequest {
        pub user: User,
        pub factor: pub enum AuthRequestFactor {
            Auto {
                device: Option<String>,
                r#type: Option<String>,
                display_username: Option<String>,
                push_info: Option<String>,
            },
            Push {
                device: String,
                r#type: Option<String>,
                display_username: Option<String>,
                push_info: Option<String>,
            },
            Passcode { passcode: String },
            Phone { device: String },
            Sms { device: String },
        },
        pub ipaddr: Option<String>,
        pub hostname: Option<String>,
    }
}

impl AuthRequest {
    pub fn new(user: User, factor: AuthRequestFactor) -> Self {
        Self {
            user,
            factor,
            ipaddr: None,
            hostname: None,
        }
    }

    pub(crate) fn apply(self, parameters: &mut Parameters) {
        self.user.apply(parameters);
        self.factor.apply(parameters);
        parameters.set_opt("ipaddr", self.ipaddr);
        parameters.set_opt("hostname", self.hostname);
    }
}

impl AuthRequestFactor {
    pub fn auto() -> Self {
        Self::Auto {
            device: Some("auto".into()),
            r#type: None,
            display_username: None,
            push_info: None,
        }
    }

    pub(crate) fn apply(self, parameters: &mut Parameters) {
        match self {
            Self::Auto {
                device,
                r#type,
                display_username,
                push_info,
            } => {
                parameters.set("factor", "auto");
                parameters.set_opt("device", device);
                parameters.set_opt("type", r#type);
                parameters.set_opt("display_username", display_username);
                parameters.set_opt("push_info", push_info);
            }
            Self::Push {
                device,
                r#type,
                display_username,
                push_info,
            } => {
                parameters.set("factor", "push");
                parameters.set("device", device);
                parameters.set_opt("type", r#type);
                parameters.set_opt("display_username", display_username);
                parameters.set_opt("push_info", push_info);
            }
            Self::Passcode { passcode } => {
                parameters.set("factor", "passcode");
                parameters.set("passcode", passcode);
            }
            Self::Phone { device } => {
                parameters.set("factor", "phone");
                parameters.set("device", device);
            }
            Self::Sms { device } => {
                parameters.set("factor", "sms");
                parameters.set("device", device);
            }
        };
    }
}

#[derive(Debug, Deserialize)]
pub struct EnrollResponse {
    pub activation_barcode: String,
    pub activation_code: String,
    pub expiration: u64,
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollStatusResponse {
    Success,
    Invalid,
    Waiting,
}
