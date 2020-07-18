pub trait Color {
    fn colored(&self) -> Self;
}

impl Color for String {
    fn colored(&self) -> Self {
        self.replace("&", "§")
    }
}