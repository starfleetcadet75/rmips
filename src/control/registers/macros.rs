macro_rules! register_r {
    ($getter: ident, $bit: expr) => {
        #[inline]
        pub fn $getter(&self) -> bool {
            self.bits.get_bit($bit)
        }
    };
}

macro_rules! register_w {
    ($setter: ident, $resetter: ident, $bit: expr) => {
        #[inline]
        pub fn $setter(&mut self) {
            self.bits.set_bit($bit, true);
        }

        #[inline]
        pub fn $resetter(&mut self) {
            self.bits.set_bit($bit, false);
        }
    };
}

macro_rules! register_rw {
    ($getter: ident, $setter: ident, $resetter: ident, $bit: expr) => {
        register_r!($getter, $bit);
        register_w!($setter, $resetter, $bit);
    };
}

macro_rules! register_field {
    ($getter: ident, $setter: ident, $start: expr, $end: expr) => {
        pub fn $getter(&self) -> u32 {
            self.bits.get_bits($start..=$end)
        }

        pub fn $setter(&mut self, val: u32) {
            self.bits.set_bits($start..=$end, val);
        }
    };
}
