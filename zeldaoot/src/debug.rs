/// number of REG groups, i.e. REG, SREG, OREG, etc.
const REG_GROUPS: usize = 29;
const REG_PAGES: usize = 6;
const REG_PER_PAGE: usize = 16;
const REG_PER_GROUP: usize = REG_PAGES * REG_PER_PAGE;

/// Game Info aka. Static Context (dbg ram start: 80210A10)
/// Data normally accessed through REG macros (see regs.h)
pub struct GameInfo {
    /// 1 is first page
    reg_page: i32,
    /// "register" group (R, RS, RO, RP etc.)
    reg_group: i32,
    /// selected register within page
    reg_cur: i32,
    dpad_last: i32,
    repeat: i32,
    data: [i16; REG_GROUPS * REG_PER_GROUP],

    pub trnsn_unk_state: i32,
}

impl Default for GameInfo {
    fn default() -> Self {
        Self {
            reg_page: 0,
            reg_group: 0,
            reg_cur: 0,
            dpad_last: 0,
            repeat: 0,
            data: [0; REG_GROUPS * REG_PER_GROUP],

            trnsn_unk_state: 0,
        }
    }
}

impl GameInfo {
    pub fn base_reg(&self, n: usize, r: usize) -> i16 {
        self.data[n * REG_PER_GROUP + r]
    }

    pub fn set_base_reg(&mut self, n: usize, r: usize, value: i16) {
        self.data[n * REG_PER_GROUP + r] = value;
    }

    pub fn reg(&self, r: usize) -> i16 {
        self.base_reg(0, r)
    }

    pub fn sreg(&self, r: usize) -> i16 {
        self.base_reg(1, r)
    }

    pub fn hreg(&self, r: usize) -> i16 {
        self.base_reg(21, r)
    }

    pub fn set_hreg(&mut self, r: usize, value: i16) {
        self.set_base_reg(21, r, value)
    }

    pub fn pause_menu_mode(&mut self) -> i16 {
        self.sreg(94)
    }
}
