use serde::Deserialize;

/// A screenshot the backend saved on disk (temporary).
#[derive(Deserialize, Debug, Clone)]
pub struct ScreenShot {
    pub id: String,
    pub image_path: String,
    pub width: u32,
    pub height: u32,
}
