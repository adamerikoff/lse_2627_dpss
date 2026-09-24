#[derive(Debug, Clone, Copy)]
pub(crate) enum Mode {
    Secure,
    Insecure,
}

impl Mode {
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        match raw {
            "s" | "secure" => Some(Self::Secure),
            "i" | "insecure" => Some(Self::Insecure),
            _ => None,
        }
    }
}
