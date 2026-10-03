use serde::Serialize;

/// Screenshort which will save in the disk(temporary)
#[derive(Serialize, Debug, Clone)]
pub struct ScreenShot {
    pub id: String,
    pub image_path: String,
    pub width: u32,
    pub height: u32,
}
