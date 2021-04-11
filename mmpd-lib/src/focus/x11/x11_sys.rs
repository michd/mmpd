use x11::xlib;
use std::ffi::c_void;
use std::os::raw::{c_long, c_ulong, c_int, c_uchar};

use crate::focus::FocusedWindow;

pub struct X11Sys { display: *mut xlib::Display }

impl X11Sys {
    pub fn new() -> X11Sys {
        X11Sys {
            display: unsafe { xlib::XOpenDisplay(::std::ptr::null()) }
        }
    }

    /// Gets FocusedWindow info for a given window_id
    ///
    /// ## Parameters:
    /// - window_id: an X window id (which is an unsigned int type)
    ///
    /// ## Returns:
    ///
    /// `Some(FocusedWindow)` if the data could be retrieved without error
    /// `None` if there was any issue retrieving data
    pub fn get_window_info(&self, window_id: xlib::Window) -> Option<FocusedWindow> {
        let window_class = self.get_window_prop_strings(window_id, xlib::XA_WM_CLASS)?;
        let window_name = self.get_window_prop_strings(window_id, xlib::XA_WM_NAME)?;

        Some(FocusedWindow {
            window_class,
            window_name: window_name.first()?.to_string()
        })
    }

    /// Gets a window property that is represented as one or more Strings
    ///
    /// ## Parameters:
    /// - window_id: an X Window id (which is an unsigned int type)
    /// - property name: an X Atom referring to a window property, see xlib's XA_WM_ constants.
    ///
    /// ## Returns:
    /// Some(Vec<String>) if successfully retrieving the data
    /// None if for any reason the data can't be retrieved or parsed.
    fn get_window_prop_strings(
        &self,
        window_id: xlib::Window,
        property_name: xlib::Atom
    ) -> Option<Vec<String>> {
        // In the data for the property we're reading, return data starting at this offset
        // The offset is in terms of "items", however large they may be. In this function
        // we're only concerned with bytes.
        const READ_OFFSET: c_long = 0;

        // Maximum number of items to (in our case, bytes) to read from this property
        const READ_LENGTH: c_long = 1000;

        // Expected data type as represented to X. Our return values can be several different
        // types that all represent some form of string. As such we can't just specify XA_STRING,
        // and must accept AnyPropertyType instead.
        let req_type: xlib::Atom = xlib::AnyPropertyType as xlib::Atom;

        // Where XGetWindowProperty will store the type of the property value found.
        // We don't do anything with this value
        let mut actual_type: xlib::Atom = 0;

        // Where XGetWindowProperty will store the "item" size. We expect this to be 8 (byte).
        // If it does not get set to 8, this function will return None.
        let mut actual_format: c_int = 0;

        // Where XGetWindowProperty will store the number of items available in the array pointed
        // to by prop_ptr_ptr.
        let mut nitems: c_ulong = 0;

        // Where XGetWindowProperty will store the number of bytes left beyond our read length
        // (always in bytes, even if actual_format isn't 8). We don't use this value and assume that
        // contents returned will always fit within READ_LENGTH.
        let mut bytes_after: c_ulong = 0;

        // Where XGetWindowProperty will store a pointer to an array it has allocated, containing
        // the data of the requested property. This must later be cast to the approriate type to
        // access the data.
        let mut prop_ptr_ptr: *mut c_uchar = std::ptr::null_mut();

        let result = unsafe {
            // See documentation at
            // https://tronche.com/gui/x/xlib/window-information/XGetWindowProperty.html
            xlib::XGetWindowProperty(
                self.display,
                window_id,
                property_name,
                READ_OFFSET,
                READ_LENGTH,

                // Do not delete the property from the window
                false as i32,
                req_type,

                // "Return" values XGetWindowProperty will write into
                &mut actual_type,
                &mut actual_format,
                &mut nitems,
                &mut bytes_after,
                &mut prop_ptr_ptr
            )
        };

        // If the call was unsuccessful, we won't try anything else and return no data.
        if result != xlib::Success as c_int { return None; }

        // If the return item size is not 8 bits, we definitely misfired and can't deal with that
        // here; return no data.
        if actual_format != 8 { return None; }

        // Cast the pointer given to us by XGetWindowProperty to a byte array
        let bytes = unsafe {
            std::slice::from_raw_parts(prop_ptr_ptr, nitems as usize)
        };

        let strings = decode_strings(bytes);

        unsafe {
            // As documented here:
            // https://tronche.com/gui/x/xlib/window-information/XGetWindowProperty.html
            // we must free the memory XGetWindowProperty allocated
            // using XFree.
            xlib::XFree(prop_ptr_ptr as *mut c_void);
        }

        Some(strings)
    }
}

///Decodes a nul-separated byte array into a Vec of strings
fn decode_strings(bytes: &[u8]) -> Vec<String> {
    let mut strings: Vec<String> = vec![];

    let mut slice = bytes;

    // A property can contain one or more strings in the byte array.
    // If there is more than one string, they are separated by a "nul" byte (0)
    loop {
        let slice_len = slice.len();

        // Find first position of 0 byte, falling back to 1 past the end of the slice,
        // if there aren't any 0 bytes.
        let nul_pos = slice.iter().position(|b| *b == 0).unwrap_or(slice_len);

        // Try to read a UTF-8 string up to the nul_pos or end of the slice.
        // If the bytes failed to parse as a string, it doesn't add one, but produces
        // no error otherwise.
        if let Ok(str) = String::from_utf8(slice[..nul_pos].to_vec()) {
            strings.push(str);
        }

        if nul_pos >= slice_len - 1 {
            // If the nul byte was at the end or beyond, then we're done.
            // This covers both the case where there was no nul byte, and when the
            // last byte in the slice is nul (terminating it)
            break;
        } else {
            // Otherwise, re-assign the slice to start one past nul_pos, and continue
            slice = &slice[nul_pos + 1..];
        }
    }

    strings
}