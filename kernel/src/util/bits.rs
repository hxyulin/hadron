use core::ops::Range;

#[const_trait]
pub trait BitHelper {
    fn set_bit(&mut self, bit: usize, value: bool);
    fn set_bits(&mut self, bits: Range<usize>, value: Self);
    fn get_bit(&self, bit: usize) -> bool;
    fn get_bits(&self, bits: Range<usize>) -> Self;
}

macro_rules! impl_bit_helper {
    ($ut:ty, $it:ty, $bc:literal) => {
        impl BitHelper for $ut {
            fn set_bit(&mut self, bit: usize, value: bool) {
                assert!(bit < $bc, concat!("bit out of range for u", stringify!($bc)));
                *self &= !(1 << bit);
                *self |= (value as $ut) << bit;
            }

            fn set_bits(&mut self, bits: Range<usize>, value: Self) {
                assert!(
                    bits.start < $bc && bits.end <= $bc,
                    concat!("bits out of range for u", stringify!($bc))
                );
                assert!(bits.end != bits.start, "bits range must be at least 1 bit");
                let mask = ((<$it>::MIN >> (bits.end - bits.start - 1)) as $ut) >> ($bc - bits.end);
                *self &= !mask;
                *self |= (value << bits.start) & mask;
            }

            fn get_bit(&self, bit: usize) -> bool {
                assert!(bit < $bc, concat!("bit out of range for u", stringify!($bc)));
                ((self >> bit) & 1) == 1
            }

            fn get_bits(&self, bits: Range<usize>) -> Self {
                assert!(
                    bits.start < $bc && bits.end <= $bc,
                    concat!("bits out of range for u", stringify!($bc))
                );
                assert!(bits.end != bits.start, "bits range must be at least 1 bit");
                let truncated = (self << ($bc - bits.end)) >> ($bc - bits.end);
                truncated >> bits.start
            }
        }
    };
}

impl_bit_helper!(u8, i8, 8);
impl_bit_helper!(u16, i16, 16);
impl_bit_helper!(u32, i32, 32);
impl_bit_helper!(u64, i64, 64);
impl_bit_helper!(u128, i128, 64);

#[cfg(all(test, feature = "test"))]
mod tests {
    use crate::util::bits::BitHelper;

    #[test]
    fn bithelper_u8_set_bit() {
        let mut val = 0u8;
        val.set_bit(2, true);
        assert_eq!(val, 0b0000_0100);
        val.set_bit(2, false);
        assert_eq!(val, 0b0000_0000);

        val.set_bit(0, true);
        val.set_bit(7, true);
        assert_eq!(val, 0b1000_0001);
    }

    #[test]
    fn bithelper_u8_set_bits() {
        let mut val = 0u8;
        val.set_bits(2..4, 0b11); // value 3
        assert_eq!(val, 0b0000_1100);
        val.set_bits(2..4, 0b00); // value 0
        assert_eq!(val, 0b0000_0000);

        val.set_bits(0..3, 0b101); // value 5
        assert_eq!(val, 0b0000_0101);

        // Test full range
        let mut val_full = 0u8;
        val_full.set_bits(0..8, 0xFF);
        assert_eq!(val_full, 0xFF);
        val_full.set_bits(0..8, 0x00);
        assert_eq!(val_full, 0x00);
    }

    #[test]
    fn bithelper_u8_get_bit() {
        let val = 0b1001_0110u8;
        assert_eq!(val.get_bit(0), false);
        assert_eq!(val.get_bit(1), true);
        assert_eq!(val.get_bit(7), true);
    }

    #[test]
    fn bithelper_u8_get_bits() {
        let val = 0b1111_0011u8;
        assert_eq!(val.get_bits(0..2), 0b11);
        assert_eq!(val.get_bits(2..4), 0b00);
        assert_eq!(val.get_bits(4..8), 0b1111);
    }

    // --- U16 Tests ---
    #[test]
    fn bithelper_u16_set_bit() {
        let mut val = 0u16;
        val.set_bit(7, true);
        assert_eq!(val, 0b1000_0000); // 128
        val.set_bit(15, true);
        assert_eq!(val, 0b1000_0000_1000_0000); // 32896
        val.set_bit(7, false);
        assert_eq!(val, 0b1000_0000_0000_0000); // 32768
    }

    #[test]
    fn bithelper_u16_set_bits() {
        let mut val = 0u16;
        val.set_bits(0..4, 0b1111); // Low nibble
        assert_eq!(val, 0b0000_0000_0000_1111);

        val.set_bits(8..12, 0b0101); // Middle nibble
        assert_eq!(val, 0b0000_0101_0000_1111);

        val.set_bits(12..16, 0b1010); // High nibble
        assert_eq!(val, 0b1010_0101_0000_1111);

        val.set_bits(0..16, 0x55AA); // Full range
        assert_eq!(val, 0x55AA);
        val.set_bits(0..16, 0x0000);
        assert_eq!(val, 0x0000);
    }

    #[test]
    fn bithelper_u16_get_bit() {
        let val = 0b1010_1010_0101_0101u16;
        assert_eq!(val.get_bit(0), true);
        assert_eq!(val.get_bit(1), false);
        assert_eq!(val.get_bit(8), false);
        assert_eq!(val.get_bit(15), true);
    }

    #[test]
    fn bithelper_u16_get_bits() {
        let val = 0b1010_1010_0101_0101u16;
        assert_eq!(val.get_bits(0..4), 0b0101); // LSB nibble
        assert_eq!(val.get_bits(8..12), 0b1010); // Middle nibble
        assert_eq!(val.get_bits(12..16), 0b1010); // MSB nibble
        assert_eq!(val.get_bits(0..16), val); // Full range
    }

    // --- U32 Tests ---
    #[test]
    fn bithelper_u32_set_bit() {
        let mut val = 0u32;
        val.set_bit(10, true);
        assert_eq!(val, 1 << 10);
        val.set_bit(31, true);
        assert_eq!(val, (1 << 10) | (1 << 31));
        val.set_bit(10, false);
        assert_eq!(val, 1 << 31);
    }

    #[test]
    fn bithelper_u32_set_bits() {
        let mut val = 0u32;
        val.set_bits(0..8, 0xAB); // Low byte
        assert_eq!(val, 0xAB);

        val.set_bits(16..24, 0xCD); // Middle byte
        assert_eq!(val, 0xCD00AB);

        val.set_bits(24..32, 0xEF); // High byte
        assert_eq!(val, 0xEFCD00AB);

        val.set_bits(0..32, 0xDEADBEEF); // Full range
        assert_eq!(val, 0xDEADBEEF);
        val.set_bits(0..32, 0x00000000);
        assert_eq!(val, 0x00000000);
    }

    #[test]
    fn bithelper_u32_get_bit() {
        let val = 0b10101010_01010101_11001100_00110011u32;
        assert_eq!(val.get_bit(0), true);
        assert_eq!(val.get_bit(1), true);
        assert_eq!(val.get_bit(15), true);
        assert_eq!(val.get_bit(31), true);
    }

    #[test]
    fn bithelper_u32_get_bits() {
        let val = 0xDEADBEEFu32;
        assert_eq!(val.get_bits(0..8), 0xEF); // Low byte
        assert_eq!(val.get_bits(8..16), 0xBE); // Next byte
        assert_eq!(val.get_bits(16..24), 0xAD); // Next byte
        assert_eq!(val.get_bits(24..32), 0xDE); // High byte
        assert_eq!(val.get_bits(0..32), val); // Full range
    }

    // --- U64 Tests ---
    #[test]
    fn bithelper_u64_set_bit() {
        let mut val = 0u64;
        val.set_bit(30, true);
        assert_eq!(val, 1 << 30);
        val.set_bit(63, true);
        assert_eq!(val, (1 << 30) | (1 << 63));
        val.set_bit(30, false);
        assert_eq!(val, 1 << 63);
    }

    #[test]
    fn bithelper_u64_set_bits() {
        let mut val = 0u64;
        val.set_bits(0..16, 0xAABB); // Low word
        assert_eq!(val, 0xAABB);

        val.set_bits(32..48, 0xCCDD); // Middle word
        assert_eq!(val, 0x0000_CCDD_0000_AABB);

        val.set_bits(48..64, 0xEEFF); // High word
        assert_eq!(val, 0xEEFF_CCDD_0000_AABB);

        val.set_bits(0..64, 0x123456789ABCDEF0); // Full range
        assert_eq!(val, 0x123456789ABCDEF0);
        val.set_bits(0..64, 0x0000000000000000);
        assert_eq!(val, 0x0000000000000000);
    }

    #[test]
    fn bithelper_u64_get_bit() {
        let val = 0x1122334455667788u64;
        assert_eq!(val.get_bit(0), false); // 8 is 1000
        assert_eq!(val.get_bit(3), true); // 8 is 1000
        assert_eq!(val.get_bit(32), false); // 4 is 0100
        assert_eq!(val.get_bit(63), false); // 1 is 0001
    }

    #[test]
    fn bithelper_u64_get_bits() {
        let val = 0x123456789ABCDEF0u64;
        assert_eq!(val.get_bits(0..8), 0xF0); // 0th byte
        assert_eq!(val.get_bits(8..16), 0xDE); // 1st byte
        assert_eq!(val.get_bits(32..40), 0x78); // 4th byte
        assert_eq!(val.get_bits(56..64), 0x12); // 7th byte
        assert_eq!(val.get_bits(0..64), val); // Full range
    }
}
