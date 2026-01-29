use alloc::{rc::Rc, string::String};

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

    pub fn len(&self) -> usize {
        self.string.len()
    }
}
