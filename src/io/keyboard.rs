use bitflags::bitflags;
use conquer_once::spin::OnceCell;
use core::ops::Index;
use core::pin::Pin;
use core::sync::atomic::{AtomicU8, Ordering};
use core::task::{Context, Poll};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::Stream;
use futures_util::task::AtomicWaker;
use futures_util::StreamExt;
use port::Port;

const NONE: u8 = 0;

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

#[rustfmt::skip]
const NORMAL_MAP: [u8; 256] = [
    NONE, 0x1B, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', b'\x08', b'\t', // 0x10
    b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', b'\n', NONE, b'a', b's', // 0x20
    b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`', NONE, b'\\', b'z', b'x', b'c', b'v', // 0x30
    b'b', b'n', b'm', b',', b'.', b'/', NONE, b'*', NONE, b' ', NONE, NONE, NONE, NONE, NONE, NONE,   // 0x40
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, b'7', b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1', // 0x50
    b'2', b'3', b'0', b'.', NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0x60
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0x70
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0x80
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0x90
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, KEY_HOME, NONE, NONE, NONE, NONE, b'\n', NONE, NONE, NONE, // 0xA0
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0xB0
    NONE, NONE, NONE, NONE, NONE, b'/', NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0xC0
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, KEY_UP, KEY_PGUP, NONE, KEY_LF, NONE, KEY_RT, NONE, KEY_END, // 0xD0
    KEY_DN, KEY_PGDN, KEY_INS, KEY_DEL, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0xE0
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0xF0
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0x100
];

#[rustfmt::skip]
const SHIFT_MAP: [u8; 256] = [
    NONE, 27, b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'_', b'+', b'\x08', b'\t', // 0x10
    b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P', b'{', b'}', b'\n', NONE, b'A', b'S', // 0x20
    b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':', b'"', b'~', NONE, b'|', b'Z', b'X', b'C', b'V', // 0x30
    b'B', b'N', b'M', b'<', b'>', b'?', NONE, b'*', NONE, b' ', NONE, NONE, NONE, NONE, NONE, NONE, // 0x40
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, b'7', b'8', b'9', b'-', b'4', b'5', b'6', b'+', b'1', // 0x50
    b'2', b'3', b'0', b'.', NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0x60
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0x70
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0x80
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0x90
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, KEY_HOME, NONE, NONE, NONE, NONE, b'\n', NONE, NONE, NONE, // 0xA0
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0xB0
    NONE, NONE, NONE, NONE, NONE, b'/', NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0xC0
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, KEY_UP, KEY_PGUP, NONE, KEY_LF, NONE, KEY_RT, NONE, KEY_END, // 0xD0
    KEY_DN, KEY_PGDN, KEY_INS, KEY_DEL, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0xE0
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0xF0
    NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, NONE, // 0x100
];

bitflags! {
    struct KeyboardState: u8 {
        const NO = 0;
        // modifiers
        const SHIFT = 1<<0;
        const CTRL = 1<<1;
        const ALT = 1<<2;
        // locks
        const CAPSLOCK = 1<<3;
        const NUMLOCK = 1<<4;
        const SCROLLLOCK = 1<<5;
        // Esc
        const E0ESC = 1<<6;
    }
}

impl Index<u8> for KeyboardState {
    type Output = KeyboardState;
    fn index(&self, index: u8) -> &Self::Output {
        match index {
            0x1D => &Self::CTRL,
            0x2A => &Self::SHIFT,
            0x36 => &Self::SHIFT,
            0x38 => &Self::ALT,
            0x9D => &Self::CTRL,
            0xB8 => &Self::ALT,
            // toggle codes
            0x3A => &Self::CAPSLOCK,
            0x45 => &Self::NUMLOCK,
            0x46 => &Self::SCROLLLOCK,
            _ => panic!("invalid scancode"),
        }
    }
}

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            crate::kprintln!("WARNING: scancode queue full; dropping keyboard input")
        } else {
            WAKER.wake();
        }
    } else {
        crate::kprintln!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    _priavte: (), // disallows manual struct creation
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new() should only be called once");
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
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("ScancodeStream: not initialized");

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

    pub fn get_scancode(&self) -> Option<u8> {
        let scancode = self.read();
        Some(scancode)
    }

    pub fn parse_scancode(scancode: u8) -> Option<char> {
        static MODIFER_STATE: AtomicU8 = AtomicU8::new(0);

        let mut scancode = scancode;

        if scancode == 0xE0 {
            MODIFER_STATE.fetch_or(KeyboardState::E0ESC.bits, Ordering::Relaxed);
            return None;
        } else if scancode & 0x80 != 0 {
            // key released
            if MODIFER_STATE.load(Ordering::Relaxed) & KeyboardState::E0ESC.bits == 0 {
                scancode &= 0x7F;
            }
            let mut shift_code = if scancode == 0x1D
                || scancode == 0x2A
                || scancode == 0x36
                || scancode == 0x38
                || scancode == 0x9D
                || scancode == 0xB8
            {
                KeyboardState::empty()[scancode].bits
            } else {
                0
            };
            shift_code = !(shift_code | KeyboardState::E0ESC.bits);
            MODIFER_STATE.fetch_and(shift_code, Ordering::Relaxed);
            return None;
        } else if MODIFER_STATE.load(Ordering::Relaxed) & KeyboardState::E0ESC.bits != 0 {
            scancode |= 0x80;
            MODIFER_STATE.fetch_and(!KeyboardState::E0ESC.bits, Ordering::Relaxed);
        }

        let shift_code = if scancode == 0x1D
            || scancode == 0x2A
            || scancode == 0x36
            || scancode == 0x38
            || scancode == 0x9D
            || scancode == 0xB8
        {
            KeyboardState::empty()[scancode].bits
        } else {
            0
        };

        let toggle_code = if scancode == 0x3A || scancode == 0x45 || scancode == 0x46 {
            KeyboardState::empty()[scancode].bits
        } else {
            0
        };

        MODIFER_STATE.fetch_or(shift_code, Ordering::Relaxed);
        MODIFER_STATE.fetch_xor(toggle_code, Ordering::Relaxed);

        let index = MODIFER_STATE.load(Ordering::Relaxed)
            & (KeyboardState::CTRL | KeyboardState::SHIFT).bits;

        let mut c = match index {
            0 => NORMAL_MAP[scancode as usize],
            1 => SHIFT_MAP[scancode as usize],
            2 => {
                crate::kprintln!("ctrl not implemented");
                return None;
            }
            3 => todo!(),
            _ => unreachable!(),
        };

        if (MODIFER_STATE.load(Ordering::Relaxed) & KeyboardState::CAPSLOCK.bits) != 0 {
            if (b'a'..=b'z').contains(&c) {
                c = c.wrapping_add(b'A'.wrapping_sub(b'a'));
            } else if (b'A'..=b'Z').contains(&c) {
                c += b'a' - b'A';
            }
        }

        Some(char::from(c))
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
        if let Some(c) = Keyboard::parse_scancode(scancode) {
            crate::kprint!("{}", c);
        }
    }
}
