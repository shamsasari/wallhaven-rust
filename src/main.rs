use std::fs;
use serde::Deserialize;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;

fn main() {
    let home_dir = dirs::home_dir().expect("We have no home!");
    let app_dir = home_dir.join("wallhaven-plugin");
    let config_file = app_dir.join("wallhaven-plugin.json");
    let config_string = fs::read_to_string(config_file).expect("Unable to read config");
    let config: Config = serde_json::from_str(config_string.as_str()).expect("Invalid config");
    let monitor = EventLoop::new().available_monitors().next().expect("No monitors found!");
    let resolution = monitor.size();
    let result = find_matching_wallpaper(&config, &resolution);
    println!("{:#?}", result)
}

fn find_matching_wallpaper(config: &Config, resolution: &PhysicalSize<u32>) -> reqwest::Result<Option<WallpaperInfo>> {
    let mut url = String::from("https://wallhaven.cc/api/v1/search?sorting=random");
    url.push_str("&resolutions=");
    url.push_str(&resolution.width.to_string());
    url.push('x');
    url.push_str(&resolution.height.to_string());
    if let Some(q) = &config.q {
        url.push_str("&q=");
        url.push_str(q);
    }
    let response: QueryResult = reqwest::blocking::get(url)?.json()?;
    Ok(response.data.into_iter().next())
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
