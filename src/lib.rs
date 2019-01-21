
use image::{GenericImage, Pixel};

use xcb::Connection;
use xcb::xproto;
use xcb::xproto::Format;

use std::mem;

mod error;

use crate::error::Error;

trait XcbImage {
    fn depth(&self) -> u8;
    fn bpp(&self) -> u8;
}

impl<P: Pixel, I: GenericImage<Pixel=P>> XcbImage for I {
    fn depth(&self) -> u8 {
        P::channel_count() as u8 * mem::size_of::<P>() as u8
    }

    fn bpp(&self) -> u8 {
        self.depth()
    }
}

pub struct Manager {
    connection: Connection,
}

pub struct Screen<'a> {
    connection: &'a Connection,
    inner: xproto::Screen<'a>,
    window: xproto::Window,
    gc: u32,
    depth: u8,
}

impl<'a> Screen<'a> {
    fn new(conn: &'a Connection, screen: xproto::Screen<'a>) -> Self {

        let root = screen.root();
        let gc = conn.generate_id();
        let depth = screen.root_depth();

        xcb::create_gc(conn, gc, screen.root(), &[
              (xcb::GC_FUNCTION, xcb::GX_XOR),
              (xcb::GC_FOREGROUND, screen.white_pixel()),
              (xcb::GC_BACKGROUND, screen.black_pixel()),
              (xcb::GC_LINE_WIDTH, 1),
              (xcb::GC_LINE_STYLE, xcb::LINE_STYLE_ON_OFF_DASH),
              (xcb::GC_GRAPHICS_EXPOSURES, 0)
          ]);


        Screen {
            connection: conn,
            inner: screen,
            window: root,
            gc: gc,
            depth: depth,
        }
    }

    pub fn set<P: Pixel, I: GenericImage<Pixel=P>>(&self, image: I) -> Result<(), Error> {
        let format = self.format_for(image)?;
    }

    /* get format for image type */
    pub fn format_for<P: Pixel, I: GenericImage<Pixel=P>>(&self, image: &I) -> Result<Format, Error> {
        self.connection.get_setup().pixmap_formats()
            .find(|f| f.depth() == image.depth() && f.bits_per_pixel() == image.bpp())
            .ok_or(Error::ImageFormat)
    }
}

impl Manager {
    pub fn new() -> Result<Manager, Error> {
        let (conn, _) = Connection::connect(None)?;

        Ok(Manager {
            connection: conn,
        })
    }

    pub fn screens<'a>(&'a self) -> impl Iterator<Item=Screen> {
        self.connection.get_setup().roots()
            .map(move |x| Screen::new(&self.connection, x))
    }
}
