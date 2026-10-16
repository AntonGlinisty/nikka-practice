// Минимальная замена bitflags для запуска без Cargo.
#[derive(Clone, Copy)]
struct RegisterB(u8);

impl RegisterB {
    const USE_24_HOUR_FORMAT: Self = Self(1 << 1);
    const USE_BINARY_FORMAT: Self = Self(1 << 2);

    fn empty() -> Self {
        Self(0)
    }

    fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }
}

impl std::ops::BitOr for RegisterB {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

fn parse_hour(hour: u8, format: RegisterB) -> u8 {
    let is_24_hour_format = format.contains(RegisterB::USE_24_HOUR_FORMAT);

    const MERIDIEM_FLAG: u8 = 1 << 7;
    let is_pm = (hour & MERIDIEM_FLAG) != 0;

    let hour = hour & !MERIDIEM_FLAG;
    let res: u8;

    if format.contains(RegisterB::USE_BINARY_FORMAT) {
        res = hour
    } else {
        res = (hour / 16) * 10 + (hour & 0xf) 
    }

    if !is_24_hour_format {
        if is_pm && res != 12 {
            return res + 12
        }
        if !is_pm && res == 12 {
            return 0
        }
    }
    return res
}

fn main() {
    // Проверяем все 24 часа во всех четырёх форматах RTC.
    for binary in [false, true] {
        for use_24_hour in [false, true] {
            let mut format = RegisterB::empty();
            if binary {
                format = format | RegisterB::USE_BINARY_FORMAT;
            }
            if use_24_hour {
                format = format | RegisterB::USE_24_HOUR_FORMAT;
            }

            for expected in 0u8..=23 {
                // В 12-часовом формате полночь — 12 AM, полдень — 12 PM.
                let displayed_hour = if use_24_hour {
                    expected
                } else if expected % 12 == 0 {
                    12
                } else {
                    expected % 12
                };

                let mut encoded_hour = if binary {
                    displayed_hour
                } else {
                    ((displayed_hour / 10) << 4) | (displayed_hour % 10)
                };

                // В 12-часовом формате старший бит обозначает PM.
                if !use_24_hour && expected >= 12 {
                    encoded_hour |= 0x80;
                }

                assert_eq!(
                    parse_hour(encoded_hour, format),
                    expected,
                    "hour=0x{:02X}, binary={}, use_24_hour={}",
                    encoded_hour,
                    binary,
                    use_24_hour,
                );
            }
        }
    }

    println!("Все 96 проверок прошли!");
}
