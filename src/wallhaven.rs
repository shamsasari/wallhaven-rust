use serde::Deserialize;
use winit::dpi::PhysicalSize;

pub fn search(q: Option<&str>, atleast: &PhysicalSize<u32>) -> reqwest::Result<QueryResult> {
    let mut url = String::from("https://wallhaven.cc/api/v1/search?sorting=random");
    url.push_str(format!("&atleast={}x{}", &atleast.width, &atleast.height).as_str());
    if let Some(q) = q {
        url.push_str("&q=");
        url.push_str(q);
    }
    reqwest::blocking::get(url)?.json()
}

pub fn get_wallpaper(id: &str) -> reqwest::Result<Wallpaper> {
    let response: WallpaperWrapper = reqwest::blocking::get(format!("https://wallhaven.cc/api/v1/w/{id}"))?.json()?;
    Ok(response.data)
}

#[derive(Deserialize, Debug)]
pub struct QueryResult {
    pub data: Vec<WallpaperInfo>,
    pub meta: Metadata,
}

#[derive(Deserialize, Debug)]
pub struct WallpaperInfo {
    pub id: String,
    pub url: String,
    pub path: String,
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub last_page: u32,
    pub seed: String,
}

#[derive(Deserialize, Debug)]
struct WallpaperWrapper {
    data: Wallpaper,
}

#[derive(Deserialize, Debug)]
pub struct Wallpaper {
    pub tags: Vec<Tag>,
}

#[derive(Deserialize, Debug)]
pub struct Tag {
    pub name: String,
}
