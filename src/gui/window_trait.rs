#[cfg(test)]
use mockall::{automock, predicate::*};

#[cfg_attr(test, automock)]
pub trait WindowTrait {
    fn load_image(&mut self, image_path: &str) -> anyhow::Result<()>;
    fn show_image(&self) -> anyhow::Result<()>;
    fn hide_image(&self) -> anyhow::Result<()>;
}
