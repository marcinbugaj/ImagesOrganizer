use std::collections::LinkedList;

use crate::types::*;

pub fn to_serde_tree(tree: Tree2) -> SerdeTree {
    match tree {
        Tree2::Node(convex_hull, number_of_leaves, children) => {
            let serde_children: LinkedList<_> = children
                .into_iter()
                .map(|n| Box::new(to_serde_tree(*n)))
                .collect();
            let serde_node = SerdeNode {
                convex_hull,
                number_of_leaves,
                children: serde_children,
            };
            SerdeTree {
                node: Some(serde_node),
                leaf: None,
            }
        }
        Tree2::Leaf(filepath, coord) => {
            let serde_leaf = SerdeLeaf {
                filepath,
                coord,
            };
            SerdeTree {
                node: None,
                leaf: Some(serde_leaf),
            }
        }
    }
}
