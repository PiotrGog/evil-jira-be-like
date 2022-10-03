mod window;
mod window_trait;

pub use window::Window;
pub use window_trait::WindowTrait;

#[cfg(test)]
pub mod testing {
    pub use super::window_trait::MockWindowTrait;
}
