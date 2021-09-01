use crate::utils::seeker::*;

pub enum SignalResponse<T> {
    Ignore(T),
    Toggle(T, bool),
    Commence(T, bool),
    Halt(T, bool),
}

impl<T> SignalResponse<T> {
    pub fn respond(&mut self) {
        match self {
            Self::Toggle(_, ref mut b) => *b = !*b,
            Self::Commence(_, ref mut b) => *b = true,
            Self::Halt(_, ref mut b) => *b = false,
            _ => {}
        }
    }

    pub fn unwrap(&self) -> &T {
        match self {
            Self::Ignore(ref val) 
            | Self::Toggle(ref val, _)
            | Self::Commence(ref val, _)
            | Self::Halt(ref val, _) => val
        }
    }

    pub fn unwrap_mut(&mut self) -> &mut T {
        match self {
            Self::Ignore(ref mut val) 
            | Self::Toggle(ref mut val, _)
            | Self::Commence(ref mut val, _)
            | Self::Halt(ref mut val, _) => val
        }
    }

}

pub type Channel<T> = Vec<Epoch<T>>;

pub struct PlayList<T> {
    channels: Vec<Channel<SignalResponse<T>>>,
}
