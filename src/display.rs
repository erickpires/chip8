pub(crate) struct Display {
    // TODO: Replacing this array with a Vec would
    // enable us to easily implement multiple screen sizes.
    data: [u8; (64 / 8) * 32],
}

impl Display {
    pub(crate) const fn new() -> Self {
        Self {
            data: [0; (64 / 8) * 32]
        }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}