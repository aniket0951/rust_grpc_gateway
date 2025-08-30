use core::fmt;

pub enum ResponseErrors {
    Success,
    ServiceUnAvailable,
    ServiceNotRegister(String),
    TransportFailure,
}

impl fmt::Display for ResponseErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = match self {
            Self::Success => String::from("api call has been done"),
            Self::ServiceUnAvailable => String::from("service unavailable"),
            Self::ServiceNotRegister(service_name) => format!(
                "{} {}",
                service_name,
                String::from("is not register, please register the sevice")
            ),
            Self::TransportFailure => String::from("Unknown transport failure"),
        };

        write!(f, "{}", data)
    }
}

pub enum ResponseSuccess {
    ServiceRegisterSuccessfully(String),
}

impl fmt::Display for ResponseSuccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = match self {
            Self::ServiceRegisterSuccessfully(service_name) => {
                format!("{} has been register successfully", service_name)
            }
        };
        write!(f, "{}", data)
    }
}
