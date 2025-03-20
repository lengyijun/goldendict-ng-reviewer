use anyhow::Result;
use std::fs;
use std::path::PathBuf;

fn favourites_path() -> PathBuf {
    dirs::config_dir()
        .unwrap()
        .join("goldendict")
        .join("favorites")
}

pub fn extract_words_from_favorites_folder(folder_name: &str) -> Result<Vec<String>> {
    let s = fs::read_to_string(favourites_path()).unwrap();
    let doc = roxmltree::Document::parse(&s).unwrap();

    let folder: roxmltree::Node<'_, '_> = doc
        .descendants()
        .find(|n| n.attribute("name") == Some(folder_name))
        .unwrap();

    let v: Vec<_> = folder
        .children()
        .flat_map(|x| x.children())
        .flat_map(|x| x.text())
        .map(str::to_owned)
        .collect();

    Ok(v)
}

pub fn extract_all_words_from_favorites() -> Result<Vec<String>> {
    let s = fs::read_to_string(favourites_path()).unwrap();
    let doc = roxmltree::Document::parse(&s).unwrap();

    let v: Vec<_> = doc
        .descendants()
        .flat_map(|x| x.children())
        .flat_map(|x| x.children())
        .flat_map(|x| x.text())
        .filter(|s| !s.chars().all(|c| c.is_whitespace()))
        .map(str::to_owned)
        .collect();

    Ok(v)
}
