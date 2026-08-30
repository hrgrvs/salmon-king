//! One tick = one tide. Two tides per calendar day (flood, then ebb).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tide {
    Flood,
    Ebb,
}

impl Tide {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Flood => "flood",
            Self::Ebb => "ebb",
        }
    }
}

const MDAYS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GameDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl GameDate {
    pub fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    pub fn add_days(self, days: i32) -> Self {
        let mut y = self.year;
        let mut m = self.month;
        let mut d = self.day as i32 + days;
        loop {
            let dim = days_in_month(y, m) as i32;
            if d > dim {
                d -= dim;
                m += 1;
                if m > 12 {
                    m = 1;
                    y += 1;
                }
            } else if d < 1 {
                if m == 1 {
                    m = 12;
                    y -= 1;
                } else {
                    m -= 1;
                }
                d += days_in_month(y, m) as i32;
            } else {
                break;
            }
        }
        Self {
            year: y,
            month: m,
            day: d as u8,
        }
    }

    pub fn days_until(self, other: GameDate) -> i32 {
        other.to_ordinal() - self.to_ordinal()
    }

    fn to_ordinal(self) -> i32 {
        let mut n = 0;
        for y in 1..self.year {
            n += if is_leap(y) { 366 } else { 365 };
        }
        n + self.doy()
    }

    pub fn doy(self) -> i32 {
        let mut n = self.day as i32;
        for m in 1..self.month {
            n += days_in_month(self.year, m) as i32;
        }
        n
    }

    /// `%d %b %Y` — 01 Jun 2025
    pub fn fmt_short(self) -> String {
        format!("{:02} {} {}", self.day, MONTHS[self.month as usize - 1], self.year)
    }

    /// `%a %d %b %Y` — Sun 01 Jun 2025
    pub fn fmt_long(self) -> String {
        format!(
            "{} {:02} {} {}",
            WEEKDAYS[self.weekday() as usize],
            self.day,
            MONTHS[self.month as usize - 1],
            self.year
        )
    }

    /// `%m/%d`
    pub fn fmt_md(self) -> String {
        format!("{:02}/{:02}", self.month, self.day)
    }

    /// Sakamoto: 0 = Sunday.
    pub fn weekday(self) -> u8 {
        let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let mut y = self.year;
        if self.month < 3 {
            y -= 1;
        }
        ((y + y / 4 - y / 100 + y / 400 + t[self.month as usize - 1] + self.day as i32) % 7) as u8
    }
}

fn is_leap(y: i32) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

fn days_in_month(y: i32, m: u8) -> u8 {
    if m == 2 && is_leap(y) {
        29
    } else {
        MDAYS[m as usize - 1]
    }
}

pub fn season_start(year: i32) -> GameDate {
    GameDate::new(year, 6, 1)
}

pub fn season_end(year: i32) -> GameDate {
    // Game: most westside camps are tearing down; legal season runs to Oct 31.
    GameDate::new(year, 9, 18)
}

pub fn doy(d: GameDate) -> i32 {
    d.doy()
}

pub fn tides_in_season(year: i32) -> i32 {
    season_start(year).days_until(season_end(year)) * 2
}

pub fn advance(day: GameDate, tide: Tide) -> (GameDate, Tide) {
    match tide {
        Tide::Flood => (day, Tide::Ebb),
        Tide::Ebb => (day.add_days(1), Tide::Flood),
    }
}

pub fn phase_label(d: GameDate, odd_year: bool, district: &str) -> String {
    let n = doy(d);
    if n < 167 {
        "mixed-stock early sockeye · 33-hr tests".into()
    } else if n < 187 {
        if district == "central" {
            "Karluk early reds passing the beaches".into()
        } else {
            "Frazer / early Upper Station reds".into()
        }
    } else if n < 228 {
        if odd_year {
            "pink push (odd-year flood)".into()
        } else {
            "pink push (even-year thin)".into()
        }
    } else if n < 237 {
        if district == "central" {
            "pinks + Karluk late reds".into()
        } else {
            "pinks + Upper Station".into()
        }
    } else if n < 249 {
        if district == "central" {
            "Karluk late sockeye · camps pulling".into()
        } else {
            "Upper Station late · camps pulling".into()
        }
    } else {
        "late sockeye + coho · skeleton crew".into()
    }
}
