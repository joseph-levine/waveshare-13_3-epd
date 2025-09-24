pub struct RGB<T> {
    pub red: T,
    pub green: T,
    pub blue: T,
}

impl<T> RGB<T> where T: Into<u32> + Copy {}