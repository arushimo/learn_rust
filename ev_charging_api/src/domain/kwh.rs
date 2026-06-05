#[derive(Debug, Clone, Copy)]
pub struct Kwh(i32);

impl Kwh {
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl TryFrom<i32> for Kwh {
    type Error = &'static str;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > 0 && value <= 150 {
            Ok(Kwh(value))
        } else {
            Err("無効な充電量です。1〜150kWhの間で指定してください。")
        }
    }
}
