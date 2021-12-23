use core::pin::Pin;
use core::task::{Context, Poll};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::Stream;
use futures_util::task::AtomicWaker;
use futures_util::StreamExt;
use port::Port;
use spin::lazy::Lazy;

const NO: u8 = 0;

// Special keycodes
const KEY_HOME: u8 = 0xE0;
const KEY_END: u8 = 0xE1;
const KEY_UP: u8 = 0xE2;
const KEY_DN: u8 = 0xE3;
const KEY_LF: u8 = 0xE4;
const KEY_RT: u8 = 0xE5;
const KEY_PGUP: u8 = 0xE6;
const KEY_PGDN: u8 = 0xE7;
const KEY_INS: u8 = 0xE8;
const KEY_DEL: u8 = 0xE9;

const NORMAL_MAP: [u8; 256] = [
    // 0x00
    NO, 0x1B, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', b'\x08',
    b'\t', // 0x10
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', b'\n', NO, b'a', b's',
    // 0x20
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`', NO, b'\\', b'z', b'x', b'c', b'v',
    // 0x30
    b'b', b'n', b'm', b',', b'.', b'/', NO, b'*', NO, b' ', NO, NO, NO, NO, NO, NO, // 0x40
    NO, NO, NO, NO, NO, NO, NO, b'7', b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1',
    // 0x50
    b'2', b'3', b'0', b'.', NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x60
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x70
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x80
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x90
    NO, NO, NO, NO, NO, NO, NO, KEY_HOME, NO, NO, NO, NO, b'\n', NO, NO, NO, // 0xA0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xB0
    NO, NO, NO, NO, NO, b'/', NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xC0
    NO, NO, NO, NO, NO, NO, NO, NO, KEY_UP, KEY_PGUP, NO, KEY_LF, NO, KEY_RT, NO, KEY_END,
    // 0xD0
    KEY_DN, KEY_PGDN, KEY_INS, KEY_DEL, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO,
    // 0xE0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xF0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x100
];

const SHIFT_MAP: [u8; 256] = [
    // 0x00
    NO, 27, b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'_', b'+', b'\x08',
    b'\t', // 0x10
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P', b'{', b'}', b'\n', NO, b'A', b'S',
    // 0x20
    b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':', b'"', b'~', NO, b'|', b'Z', b'X', b'C', b'V',
    // 0x30
    b'B', b'N', b'M', b'<', b'>', b'?', NO, b'*', NO, b' ', NO, NO, NO, NO, NO, NO, // 0x40
    NO, NO, NO, NO, NO, NO, NO, b'7', b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1',
    // 0x50
    b'2', b'3', b'0', b'.', NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x60
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x70
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x80
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x90
    NO, NO, NO, NO, NO, NO, NO, KEY_HOME, NO, NO, NO, NO, b'\n', NO, NO, NO, // 0xA0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xB0
    NO, NO, NO, NO, NO, b'/', NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xC0
    NO, NO, NO, NO, NO, NO, NO, NO, KEY_UP, KEY_PGUP, NO, KEY_LF, NO, KEY_RT, NO, KEY_END,
    // 0xD0
    KEY_DN, KEY_PGDN, KEY_INS, KEY_DEL, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO,
    // 0xE0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0xF0
    NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, NO, // 0x100
];

static SCANCODE_QUEUE: Lazy<ArrayQueue<u8>> = Lazy::new(|| ArrayQueue::new(100));
static WAKER: AtomicWaker = AtomicWaker::new();

pub fn add_scancode(scancode: u8) {
    let queue = &SCANCODE_QUEUE;
    if queue.push(scancode).is_err() {
        crate::kprintln!("WARNING: scancode queue full; dropping keyboard input")
    } else {
        WAKER.wake();
    }
}

pub struct ScancodeStream {
    _priavte: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        assert_eq!(100, SCANCODE_QUEUE.capacity());
        Self { _priavte: () }
    }
}

impl Default for ScancodeStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = &SCANCODE_QUEUE;

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());
        match queue.pop() {
            Some(scancode) => Poll::Ready(Some(scancode)),
            None => Poll::Pending,
        }
    }
}

pub struct Keyboard {
    data: Port<u8>,
    status: Port<u8>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            data: Port::new(0x60),
            status: Port::new(0x64),
        }
    }

    fn read(&self) -> u8 {
        unsafe { self.data.read() }
    }

    pub fn get_key(&self) -> Option<u8> {
        let scan_code = self.read();

        let key = NORMAL_MAP[scan_code as usize];
        Some(key)
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();

    while let Some(scancode) = scancodes.next().await {
        let c = char::from(scancode);
        crate::kprint!("{}", c);
    }
}
