use std::path::Path;
use ndarray::Array2;
use petal_clustering::HDbscan;
use walkdir::WalkDir;

use crate::types::*;

use crate::build_tree::*;
use crate::convert_tree::*;
use crate::extract_filepath_location::*;
use crate::to_serde_tree::*;

fn get_number_of_leaves(tree: &Tree2) -> usize {
    match tree {
        Tree2::Node(_, n, _) => n.0,
        Tree2::Leaf(_, _) => 1,
    }
}

fn validate_tree(tree: &Tree2) {
    match tree {
        Tree2::Node(_, n, children) => {
            let calculated = children
                .into_iter()
                .fold(0, |accum, f| accum + get_number_of_leaves(f));
            if n.0 != calculated {
                panic!("Validation error");
            }
        }
        Tree2::Leaf(_, _) => {}
    }
}

pub fn launch_pipeline_for_directory(path: &Path) -> Option<SerdeTree> {
    let jpegs_with_geo: Vec<_> = WalkDir::new(path)
        .into_iter()
        .filter_map(extract_filepath_location)
        .collect();

    if jpegs_with_geo.len() < 5 {
        return None;
    }

    let hdbscan_input_vector: Vec<_> = jpegs_with_geo
        .iter()
        .map(|(_, geoloc)| [geoloc.latitude(), geoloc.longitude()])
        .collect();

    let geolocs = Array2::from(hdbscan_input_vector);

    let mut hdbscan = HDbscan {
        eps: f64::MAX,
        alpha: 1.,
        min_samples: 2,
        min_cluster_size: 4,
        metric: Haversine::default(),
        boruvka: false,
    };

    let condensed_tree = hdbscan.compute_condensed(&geolocs);
    let mut tree = build_tree(&jpegs_with_geo, &condensed_tree);
    let tree2 = convert_tree(&mut tree);
    validate_tree(&tree2);

    let serde_tree = to_serde_tree(tree2);

    // println!(
    //     "converted tree: {:#?}",
    //     tree2 /*serde_json::to_string(&cc).unwrap()*/
    // );
    // println!("number of pictures: {}", jpegs_with_geo.len());
    // println!(
    //     "number of leaves pointed by root: {}",
    //     get_number_of_leaves(&tree2)
    // );

    Some(serde_tree)
}
