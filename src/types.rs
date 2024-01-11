use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::LinkedList, ops::Add, rc::Rc};

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct Haversine {}

unsafe impl Sync for Haversine {}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Copy)]
pub struct NumberOfLeaves(pub usize);

impl Add for NumberOfLeaves {
    type Output = NumberOfLeaves;

    fn add(self, rhs: Self) -> Self::Output {
        NumberOfLeaves(self.0 + rhs.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Copy)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Filepath(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Folder(pub String);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Centroid(pub Coord);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConvexHull2(pub LinkedList<Coord>);

pub enum Tree {
    Node(NumberOfLeaves, LinkedList<Rc<RefCell<Tree>>>),
    Leaf(Filepath, Coord),
}

#[derive(Clone, Debug)]
pub enum Tree2 {
    Node(ConvexHull2, NumberOfLeaves, LinkedList<Box<Tree2>>),
    Leaf(Filepath, Coord),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerdeNode {
    pub convex_hull: ConvexHull2,
    pub number_of_leaves: NumberOfLeaves,
    pub children: LinkedList<Box<SerdeTree>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerdeLeaf {
    pub filepath: Filepath,
    pub coord: Coord,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerdeTree {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<SerdeNode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub leaf: Option<SerdeLeaf>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Cluster(pub Vec<Filepath>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clusters(pub Vec<Cluster>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commit {
    pub clusters: Clusters,
    pub folder: Folder,
    pub dryrun: bool,
}
