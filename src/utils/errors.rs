use core::fmt;
use std::borrow::Cow;

pub enum ResponseErrors {
    Success,
    ServiceUnAvailable,
    ServiceNotRegister(String),
    TransportFailure,
    Error,
    OAuthRefreshConfigMissingError,
    InternalServerError,
}

impl ResponseErrors {
    pub fn message(&self) -> Cow<'static, str> {
        match self {
            ResponseErrors::Success => Cow::Borrowed("success"),
            ResponseErrors::ServiceUnAvailable => Cow::Borrowed("service unavailable"),
            ResponseErrors::ServiceNotRegister(service_name) => Cow::Owned(format!(
                "{} {}",
                service_name,
                String::from("is not register, please register the sevice")
            )),
            ResponseErrors::TransportFailure => Cow::Borrowed("Unknown transport failure"),
            ResponseErrors::Error => Cow::Borrowed("error"),
            ResponseErrors::OAuthRefreshConfigMissingError => {
                Cow::Borrowed("oauth refresh config is missing")
            }
            ResponseErrors::InternalServerError => Cow::Borrowed("internal server error"),
        }
    }
}

impl fmt::Display for ResponseErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let data = match self {
        //     Self::Success => String::from("success"),
        //     Self::ServiceUnAvailable => String::from("service unavailable"),
        //     Self::ServiceNotRegister(service_name) => format!(
        //         "{} {}",
        //         service_name,
        //         String::from("is not register, please register the sevice")
        //     ),
        //     Self::TransportFailure => String::from("Unknown transport failure"),
        //     Self::Error => String::from("error"),
        //     Self::OAuthRefreshConfigMissingError => String::from("oauth refresh config is missing"),
        //     Self::InternalServerError => String::from("internal server error"),
        // };
        //
        // write!(f, "{}", data)

        f.write_str(&self.message())
    }
}

pub enum ResponseSuccess {
    ServiceRegisterSuccessfully(String),
}
impl ResponseSuccess {
    pub fn message(&self) -> Cow<'static, str> {
        match self {
            ResponseSuccess::ServiceRegisterSuccessfully(service_name) => {
                Cow::Owned(format!("{} has been register successfully", service_name))
            }
        }
    }
}
impl fmt::Display for ResponseSuccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}
