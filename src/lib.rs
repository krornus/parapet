
pub extern crate image;

use image::{Pixel, DynamicImage, GenericImageView};

use xcb::Connection;
use xcb::xproto;
use xcb::randr as xrandr;

use xcb_util::image as xcb_image;

mod error;

use crate::error::Error;

trait AsU32 {
    fn as_u32(&self) -> u32;
}

impl<T: Pixel<Subpixel=u8>> AsU32 for T {
    fn as_u32(&self) -> u32 {

        let rgb = self.to_rgb();
        let c = rgb.channels();

        ((c[0] as u32) << 16) |
        ((c[1] as u32) <<  8) |
        ((c[2] as u32) <<  0)
    }
}

pub enum ImageMode {
    Fill,
    Max,
}

impl ImageMode {
    fn apply(&self, image: &image::DynamicImage, width: u32, height: u32) -> image::DynamicImage {
        match self {
            ImageMode::Fill => {
                image.resize(width, height, image::FilterType::Lanczos3)
            }
            ImageMode::Max => {
                image.resize_to_fill(width, height, image::FilterType::Lanczos3)
            }
        }
    }
}


pub struct Display<'a> {
    connection: &'a Connection,
    inner: xproto::Screen<'a>,
    info: xrandr::GetCrtcInfoReply,
}

impl<'a> Display<'a> {
    fn new(conn: &'a Connection, screen: xproto::Screen<'a>, info: xrandr::GetCrtcInfoReply) -> Self {
        Display {
            connection: conn,
            inner: screen,
            info: info,
        }
    }

    pub fn set(&self, buf: &DynamicImage, mode: ImageMode) -> Result<(), Error> {

        /* FIXME: cant overwrite pixmap for display which is above this one */


        let root = self.inner.root();
        let depth = self.inner.root_depth();

        let (width, height) = self.size();
        let (base_x, base_y) = self.origin();

        let buf = mode.apply(buf, width.into(), height.into());

        let mut image = xcb_util::image::shm::create(self.connection, depth, width, height)
            .map_err(|_| crate::error::Error::ImageCreate)?;

        println!("{},{} - {},{}", base_x, base_y, width, height);

        for (x,y,pixel) in buf.pixels() {
            image.put(x, y, pixel.as_u32())
        }

        let drawable = self.drawable()?;

        let gc = self.connection.generate_id();
        xcb::create_gc_checked(self.connection, gc, root, &[]).request_check()?;

        xcb_image::shm::put(self.connection, drawable, gc, &image, 0, 0, base_x, base_y, width, height, false)
            .map_err(|_| crate::error::Error::ImagePut)?;

        xcb::change_window_attributes_checked(self.connection, root, &[
                (xcb::CW_BACK_PIXMAP, drawable)
            ]).request_check()?;

        self.set_pixmap_property("_XROOTPMAP_ID", drawable)?;
        self.set_pixmap_property("ESETROOT_PMAP_ID", drawable)?;

        xcb::clear_area_checked(self.connection, true, root, base_x, base_y, width, height).request_check()?;

        if !self.connection.flush() {
            Err(Error::Flush)
        } else {
            Ok(())
        }
    }

    fn drawable(&self) -> Result<u32, Error> {

        let root = self.inner.root();
        let depth = self.inner.root_depth();
        let width = self.inner.width_in_pixels();
        let height = self.inner.height_in_pixels();
        let drawable = self.connection.generate_id();

        xcb::create_pixmap_checked(self.connection, depth, drawable, root, width, height)
            .request_check()?;

        Ok(drawable)
    }

    fn set_pixmap_property(&self, prop: &str, drawable: u32) -> Result<(), Error> {

        let atom = xcb::intern_atom(self.connection, false, prop).get_reply()?.atom();

        xcb::change_property(
            self.connection, xcb::PROP_MODE_REPLACE as u8,
            self.inner.root(), atom, xcb::ATOM_PIXMAP, 32, &[drawable]);

        Ok(())
    }

    fn size(&self) -> (u16, u16) {
        (self.info.width(), self.info.height())
    }

    fn origin(&self) -> (i16, i16) {
        (self.info.x(), self.info.y())
    }

    fn has_size(&self) -> bool {
        let (width, height) = self.size();

        width > 0 && height > 0
    }
}

pub struct Manager {
    connection: Connection,
    screen: usize,
    resources: xrandr::GetScreenResourcesReply,
}

impl Manager {
    pub fn new() -> Result<Manager, Error> {
        let (conn, scr) = Connection::connect(None)?;
        let screen = conn.get_setup().roots().nth(scr as usize).unwrap();
        let root = screen.root();
        let res = xrandr::get_screen_resources(&conn, root)
            .get_reply()?;

        Ok(Manager {
            connection: conn,
            screen: scr as usize,
            resources: res,
        })
    }

    pub fn displays<'a>(&'a self) -> Result<impl Iterator<Item=Display>, Error> {

        let timestamp = self.resources.timestamp();

        Ok(self.resources.crtcs().iter()
            .filter_map(move |crtc| {
                xrandr::get_crtc_info(&self.connection, *crtc, timestamp)
                    .get_reply().ok()
            })
            .map(move |info| {
                let screen = self.connection.get_setup().roots().nth(self.screen).unwrap();
                Display::new(&self.connection, screen, info)
            })
            .filter(|disp| disp.has_size()))
    }
}
