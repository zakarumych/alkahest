use std::{ops::Deref, rc::Rc};

// Cheaply cloneable buffer for parsing.
#[derive(Clone)]
pub(crate) struct Buffer {
    string: Rc<String>,
}

impl Buffer {
    pub fn from_string(s: String) -> Self {
        Buffer { string: Rc::new(s) }
    }

    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }
}

impl Deref for Buffer {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Buffer {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
