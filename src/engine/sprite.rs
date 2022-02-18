use std::io::Cursor;

use image::io::Reader;

pub struct Sprite {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Sprite {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let cursor = Cursor::new(bytes);
        let reader = Reader::new(cursor)
            .with_guessed_format()
            .expect("Cursor io never fails");
        let image = reader.decode().unwrap(); // TODO: remove unwraps.
        let image = image.to_rgba8();

        let (width, height) = image.dimensions();
        let data = image.to_vec();

        Self {
            width,
            height,
            data,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}
