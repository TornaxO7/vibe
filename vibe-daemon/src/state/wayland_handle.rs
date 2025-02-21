use std::{ffi::c_void, ptr::NonNull};

use raw_window_handle::{
    DisplayHandle, HasDisplayHandle, HasWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
    WindowHandle,
};
use smithay_client_toolkit::reexports::client::{
    protocol::{wl_display::WlDisplay, wl_surface::WlSurface},
    Connection,
};
use wgpu::SurfaceTarget;

pub struct WaylandHandle {
    conn: Connection,
    wl_surface: WlSurface,
}

impl WaylandHandle {
    pub fn new(conn: Connection, wl_surface: WlSurface) -> Self {
        Self { conn, wl_surface }
    }

    pub fn into_surface_target<'a>(self) -> SurfaceTarget<'a> {
        SurfaceTarget::Window(Box::new(self))
    }
}

impl HasDisplayHandle for WaylandHandle {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        let ptr = &mut self.conn.display() as *mut WlDisplay as *mut c_void;
        let handle = WaylandDisplayHandle::new(NonNull::new(ptr).unwrap());

        Ok(unsafe {
            DisplayHandle::borrow_raw(raw_window_handle::RawDisplayHandle::Wayland(handle))
        })
    }
}

impl HasWindowHandle for WaylandHandle {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let mut surface = self.wl_surface.clone();
        let ptr = &mut surface as *mut WlSurface as *mut c_void;
        let handle = WaylandWindowHandle::new(NonNull::new(ptr).unwrap());

        Ok(
            unsafe {
                WindowHandle::borrow_raw(raw_window_handle::RawWindowHandle::Wayland(handle))
            },
        )
    }
}
