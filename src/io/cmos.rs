use bitflags::bitflags;
use derive_more::{Add, AddAssign, Display, From, Into, Sub, SubAssign};
use port::{Port, PortWriteOnly};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
    From,
    Into,
    Add,
    AddAssign,
    Sub,
    SubAssign,
)]
pub struct Second(u16);
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
    From,
    Into,
    Add,
    AddAssign,
    Sub,
    SubAssign,
)]
pub struct Minute(u16);
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
    From,
    Into,
    Add,
    AddAssign,
    Sub,
    SubAssign,
)]
pub struct Hour(u16);
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
    From,
    Into,
    Add,
    AddAssign,
    Sub,
    SubAssign,
)]
pub struct Day(u16);

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
    From,
    Into,
    Add,
    AddAssign,
    Sub,
    SubAssign,
)]
pub struct Month(u16);

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Display,
    From,
    Into,
    Add,
    AddAssign,
    Sub,
    SubAssign,
)]
pub struct Year(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
#[repr(u8)]
pub enum Weekday {
    Sunday = 1,
    Monday = 2,
    Tuesday = 3,
    Wednesday = 4,
    Thursday = 5,
    Friday = 6,
    Saturday = 7,
}

impl From<u8> for Weekday {
    fn from(value: u8) -> Self {
        match value {
            1 => Weekday::Sunday,
            2 => Weekday::Monday,
            3 => Weekday::Tuesday,
            4 => Weekday::Wednesday,
            5 => Weekday::Thursday,
            6 => Weekday::Friday,
            7 => Weekday::Saturday,
            _ => panic!("Invalid"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
#[display(
    fmt = "{} {} {} {}:{}:{} {}",
    weekday,
    month,
    day,
    hour,
    minute,
    second,
    year
)]
pub struct RtcDate {
    second: Second,
    minute: Minute,
    hour: Hour,
    day: Day,
    month: Month,
    year: Year,
    weekday: Weekday,
}

impl RtcDate {
    /// Create an empty date (everything as a zero)
    pub fn empty() -> Self {
        Self {
            second: Second(0),
            minute: Minute(0),
            hour: Hour(0),
            day: Day(0),
            month: Month(0),
            year: Year(0),
            weekday: Weekday::Sunday,
        }
    }

    /// Create a new date that reads from the cmos (this is somewhat expensive)
    pub fn cmos_read() -> Self {
        let mut cmos = Cmos::default();

        Self {
            second: Second(cmos.read(Time::SECS).into()),
            minute: Minute(cmos.read(Time::MINS).into()),
            hour: Hour(cmos.read(Time::HOURS).into()),
            day: Day(cmos.read(Time::DAY).into()),
            month: Month(cmos.read(Time::MONTH).into()),
            year: Year(cmos.read(Time::YEAR).into()),
            weekday: cmos.read(Time::WEEKDAY).into(),
        }
    }

    /// Convert from bsd encoded data
    pub fn bsd_convert(&mut self) {
        self.second = Second(Self::convert(self.second.into()));
        self.minute = Minute(Self::convert(self.minute.into()));
        self.hour = Hour(Self::convert(self.hour.into()));
        self.day = Day(Self::convert(self.day.into()));
        self.month = Month(Self::convert(self.month.into()));
        self.year = Year(Self::convert(self.year.into()));

        self.year += Year(2000);
    }

    /// Convert function
    const fn convert(value: u16) -> u16 {
        ((value >> 4) * 10) + (value & 0xf)
    }
}

bitflags! {
    pub struct Time: u8 {
        const SECS = 0x00;
        const MINS = 0x02;
        const HOURS = 0x04;
        const WEEKDAY = 0x06;
        const DAY = 0x07;
        const MONTH = 0x08;
        const YEAR = 0x09;

        // other
        const STATA = 0x0A;
        const STATB = 0x0b;
        const UIP = 1 << 7;
    }
}

pub struct Cmos {
    command: PortWriteOnly<u8>,
    data: Port<u8>,
}

impl Cmos {
    pub const fn new() -> Self {
        const PORT_NUM: u16 = 0x70;
        Self {
            command: PortWriteOnly::new(PORT_NUM),
            data: Port::new(PORT_NUM + 1),
        }
    }

    pub fn init(&mut self) {
        unsafe { self.command.write(0x80) };
    }

    pub fn read(&mut self, time: Time) -> u8 {
        unsafe {
            self.command.write(time.bits());
            //super::micro_delay(200);
            self.data.read()
        }
    }

    pub fn is_updating(&mut self) -> bool {
        self.read(Time::STATB) & Time::UIP.bits() != 0
    }

    pub fn is_bcd(&mut self) -> bool {
        let sb = self.read(Time::STATB);
        (sb & (1 << 2)) == 0
    }

    pub fn time(&mut self) -> RtcDate {
        let mut t1;

        loop {
            t1 = RtcDate::cmos_read();
            if self.is_updating() {
                continue;
            }
            let t2 = RtcDate::cmos_read();
            if t1 == t2 {
                break;
            }
        }

        if self.is_bcd() {
            t1.bsd_convert();
        }

        t1
    }

    /// Set the cmos reset value to "warm start with far just"
    pub fn warm_reset(&mut self) {
        unsafe {
            self.command.write(0x0F);
            self.data.write(0x0A);
        }
    }
}

impl Default for Cmos {
    fn default() -> Self {
        Self::new()
    }
}
