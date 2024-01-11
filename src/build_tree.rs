use std::{
    cell::RefCell,
    collections::{HashMap, LinkedList},
    path::PathBuf,
    rc::Rc,
};

use geoutils::Location;
use ndarray::{ArrayBase, Dim, OwnedRepr};

use crate::types::*;

pub fn build_tree(
    l: &Vec<(PathBuf, Location)>,
    condensed_tree: &ArrayBase<OwnedRepr<(usize, usize, f64, usize)>, Dim<[usize; 1]>>,
) -> Tree {
    // child -> (parent, num of descendands)
    let cluster_map = condensed_tree.fold(
        HashMap::new(),
        |mut accum, (parent, child, _, num_of_leaves)| {
            accum.insert(*child, (*parent, NumberOfLeaves(*num_of_leaves)));
            accum
        },
    );

    let mut mns: HashMap<usize, _> = HashMap::new();

    let get_leaf = |child: usize| {
        let (path, location) = l
            .get(child)
            .expect("child index not is not a singleton cluster");
        let filepath = Filepath(String::from(
            path.to_str().expect("Cannot convert path to string"),
        ));
        let coord = Coord {
            lat: location.latitude(),
            lon: location.longitude(),
        };
        Tree::Leaf(filepath, coord)
    };

    let l_size = l.len();
    for (child, (parent, num_of_leaves)) in cluster_map.iter() {
        let node_to_update_parent_with = match mns.get(child) {
            None => {
                if child < &l_size {
                    let leaf = Rc::new(RefCell::new(get_leaf(*child)));

                    mns.insert(*child, leaf.clone());
                    leaf
                } else {
                    let node = Rc::new(RefCell::new(Tree::Node(*num_of_leaves, LinkedList::new())));

                    mns.insert(*child, node.clone());
                    node
                }
            }
            Some(tree) => tree.clone(),
        };

        match mns.get(parent) {
            Some(parent) => {
                let mut parent = parent.borrow_mut();
                match &mut *parent {
                    Tree::Node(_, l) => l.push_back(node_to_update_parent_with),
                    Tree::Leaf(_, _) => panic!("leaf cannot be a parent"),
                }
            }
            None => {
                match cluster_map.get(parent) {
                    Some((_, num_of_leaves)) => {
                        let node = Rc::new(RefCell::new(Tree::Node(
                            *num_of_leaves,
                            LinkedList::from([node_to_update_parent_with]),
                        )));

                        mns.insert(*parent, node);
                    }
                    None => {} // this can happen only for root cluster
                }
            }
        }
    }

    let children_of_root: LinkedList<_> = condensed_tree
        .iter()
        .filter(|(parent, _, _, _)| !cluster_map.contains_key(parent))
        .map(|(_, child, _, _)| mns.get(child).expect("Cannot find child").clone())
        .collect();

    let total_number_of_leaves =
        children_of_root
            .iter()
            .fold(NumberOfLeaves(0), |accum, child| -> NumberOfLeaves {
                match *child.borrow() {
                    Tree::Node(n, _) => accum + n,
                    Tree::Leaf(_, _) => accum + NumberOfLeaves(1),
                }
            });

    Tree::Node(total_number_of_leaves, children_of_root)
}
