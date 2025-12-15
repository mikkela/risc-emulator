use std::fmt;
use arboard::Clipboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State { Idle, Get, Put }

pub struct ClipboardDevice {
    state: State,
    data: Vec<u8>,
    ptr: usize,
    cb: Clipboard,
}
impl fmt::Debug for ClipboardDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClipboardDevice")
            .field("state", &self.state)
            .field("data_len", &self.data.len())
            .field("ptr", &self.ptr)
            .finish()
    }
}

impl ClipboardDevice {
    pub fn new() -> Self {
        Self {
            state: State::Idle,
            data: Vec::new(),
            ptr: 0,
            cb: Clipboard::new().expect("clipboard"),
        }
    }

    fn reset(&mut self) {
        self.state = State::Idle;
        self.data.clear();
        self.ptr = 0;
    }

    // IOStart+40 read: return length, and prepare GET
    pub fn read_control(&mut self) -> u32 {
        self.reset();
        let Ok(txt) = self.cb.get_text() else { return 0; };
        if txt.is_empty() { return 0; }

        // Oberon forventer CR, og C-koden normaliserer CRLF
        let bytes = txt.into_bytes();

        // Hvis du vil lave samme CRLF-justering som C:
        // C: mindsker reported length hvis CRLF
        // Her: vi kan bare gøre det simpelt: send bytes som de er,
        // og i read_data normaliserer vi '\n' -> '\r'.
        self.data = bytes;
        self.state = State::Get;
        self.data.len().min(u32::MAX as usize) as u32
    }

    // IOStart+40 write: len => prepare PUT buffer
    pub fn write_control(&mut self, len: u32) {
        self.reset();
        if len == u32::MAX { return; }
        self.data = vec![0u8; len as usize];
        self.state = State::Put;
    }

    // IOStart+44 read: return next byte
    pub fn read_data(&mut self) -> u32 {
        if self.state != State::Get || self.ptr >= self.data.len() { return 0; }
        let mut c = self.data[self.ptr];
        self.ptr += 1;

        // C: '\n' -> '\r'
        if c == b'\n' { c = b'\r'; }

        if self.ptr >= self.data.len() {
            self.reset();
        }
        c as u32
    }

    // IOStart+44 write: accept bytes, commit at end
    pub fn write_data(&mut self, c: u32) {
        if self.state != State::Put || self.ptr >= self.data.len() { return; }
        let mut ch = (c & 0xFF) as u8;
        // C: '\r' -> '\n'
        if ch == b'\r' { ch = b'\n'; }
        self.data[self.ptr] = ch;
        self.ptr += 1;

        if self.ptr >= self.data.len() {
            if let Ok(s) = String::from_utf8(self.data.clone()) {
                let _ = self.cb.set_text(s);
            }
            self.reset();
        }
    }
}