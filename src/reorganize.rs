use core::panic;
use exif::{DateTime, In, Tag};
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, create_dir};
use std::hash::{Hash, Hasher};
use std::io::{self, Error};
use std::path::Path;

use crate::types::{Cluster, Commit, Filepath};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MyDateTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

fn to_my_date_time(f: DateTime) -> MyDateTime {
    MyDateTime {
        year: f.year,
        month: f.month,
        day: f.day,
        hour: f.hour,
        minute: f.minute,
        second: f.second,
    }
}

fn extract_date_time(filepath: &Filepath) -> Option<MyDateTime> {
    let path = Path::new(filepath.0.as_str());
    let file = std::fs::File::open(path).ok()?;
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).ok()?;

    let date_time_field = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY)?;

    let datetime = match &date_time_field.value {
        exif::Value::Ascii(ascii) => {
            let f = ascii.first()?;
            DateTime::from_ascii(f).ok()
        }
        _ => None,
    }?;

    return Some(to_my_date_time(datetime));
}

fn to_string_date_time(date_time: &MyDateTime) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        date_time.year, date_time.month, date_time.day
    )
}

fn reorganize_single_cluster(
    cluster: &Cluster,
    base_folder: &Path,
    dryrun: bool,
) -> io::Result<()> {
    let mut with_datetime: Vec<_> = cluster
        .0
        .iter()
        .map(|path| (path, extract_date_time(path)))
        .collect();

    with_datetime.sort_by(|(_, d1), (_, d2)| d1.cmp(d2));

    let min = with_datetime
        .iter()
        .find(|(_, d)| d.is_some())
        .and_then(|(_, d)| *d);
    let max = with_datetime
        .iter()
        .rfind(|(_, d)| d.is_some())
        .and_then(|(_, d)| *d);

    let name = match (min, max) {
        (None, None) => String::from("unknown"),
        (Some(d1), Some(d2)) => {
            if d1 == d2 {
                to_string_date_time(&d1)
            } else {
                format!("{}_{}", to_string_date_time(&d1), to_string_date_time(&d2))
            }
        }
        _ => panic! {"impossible"},
    };

    let final_folder_name = format!("{}__{:06x}", name, calculate_hash(&cluster));
    let folder_path = base_folder.join(&final_folder_name);

    if dryrun {
        println!("Creating directory: {:?}", &folder_path);
    } else {
        create_dir(&folder_path)?;
    }

    let mut idx = 0;
    for (file, _) in with_datetime.iter() {
        let path = Path::new(file.0.as_str());
        let ext = path
            .extension()
            .expect("Extension should be there")
            .to_str()
            .expect("Cannot convert to string")
            .to_string();
        let new_filename = format!("{}.{}", idx, ext);
        let filepath = folder_path.join(&new_filename);

        if dryrun {
            assert_directory_has_write_permission(
                path.parent().expect("Path in cluster is not a file path"),
            )?;

            println!("Moving file from {:?} to {:?} ", path, filepath);
        } else {
            fs::rename(path, filepath)?;
        }

        idx += 1;
    }

    Ok(())
}

fn assert_directory_has_write_permission(loc: &Path) -> io::Result<()> {
    let md = fs::metadata(loc)?;
    let permissions = md.permissions();
    if permissions.readonly() {
        let msg = format!("Directory {:?} has no write permission", loc);
        Err(Error::other(msg))
    } else {
        Ok(())
    }
}

pub fn reorganize(commit: Commit) -> io::Result<()> {
    let top_folder = Path::new(commit.folder.0.as_str());
    let output_folder = top_folder.join("categorized");

    if commit.dryrun {
        assert_directory_has_write_permission(top_folder)?;
        println!("Creating directory: {:?}", output_folder);
    } else {
        create_dir(&output_folder)?;
    }

    commit
        .clusters
        .0
        .into_iter()
        .fold(Ok(()), |accum, cluster| {
            accum.and_then(|_| reorganize_single_cluster(&cluster, &output_folder, commit.dryrun))
        })
}
