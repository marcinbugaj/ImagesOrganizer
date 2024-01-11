use geoutils::Location;
use ndarray::ArrayView1;
use petal_neighbors::distance::Metric;

use crate::types::Haversine;

impl Metric<f64> for Haversine {
    /// Euclidean distance metric.
    fn distance(&self, x1: &ArrayView1<f64>, x2: &ArrayView1<f64>) -> f64 {
        let snd = Location::new_const(x2[0], x2[1]);
        let fst = Location::new_const(x1[0], x1[1]);
        fst.haversine_distance_to(&snd).meters()
    }
    /// Euclidean reduce distance metric.
    fn rdistance(&self, _x1: &ArrayView1<f64>, _x2: &ArrayView1<f64>) -> f64 {
        todo!();
    }
    fn rdistance_to_distance(&self, _d: f64) -> f64 {
        todo!();
    }

    /// Euclidean reduce distance metric.
    fn distance_to_rdistance(&self, _d: f64) -> f64 {
        todo!();
    }
}
