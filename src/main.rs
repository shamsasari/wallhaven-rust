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
    let result = find_matching_wallpaper(&config, &resolution).expect("Unable to query for wallpaper");
    let wallpaper_info = result.expect("No matching wallpaper found");
    println!("{:#?}", &wallpaper_info);
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
            return Ok(Some(info));
        }
        println!("{} does not match", info.url)
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
