#[derive(Clone, Copy)]
struct RegisterB(u8);

impl RegisterB {
    const USE_BINARY_FORMAT: Self = Self(1 << 2);

    fn empty() -> Self {
        Self(0)
    }

    fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }
}

fn parse_value(
    x: u8,
    format: RegisterB,
) -> u8 {
    if format.contains(RegisterB::USE_BINARY_FORMAT) {
        return x
    } else {
        return (x / 16) * 10 + (x & 0xf) 
    }
}

fn main() {
    let bcd_format = RegisterB::empty();
    let binary_format = RegisterB::USE_BINARY_FORMAT;

    // Проверяем все корректные двухзначные BCD-значения.
    for tens in 0u8..=9 {
        for units in 0u8..=9 {
            let bcd = (tens << 4) | units;
            let expected = tens * 10 + units;

            assert_eq!(
                parse_value(bcd, bcd_format),
                expected,
                "Ошибка для BCD 0x{:02X}",
                bcd,
            );
        }
    }

    // В двоичном формате входное значение должно сохраняться.
    for x in 0u8..=255 {
        assert_eq!(
            parse_value(x, binary_format),
            x,
            "Ошибка для двоичного значения {}",
            x,
        );
    }

    println!("Все проверки прошли!");
}
