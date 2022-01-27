#[derive(Default)]
pub struct ShrinkWindow {
    d_8012ced0: i32,
    val: i32,
    current_val: i32,
}

impl ShrinkWindow {
    pub fn get_current_val(&self) -> i32 {
        self.current_val
    }
}
