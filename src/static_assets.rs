use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/ui/build"]
struct Asset;

pub struct WebPath(pub String);
pub struct FileContent(pub String);

pub fn get_assets() -> Vec<(WebPath, FileContent)> {
    let mut vec = Vec::new();

    Asset::iter().for_each(|path| {
        // println!("processing: {}", path);

        let embedded_file = Asset::get(&path).unwrap();
        let as_string = std::str::from_utf8(embedded_file.data.as_ref()).unwrap();
        vec.push((
            WebPath(path.to_string()),
            FileContent(as_string.to_string()),
        ));
    });

    vec
}
