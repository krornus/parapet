use image::{Pixel, DynamicImage, GenericImageView};

const BLOCKSIZE: usize = 128;

pub trait SplitPixels<'a, I: ?Sized + 'a> {
    fn split_pixels(&self) -> SplitPixelsIterator<I>;
}

impl<'a, I: GenericImageView + 'a> SplitPixels<'a, I> for I {
    fn split_pixels(&self) -> SplitPixelsIterator<I> {
        let (width, height) = self.dimensions();
        SplitPixelsIterator {
            image: self,
            length: width as usize * height as usize,
            x: 0,
            y: 0,
            width,
            height,
        }
    }
}

/* custom pixel iterator which can be split arbitrarily */
pub struct SplitPixelsIterator<'a, I: ?Sized + 'a> {
    image: &'a I,
    length: usize,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl<'a, I: ?Sized + 'a> SplitPixelsIterator<'a, I> {
    pub fn split(mut self) -> (SplitPixelsIterator<'a, I>, Option<SplitPixelsIterator<'a, I>>) {
        if self.length <= BLOCKSIZE {
            (self, None)
        } else {
            let bound = (self.length / 2) as u32;

            let length = self.length - bound as usize;
            let x = self.x + bound % self.width;
            let y = self.y + bound / self.width;
            let width = self.width;
            let height = self.height;

            let split = SplitPixelsIterator {
                image: self.image,
                length,
                x,
                y,
                width,
                height,
            };

            self.length = bound as usize;

            (self, Some(split))
        }
    }
}

impl<'a, I: GenericImageView> Iterator for SplitPixelsIterator<'a, I> {
    type Item = (u32, u32, I::Pixel);

    fn next(&mut self) -> Option<(u32, u32, I::Pixel)> {
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }

        if self.y >= self.height {
            None
        } else {
            let pixel = self.image.get_pixel(self.x, self.y);
            let p = (self.x, self.y, pixel);

            self.x += 1;
            self.length -= 1;

            Some(p)
        }
    }
}
