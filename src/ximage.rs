use image::{Bgra, DynamicImage, GenericImage, GenericImageView, ImageBuffer};
use xcb::IMAGE_FORMAT_Z_PIXMAP;

use crate::rect::{Position, Rect};

type BgraImage = ImageBuffer<Bgra<u8>, Vec<u8>>;

fn ceil(a: u32, b: u32) -> u32 {
    if a == 0 {
        0
    } else {
        1 + ((a - 1) / b)
    }
}

/* place rectangle of pixels from top onto bottom starting at x, y */
fn replace_area<I: GenericImage>(bottom: &mut I, top: &I, x_pos: u32, y_pos: u32, mut dim: Rect) {

    /* dim must be within size of top */
    dim = Rect::new(0, 0, top.width(), top.height()).clamp(dim);

    /* get the relative area of the canvas which will be written to */
    let rel_width = bottom.width().saturating_sub(x_pos);
    let rel_height = bottom.height().saturating_sub(y_pos);

    /* clamp within the screen's area, relative to image */
    let area = Rect::new(dim.x, dim.y, rel_width, rel_height);

    /* dim must also be within this area */
    dim = area.clamp(dim);

    for y in 0..dim.height {
        for x in 0..dim.width {
            let p = top.get_pixel(x + dim.x, y + dim.y);
            bottom.put_pixel(x + x_pos, y + y_pos, p);
        }
    }
}

fn to_bgra(image: DynamicImage) -> BgraImage {
    match image {
        DynamicImage::ImageBgra8(i) => i,
        _ => panic!("invalid attempt to unwrap dynamic image")
    }
}

pub enum ImageMode {
    Max,
    Fill,
    Center,
    Tile,
}

impl ImageMode {
    pub fn apply(&self, image: &DynamicImage, width: u32, height: u32) -> Image {

        let image = DynamicImage::ImageBgra8(image.to_bgra());
        let screen = Rect::new(0, 0, width, height);

        let bgra = match self {
            ImageMode::Max => {
                Self::max(&image, screen)
            },
            ImageMode::Fill => {
                Self::fill(&image, screen)
            },
            ImageMode::Tile => {
                Self::tile(&image, screen)
            },
            ImageMode::Center => {
                Self::center(&image, screen)
            },
        };

        Image::create(bgra)
    }

    fn max(image: &DynamicImage, screen: Rect) -> BgraImage {
        let image = image.resize_to_fill(screen.width, screen.height, image::FilterType::Nearest);
        to_bgra(image)
    }

    fn fill(image: &DynamicImage, screen: Rect) -> BgraImage {
        /* resize image */
        let image = image.resize(screen.width, screen.height, image::FilterType::Nearest);
        /* get dimensions */
        let dim = Rect::new(0, 0, image.width(), image.height());
        /* make new canvas */
        let mut canvas = ImageBuffer::new(screen.width, screen.height);
        /* get center rectangle */
        let center = screen.relative(Position::Center, dim.width, dim.height);
        /* place image on canvas */
        image::imageops::replace(&mut canvas, image.as_bgra8().unwrap(), center.x, center.y);

        canvas
    }

    fn tile(image: &DynamicImage, screen: Rect) -> BgraImage {

        /* cast image */
        let bgra = image.as_bgra8().unwrap();
        /* get image dimensions */
        let dim = Rect::new(0, 0, image.width(), image.height());

        /* check if tiling is possible, center if not */
        if !screen.contains_width(&dim) && !screen.contains_height(&dim) {
            return Self::center(image, screen);
        }

        /* get center rectangle */
        let center = screen.relative(Position::Center, dim.width, dim.height);

        /* make new canvas */
        let mut canvas = ImageBuffer::new(screen.width, screen.height);
        /* make image column */
        let mut col = ImageBuffer::new(dim.width, screen.height);

        /* place image in center of column */
        image::imageops::replace(&mut col, bgra, 0, center.y);

        /* column rectangles */
        let mut above = screen.clamp(center.relative(Position::Top, dim.width, dim.height));
        let mut below = screen.clamp(center.relative(Position::Bottom, dim.width, dim.height));

        /* number of tiles above/below the center image */
        let hcol = ceil(screen.height - center.bottom(), dim.height);

        for _ in 0..hcol {

            let crop = Rect::new(0, dim.height - above.height, above.width, above.height);
            replace_area(&mut col, bgra, above.x, above.y, crop);

            let crop = Rect::new(0, 0, below.width, below.height);
            replace_area(&mut col, bgra, below.x, below.y, crop);

            above = screen.clamp(above.relative(Position::Top, dim.width, dim.height));
            below = screen.clamp(below.relative(Position::Bottom, dim.width, dim.height));
        }


        /* place column in center of canvas */
        image::imageops::replace(&mut canvas, &col, center.x, 0);

        let mut left = screen.clamp(center.relative(Position::Left, dim.width, dim.height));
        let mut right = screen.clamp(center.relative(Position::Right, dim.width, dim.height));

        /*number of tiles left/right of the center image */
        let hrow = ceil(screen.width - center.right(), dim.width);

        for _ in 0..hrow {

            let crop = Rect::new(dim.width - left.width, 0, left.width, left.height);
            replace_area(&mut canvas, &col, left.x, left.y, crop);

            let crop = Rect::new(0, 0, right.width, right.height);
            replace_area(&mut canvas, &col, right.x, right.y, crop);

            left = screen.clamp(left.relative(Position::Left, dim.width, dim.height));
            right = screen.clamp(right.relative(Position::Right, dim.width, dim.height));
        }

        canvas
    }

    fn center(image: &DynamicImage, screen: Rect) -> BgraImage {

        /* cast image */
        let bgra = image.as_bgra8().unwrap();
        /* get image dimensions */
        let mut dim = Rect::new(0, 0, image.width(), image.height());
        /* clamp image dimensions to screen */
        dim = screen.clamp(dim);
        /* make new canvas */
        let mut canvas = ImageBuffer::new(screen.width, screen.height);
        /* get center x, y */
        let center = screen.relative(Position::Center, dim.width, dim.height);

        let crop = Rect::new((image.width() - dim.width) / 2, (image.height() - dim.height) / 2, dim.width, dim.height);

        replace_area(&mut canvas, bgra, center.x, center.y, crop);

        canvas
    }
}

pub struct Image {
    width:  u32,
    height: u32,
    depth:  u32,
    pub data:   Vec<u8>,
}

impl Image {
    pub fn create(image: BgraImage) -> Self {

        let (width, height) = image.dimensions();
        let depth = 24;

        Image {
            width, height, depth,
            data: image.into_raw(),
        }
    }

    pub fn put_checked<'a>(&self, connection: &'a xcb::Connection, drawable: xcb::Drawable, gc: xcb::Gc, x: i16, y: i16)
        -> xcb::VoidCookie<'a>
    {
        xcb::put_image_checked(
            connection,
            IMAGE_FORMAT_Z_PIXMAP as u8,
            drawable,
            gc,
            self.width as u16, self.height as u16,
            x, y, 0,
            self.depth as u8,
            &self.data)
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width as u16, self.height as u16)
    }
}
