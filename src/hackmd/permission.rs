pub enum Read {
    Owner,
    SignedIn,
    Guest,
}

pub enum Write {
    Owner,
    SignedIn,
    Guest,
}

pub enum Comment {
    Disabled,
    Forbidden,
    Owners,
    SignedInUsers,
    Everyone,
}
