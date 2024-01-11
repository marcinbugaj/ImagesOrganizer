use geo::{point, ConvexHull, MultiPoint};

use crate::types::*;
use std::collections::LinkedList;

fn get_convex_hull(children: &LinkedList<Box<Tree2>>) -> ConvexHull2 {
    let multi_point: MultiPoint<f64> = children
        .iter()
        .flat_map(|t| match &**t {
            Tree2::Node(convex_hull, _, _) => convex_hull.0.clone(),
            Tree2::Leaf(_, coord) => LinkedList::from([*coord]),
        })
        .map(|f| point! {x: f.lat, y: f.lon})
        .collect();

    let ch: LinkedList<_> = multi_point
        .convex_hull()
        .exterior()
        .into_iter()
        .map(|c| Coord { lat: c.x, lon: c.y })
        .collect();

    ConvexHull2(ch)
}

pub fn convert_tree(tree: &mut Tree) -> Tree2 {
    match tree {
        Tree::Node(n, l) => {
            let children = l
                .iter()
                .map(|r| Box::new(convert_tree(&mut *r.borrow_mut())))
                .collect();

            let convex_hull = get_convex_hull(&children);

            return Tree2::Node(convex_hull, *n, children);
        }
        Tree::Leaf(f, c) => Tree2::Leaf(f.clone(), *c),
    }
}
