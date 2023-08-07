use std::fs;
use serde::Deserialize;

fn main() {
    let home_dir = dirs::home_dir().expect("We have no home!");
    let app_dir = home_dir.join("wallhaven-plugin");
    let config_file = app_dir.join("wallhaven-plugin.json");
    let config_string = fs::read_to_string(config_file).expect("Unable to read config");
    let json: Config = serde_json::from_str(config_string.as_str()).expect("Invalid config");
    println!("{:#?}", json);
}

fn find_matching_wallpaper(config: &Config) -> reqwest::Result<Option<WallpaperInfo>> {
    let mut url = String::from("https://wallhaven.cc/api/v1/search?sorting=random");
    if let Some(q) = &config.q {
        url.push_str("&q=");
        url.push_str(q.as_str());
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

    #[test]
    fn valid_simple_query() {
        let result = find_matching_wallpaper(&Config {
            q: Some("car".to_string()), ..Default::default()
        }).unwrap();
        println!("{:#?}", result);
        assert!(result.is_some());
    }

    #[test]
    fn no_matches() {
        let rand_num: u128 = random();
        let result = find_matching_wallpaper(&Config {
            q: Some(rand_num.to_string()), ..Default::default()
        }).unwrap();
        println!("{:#?}", result);
        assert!(result.is_none());
    }

    #[test]
    fn no_query() {
        let result = find_matching_wallpaper(&Default::default()).unwrap();
        println!("{:#?}", result);
        assert!(result.is_some());
    }
}
