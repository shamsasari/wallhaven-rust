#![windows_subsystem = "windows"]

#[cfg(windows)]
extern crate winapi;

use std::{env, fs};
use std::error::Error;
use std::ffi::CString;
use std::path::PathBuf;

use log::{info, LevelFilter, warn};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use serde::Deserialize;
use winapi::um::winnt::PVOID;
use winapi::um::winuser::{SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SystemParametersInfoA};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn Error>> {
    let home_dir = dirs::home_dir().ok_or("Unable to determine user home dir")?;
    let app_dir = home_dir.join("wallhaven-plugin");

    init_logging(&app_dir);

    let config_file = app_dir.join("wallhaven-plugin.json");
    let config_string = fs::read_to_string(config_file)?;
    let config: Config = serde_json::from_str(config_string.as_str())?;

    let monitor = EventLoop::new().available_monitors().next().ok_or("Unable to find monitor")?;
    let result = find_matching_wallpaper(&config, &monitor.size())?;

    let wallpaper_info = match result {
        Some(w) => w,
        None => {
            warn!("No matching wallpaper found");
            return Ok(());
        },
    };

    let wallhaven_temp_dir = env::temp_dir().join("wallhaven");
    fs::create_dir_all(&wallhaven_temp_dir)?;
    let wallpaper_file = wallhaven_temp_dir.join(&wallpaper_info.id);
    let wallpaper_bytes = reqwest::blocking::get(&wallpaper_info.path)?.bytes()?;
    fs::write(&wallpaper_file, wallpaper_bytes)?;

    let wallpaper_path_string = wallpaper_file
        .into_os_string()
        .into_string()
        .map_err(|e| "Invalid wallpaper file path")?;
    let wallpaper_file = CString::new(wallpaper_path_string)?;

    unsafe {
        SystemParametersInfoA(
            SPI_SETDESKWALLPAPER,
            0,
            wallpaper_file.as_ptr() as PVOID,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE
        );
    }

    Ok(())
}

fn init_logging(app_dir: &PathBuf) {
    let main_log = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} - {m}{n}")))
        .build(app_dir.join("main.log"))
        .unwrap();

    let config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("main_log", Box::new(main_log)))
        .build(Root::builder().appender("main_log").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();
}

fn find_matching_wallpaper(config: &Config, resolution: &PhysicalSize<u32>) -> reqwest::Result<Option<WallpaperInfo>> {
    let exclude_similar_tags: Vec<String> = config.exclude_similar_tags
        .iter()
        .map(|tag| tag.to_lowercase())
        .collect();

    let mut url = String::from("https://wallhaven.cc/api/v1/search?sorting=random");
    url.push_str(format!("&resolutions={}x{}", &resolution.width, &resolution.height).as_str());
    if let Some(q) = &config.q {
        url.push_str("&q=");
        url.push_str(q);
    }
    let result: QueryResult = reqwest::blocking::get(url)?.json()?;

    for info in result.data {
        let tags = get_wallpaper_tags(&info.id)?;
        let matching = tags.iter().all(|tag| {
            tag_is_not_excluded(&exclude_similar_tags, tag)
        });
        if matching {
            info!("{} {:?}", info.url, tags);
            return Ok(Some(info));
        }
        info!("{} does not match", info.url)
    };

    return Ok(None);
}

fn tag_is_not_excluded(exclude_similar_tags: &Vec<String>, tag: &String) -> bool {
    exclude_similar_tags.iter().all(|exclude| !tag.contains(exclude))
}

fn get_wallpaper_tags(id: &str) -> reqwest::Result<Vec<String>> {
    let response: WallpaperDetailWrapper = reqwest::blocking::get(format!("https://wallhaven.cc/api/v1/w/{id}"))?.json()?;
    let tag_names = response.data.tags
        .iter()
        .map(|tag| tag.name.to_lowercase())
        .collect();
    Ok(tag_names)
}

#[derive(Deserialize, Debug, Default)]
struct Config {
    q: Option<String>,
    #[serde(rename = "excludeSimilarTags", default)]
    exclude_similar_tags: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct QueryResult {
    data: Vec<WallpaperInfo>,
    meta: Metadata,
}

#[derive(Deserialize, Debug)]
struct WallpaperInfo {
    id: String,
    url: String,
    path: String,
}

#[derive(Deserialize, Debug)]
struct Metadata {
    last_page: u32,
    seed: String,
}

#[derive(Deserialize, Debug)]
struct WallpaperDetailWrapper {
    data: WallpaperDetail,
}

#[derive(Deserialize, Debug)]
struct WallpaperDetail {
    tags: Vec<WallpaperTag>,
}

#[derive(Deserialize, Debug)]
struct WallpaperTag {
    name: String,
}


#[cfg(test)]
mod manual_tests {
    use rand::random;
    use super::*;

    static DEFAULT_RESOLUTION: PhysicalSize<u32> = PhysicalSize::new(3840, 2160);

    #[test]
    fn valid_simple_query() {
        let result = find_matching_wallpaper(
            &Config { q: Some("car".to_string()), ..Default::default() },
            &DEFAULT_RESOLUTION
        ).unwrap();
        println!("{:#?}", result);
        assert!(result.is_some());
    }

    #[test]
    fn no_matches() {
        let rand_num: u128 = random();
        let result = find_matching_wallpaper(
            &Config { q: Some(rand_num.to_string()), ..Default::default() },
            &DEFAULT_RESOLUTION
        ).unwrap();
        println!("{:#?}", result);
        assert!(result.is_none());
    }

    #[test]
    fn no_query() {
        let result = find_matching_wallpaper(&Default::default(), &DEFAULT_RESOLUTION).unwrap();
        println!("{:#?}", result);
        assert!(result.is_some());
    }
}
