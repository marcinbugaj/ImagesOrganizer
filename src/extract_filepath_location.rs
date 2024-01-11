use std::{ffi::OsStr, path::PathBuf};

use exif::{In, Tag, Value};
use geoutils::Location;
use walkdir::DirEntry;

fn convert_min_to_decimal(deg: f64, min: f64, sec: f64) -> f64 {
    deg + (min / 60.) + (sec / 3600.)
}

pub fn extract_filepath_location(
    f: Result<DirEntry, walkdir::Error>,
) -> Option<(PathBuf, Location)> {
    let path = f.as_ref().ok()?.path();
    let is_file = path.is_file();
    let extension = path.extension().and_then(OsStr::to_str)?.to_lowercase();
    let accepted_extensions = ["jpg", "jpeg", "heif", "heic", "tiff", "png", "raw"];
    let is_image = accepted_extensions.contains(&extension.as_str());
    if !(is_file || !is_image) {
        return None;
    }

    let file = std::fs::File::open(&path).ok()?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).ok()?;

    // println!("Processing: {}", path.to_str().unwrap());

    let lat = exif.get_field(Tag::GPSLatitude, In::PRIMARY)?;
    let lon = exif.get_field(Tag::GPSLongitude, In::PRIMARY)?;
    let lat_ref = exif.get_field(Tag::GPSLatitudeRef, In::PRIMARY)?;
    let lon_ref = exif.get_field(Tag::GPSLongitudeRef, In::PRIMARY)?;

    let geoloc = match (&lat.value, &lon.value, &lat_ref.value, &lon_ref.value) {
        (
            Value::Rational(lat),
            Value::Rational(lon),
            Value::Ascii(lat_ref),
            Value::Ascii(lon_ref),
        ) => {
            let lat_sign = match lat_ref.first() {
                Some(c) if c[0].to_ascii_uppercase() == b'N' => 1.0,
                Some(c) if c[0].to_ascii_uppercase() == b'S' => -1.0,
                c => panic!("lat ref is: {:?}", c),
            };

            let lon_sign = match lon_ref.first() {
                Some(c) if c[0].to_ascii_uppercase() == b'E' => 1.0,
                Some(c) if c[0].to_ascii_uppercase() == b'W' => -1.0,
                c => panic!("lon ref is: {:?}", c),
            };

            let lat = lat_sign
                * convert_min_to_decimal(lat[0].to_f64(), lat[1].to_f64(), lat[2].to_f64());
            let lon = lon_sign
                * convert_min_to_decimal(lon[0].to_f64(), lon[1].to_f64(), lon[2].to_f64());
            Location::new(lat, lon)
        }
        _ => panic!("Cannot read coordinates from exif"),
    };

    // println!(
    //     "lat: {}, lon: {}",
    //     lat.display_value().with_unit(&exif),
    //     lon.display_value().with_unit(&exif)
    // );

    // println!("geoloc: {:?}", geoloc);

    Some((PathBuf::from(path), geoloc))
}
