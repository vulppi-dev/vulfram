use std::marker::PhantomData;
use std::time::Duration;

use web_sys::HtmlCanvasElement;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u32);

#[derive(Debug, Clone)]
pub struct Window {
    id: WindowId,
    canvas: HtmlCanvasElement,
}

impl Window {
    pub fn new(id: u32, canvas: HtmlCanvasElement) -> Self {
        Self {
            id: WindowId(id),
            canvas,
        }
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn request_redraw(&self) {}

    pub fn is_visible(&self) -> Option<bool> {
        Some(true)
    }

    pub fn set_title(&self, _title: &str) {}

    pub fn set_outer_position(&self, _pos: winit::dpi::PhysicalPosition<i32>) {}

    pub fn outer_position(&self) -> Result<winit::dpi::PhysicalPosition<i32>, ()> {
        Ok(winit::dpi::PhysicalPosition::new(0, 0))
    }

    pub fn request_inner_size(&self, size: winit::dpi::PhysicalSize<u32>) -> Option<()> {
        self.canvas.set_width(size.width);
        self.canvas.set_height(size.height);
        Some(())
    }

    pub fn inner_size(&self) -> winit::dpi::PhysicalSize<u32> {
        winit::dpi::PhysicalSize::new(self.canvas.width(), self.canvas.height())
    }

    pub fn outer_size(&self) -> winit::dpi::PhysicalSize<u32> {
        winit::dpi::PhysicalSize::new(self.canvas.width(), self.canvas.height())
    }

    pub fn set_minimized(&self, _minimized: bool) {}

    pub fn set_maximized(&self, _maximized: bool) {}

    pub fn set_fullscreen(&self, _fullscreen: Option<winit::window::Fullscreen>) {}

    pub fn current_monitor(&self) -> Option<winit::window::MonitorHandle> {
        None
    }

    pub fn is_minimized(&self) -> Option<bool> {
        None
    }

    pub fn is_maximized(&self) -> bool {
        false
    }

    pub fn fullscreen(&self) -> Option<winit::window::Fullscreen> {
        None
    }

    pub fn set_window_icon(&self, _icon: Option<winit::window::Icon>) {}

    pub fn request_user_attention(&self, _attention: Option<winit::window::UserAttentionType>) {}

    pub fn focus_window(&self) {}

    pub fn set_decorations(&self, _decorations: bool) {}

    pub fn is_decorated(&self) -> bool {
        true
    }

    pub fn set_resizable(&self, _resizable: bool) {}

    pub fn is_resizable(&self) -> bool {
        true
    }

    pub fn set_cursor_visible(&self, _visible: bool) {}

    pub fn set_cursor_grab(&self, _mode: winit::window::CursorGrabMode) -> Result<(), ()> {
        Ok(())
    }

    pub fn set_cursor(&self, _cursor: winit::window::CursorIcon) {}
}

pub struct ActiveEventLoop;

pub struct EventLoop<T> {
    _phantom: PhantomData<T>,
}

pub struct EventLoopBuilder<T> {
    _phantom: PhantomData<T>,
}

impl<T> EventLoop<T> {
    pub fn with_user_event() -> EventLoopBuilder<T> {
        EventLoopBuilder {
            _phantom: PhantomData,
        }
    }

    pub fn create_proxy(&self) -> EventLoopProxy<T> {
        EventLoopProxy {
            _phantom: PhantomData,
        }
    }
}

impl<T> EventLoopBuilder<T> {
    pub fn build(self) -> Result<EventLoop<T>, ()> {
        Ok(EventLoop {
            _phantom: PhantomData,
        })
    }
}

pub struct EventLoopProxy<T> {
    _phantom: PhantomData<T>,
}

impl<T> EventLoopProxy<T> {
    pub fn send_event(&mut self, _event: T) -> Result<(), ()> {
        Ok(())
    }
}

pub trait ApplicationHandler<T> {}

pub trait EventLoopExtPumpEvents {
    fn pump_app_events(&mut self, _timeout: Option<Duration>, _handler: &mut dyn ApplicationHandler<()>) {
    }
}

impl<T> EventLoopExtPumpEvents for EventLoop<T> {
    fn pump_app_events(&mut self, _timeout: Option<Duration>, _handler: &mut dyn ApplicationHandler<()>) {
    }
}

pub mod winit {
    pub mod event {
        #[derive(Debug, Clone, Copy)]
        pub enum WindowEvent {}

        #[derive(Debug, Clone, Copy)]
        pub enum MouseButton {}

        #[derive(Debug, Clone, Copy)]
        pub enum TouchPhase {}
    }

    pub mod keyboard {
        #[derive(Debug, Clone, Copy)]
        pub enum KeyLocation {}

        #[derive(Debug, Clone, Copy)]
        pub enum PhysicalKey {}

        #[derive(Debug, Clone, Copy)]
        pub enum KeyCode {}
    }

    pub mod window {
        #[derive(Debug, Clone, Copy)]
        pub enum Theme {
            Dark,
            Light,
        }

        #[derive(Debug, Clone, Copy)]
        pub enum CursorGrabMode {
            None,
            Confined,
            Locked,
        }

        #[derive(Debug, Clone, Copy)]
        pub enum CursorIcon {
            Default,
            ContextMenu,
            Help,
            Pointer,
            Progress,
            Wait,
            Cell,
            Crosshair,
            Text,
            VerticalText,
            Alias,
            Copy,
            Move,
            NoDrop,
            NotAllowed,
            Grab,
            Grabbing,
            EResize,
            NResize,
            NeResize,
            NwResize,
            SResize,
            SeResize,
            SwResize,
            WResize,
            EwResize,
            NsResize,
            NeswResize,
            NwseResize,
            ColResize,
            RowResize,
            AllScroll,
            ZoomIn,
            ZoomOut,
        }

        #[derive(Debug, Clone, Copy)]
        pub enum UserAttentionType {
            Critical,
            Informational,
        }

        #[derive(Debug, Clone, Copy)]
    pub struct MonitorHandle;

    impl MonitorHandle {
        pub fn video_modes(&self) -> std::vec::IntoIter<VideoMode> {
            Vec::new().into_iter()
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct VideoMode;

        #[derive(Debug, Clone, Copy)]
        pub enum Fullscreen {
            Exclusive(VideoMode),
            Borderless(Option<MonitorHandle>),
        }

        #[derive(Debug, Clone, Copy)]
        pub struct Icon;

        impl Icon {
            pub fn from_rgba(_rgba: Vec<u8>, _width: u32, _height: u32) -> Result<Self, ()> {
                Ok(Icon)
            }
        }
    }

    pub mod dpi {
        #[derive(Debug, Clone, Copy, Default)]
        pub struct PhysicalPosition<T> {
            pub x: T,
            pub y: T,
        }

        impl<T> PhysicalPosition<T> {
            pub fn new(x: T, y: T) -> Self {
                Self { x, y }
            }
        }

        #[derive(Debug, Clone, Copy, Default)]
        pub struct PhysicalSize<T> {
            pub width: T,
            pub height: T,
        }

        impl<T> PhysicalSize<T> {
            pub fn new(width: T, height: T) -> Self {
                Self { width, height }
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub enum Position {
            Physical(PhysicalPosition<i32>),
        }
    }
}
