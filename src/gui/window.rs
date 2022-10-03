use image::{DynamicImage, GenericImageView};
use show_image::{create_window, ImageInfo, ImageView, WindowOptions, WindowProxy};

use super::window_trait::WindowTrait;

pub struct Window {
    window: WindowProxy,
    image: DynamicImage,
}

impl WindowTrait for Window {
    fn load_image(&mut self, image_path: &str) -> anyhow::Result<()> {
        self.image = image::open(image_path)?;
        Ok(())
    }

    fn show_image(&self) -> anyhow::Result<()> {
        let (width, height) = self.image.dimensions();
        let image = ImageView::new(ImageInfo::rgb8(width, height), self.image.as_bytes());
        self.window.set_image(Self::WINDOW_TITLE, image)?;
        self.window.run_function(|mut win| win.set_visible(true));
        Ok(())
    }

    fn hide_image(&self) -> anyhow::Result<()> {
        self.window.run_function(|mut win| win.set_visible(false));
        Ok(())
    }
}

impl Window {
    const WINDOW_TITLE: &'static str = "Evil JIRA be like";

    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            window: create_window(
                Self::WINDOW_TITLE,
                WindowOptions {
                    start_hidden: true,
                    ..Default::default()
                },
            )?,
            image: DynamicImage::new_rgb8(0, 0),
        })
    }
}
