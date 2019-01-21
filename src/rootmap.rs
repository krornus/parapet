use crate::Screen;

/*
 * https://en.wikipedia.org/wiki/Pseudo-transparency
 * These properties are used to inform the window where it
 * can find the pixmap used on the root window. Using this
 * information, a client can paint a section of the image
 * (corresponding to the size and position of the window)
 * onto its background, achieving the effect of transparency.
 * This method uses the most memory, but has the advantage of
 * allowing clients to perform any operation, such as tinting
 * or shading, on the image data.
 */
fn xroot<'a>(screen: &Screen<'a>) {
    xcb::get_property(
        screen.connection, false, screen.inner.root(),
        xcb::intern_atom(self.connection, false, "_XROOTPMAP_ID").get_reply()?.atom(),
        xcb::ATOM_PIXMAP, 0, 1)
}

fn esetroot<'a>(screen: &Screen<'a>) {
    xcb::get_property(
        screen.connection, false, screen.inner.root(),
        xcb::intern_atom(self.connection, false, "ESETROOT_PMAP_ID").get_reply()?.atom(),
        xcb::ATOM_PIXMAP, 0, 1)
}

struct RootWindowProperties {
    root_pixmap: Option<xlib::Atom>,
    esetroot_pixmap: Option<xlib::Atom>,
}

impl RootWindowProperties {
    fn new(xsess: &xorg::XorgSession) -> Self {
        RootWindowProperties {
            root_pixmap: xsess.atom("_XROOTPMAP_ID", true),
            esetroot_pixmap: xsess.atom("ESETROOT_PMAP_ID", true),
        }
    }


    /*
     * http://search.cpan.org/dist/X11-Protocol-Other/lib/X11/Protocol/XSetRoot.pm#ROOT_WINDOW_PROPERTIES
     * if the background is replaced, kill esetroot_pmap_id if it is the same as the root pixmap
     */
    fn cleanse_esetroot(&self, conn: &Connection) {
        let prop_root_pmap = match self.root_pixmap {
            Some(pm) => pm,
            None => {
                return;
            }
        };

        let prop_esetroot_pmap = match self.esetroot_pixmap {
            Some(pm) => pm,
            None => {
                return;
            }
        };

        let root_pmap_id =
            xsess
                .root
                .property(prop_root_pmap, 0, 1, false, xlib::AnyPropertyType as u64);

        if root_pmap_id.property_type == xlib::XA_PIXMAP {
            let esetroot_pmap_id = xsess.root.property(
                prop_esetroot_pmap,
                0,
                1,
                false,
                xlib::AnyPropertyType as u64,
            );

            if !root_pmap_id.property.is_null()
                && !esetroot_pmap_id.property.is_null()
                && esetroot_pmap_id.property_type == xlib::XA_PIXMAP
            {
                /* safe, these are type XA_PIXMAP */
                let pm1 = unsafe { *root_pmap_id.property as xlib::Pixmap };
                let pm2 = unsafe { *esetroot_pmap_id.property as xlib::Pixmap };

                /* kill if equal */
                if pm1 == pm2 {
                    xsess.kill_client(pm1);
                }
            }

            if !esetroot_pmap_id.property.is_null() {
                xorg::XFree(root_pmap_id.property);
            }
        }

        if !root_pmap_id.property.is_null() {
            xorg::XFree(root_pmap_id.property);
        }
    }

    /* change the properties that store the current background as pixmap */
    pub fn update_background(&self, conn: &Connection, drawable: u32) {

        self.cleanse_esetroot(conn);

        if let Some(root_pixmap) = self.root_pixmap {
            xsess.root.change_property(
                root_pixmap,
                xlib::XA_PIXMAP,
                32,
                xlib::PropModeReplace,
                &drawable,
                1,
            );
        }

        if let Some(esetroot_pixmap) = self.esetroot_pixmap {
            xsess.root.change_property(
                esetroot_pixmap,
                xlib::XA_PIXMAP,
                32,
                xlib::PropModeReplace,
                &drawable,
                1,
            );
        }
    }
}
