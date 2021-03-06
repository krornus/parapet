pub extern crate image;

use image::{DynamicImage};

use xcb::Connection;
use xcb::xproto;
use xcb::randr as xrandr;

mod error;
mod ximage;
mod rect;

pub use crate::error::Error;
pub use crate::ximage::ImageMode;


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

        let root = self.inner.root();

        let (width, height) = self.size();
        let (base_x, base_y) = self.origin();

        let image = mode.apply(buf, width.into(), height.into());
        let drawable = self.drawable(width, height)?;

        let gc = self.connection.generate_id();
        xcb::create_gc_checked(self.connection, gc, root, &[]).request_check()?;

        image.put_checked(self.connection, drawable, gc, base_x, base_y).request_check()?;

        xcb::change_window_attributes_checked(self.connection, root, &[
                (xcb::CW_BACK_PIXMAP, drawable)
            ]).request_check()?;

        self.set_pixmap("_XROOTPMAP_ID", drawable)?;
        self.set_pixmap("ESETROOT_PMAP_ID", drawable)?;

        if !self.connection.flush() {
            Err(Error::Flush)
        } else {

            xcb::clear_area_checked(self.connection, true, root,
                0, 0, self.inner.width_in_pixels(), self.inner.height_in_pixels()).request_check()?;

            Ok(())
        }
    }

    fn drawable(&self, width: u16, height: u16) -> Result<u32, Error> {

        let root = self.inner.root();
        let depth = self.inner.root_depth();
        let drawable = self.connection.generate_id();

        xcb::create_pixmap_checked(self.connection, depth, drawable, root, width, height)
            .request_check()?;

        Ok(drawable)
    }

    fn get_pixmap(&self, prop: &str) -> Result<xcb::Pixmap, Error> {

        let atom = xcb::intern_atom(self.connection, false, prop).get_reply()?.atom();

        let reply = xcb::get_property(
            self.connection, false, self.inner.root(),
            atom, xcb::ATOM_PIXMAP, 0, 1).get_reply();

        match reply {
            Ok(x) => if x.type_() != xcb::ATOM_PIXMAP {
                Err(Error::InvalidType(format!("Pixmap {} is not of type pixmap", prop)))
            } else {
                Ok(x.value()[0])
            }
            Err(e) => Err(e.into()),
        }
    }

    fn set_pixmap(&self, prop: &str, drawable: u32) -> Result<(), Error> {

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
            .map(move |crtc| {
                xrandr::get_crtc_info(&self.connection, *crtc, timestamp)
            })
            .filter_map(|x| {
                x.get_reply().ok()
            })
            .map(move |info| {
                let screen = self.connection.get_setup().roots().nth(self.screen).unwrap();
                Display::new(&self.connection, screen, info)
            })
            .filter(|disp| disp.has_size()))
    }
}
