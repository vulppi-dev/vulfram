use std::marker::PhantomData;

use web_sys::HtmlCanvasElement;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u32);

#[derive(Debug, Clone)]
pub struct Window {
    id: WindowId,
    _canvas: HtmlCanvasElement,
}

impl Window {
    #[cfg(target_arch = "wasm32")]
    pub fn new(id: u32, canvas: HtmlCanvasElement) -> Self {
        Self {
            id: WindowId(id),
            _canvas: canvas,
        }
    }

    pub fn id(&self) -> WindowId {
        self.id
    }
}

pub type ActiveEventLoop = ();

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
