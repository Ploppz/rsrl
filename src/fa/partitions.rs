use super::{Function, Parameterised, QFunction};

use ndarray::{arr1, Array1, Array2};
use geometry::{Span, Space, RegularSpace};
use geometry::dimensions::{Partition, Continuous};


/// Linearly partitioned function representation.
pub struct Partitions {
    weights: Array2<f64>,
    input_space: RegularSpace<Partition>,
}

impl Partitions {
    pub fn new(input_space: RegularSpace<Partition>,
               n_outputs: usize) -> Self
    {
        let n_features = match input_space.span() {
            Span::Finite(s) => s,
            _ => panic!("`Partitions` function approximator only supports \
                         finite input spaces.")
        };

        Partitions {
            weights: Array2::<f64>::zeros((n_features, n_outputs)),
            input_space: input_space,
        }
    }

    fn hash(&self, input: &[f64]) -> usize {
        let mut in_it = input.iter().rev();
        let mut d_it = self.input_space.iter().rev();

        let acc = d_it.next().unwrap().to_partition(in_it.next().unwrap());

        in_it.zip(d_it).fold(acc, |acc, (v, d)| {
            let i = d.to_partition(v);

            i + d.density() * acc
        })
    }
}


impl Function<Vec<f64>, f64> for Partitions {
    fn evaluate(&self, input: &Vec<f64>) -> f64 {
        // Hash the input down to an index:
        self.weights[[self.hash(input), 0]]
    }
}

impl Function<Vec<f64>, Vec<f64>> for Partitions {
    fn evaluate(&self, input: &Vec<f64>) -> Vec<f64> {
        // Hash the input down to a row index:
        let ri = self.hash(input);

        // Get the row slice and convert to a Vec<f64>:
        self.weights.row(ri).to_vec()
    }
}


impl Parameterised<Vec<f64>, f64> for Partitions {
    fn update(&mut self, input: &Vec<f64>, error: f64) {
        let index = self.hash(input);

        unsafe {
            *self.weights.uget_mut((index, 0)) += error
        }
    }
}

impl Parameterised<Vec<f64>, Vec<f64>> for Partitions {
    fn update(&mut self, input: &Vec<f64>, errors: Vec<f64>) {
        // Hash the input down to a row index:
        let ri = self.hash(input);

        // Get the row slice and perform update via memcpy:
        self.weights.row_mut(ri).scaled_add(1.0, &arr1(&errors));
    }
}


impl QFunction<RegularSpace<Continuous>> for Partitions
{
    fn evaluate_action(&self, input: &Vec<f64>, action: usize) -> f64 {
        // Hash the input down to a row index:
        let ri = self.hash(input);

        self.weights[[ri, action]]
    }

    fn update_action(&mut self, input: &Vec<f64>, action: usize, error: f64) {
        let index = self.hash(input);

        unsafe {
            *self.weights.uget_mut((index, action)) += error
        }
    }

    fn phi(&self, input: &Vec<f64>) -> Array1<f64> {
        let mut p = Array1::<f64>::zeros(self.weights.len());
        p[self.hash(input)] = 1.0;

        p
    }
}


#[cfg(test)]
mod tests {
    use super::Partitions;

    use fa::{Function, Parameterised};
    use geometry::RegularSpace;
    use geometry::dimensions::Partition;

    #[test]
    fn test_update_eval() {
        let mut ds = RegularSpace::new();
        ds = ds.push(Partition::new(0.0, 9.0, 10));

        let mut t = Partitions::new(ds, 1);

        t.update(&vec![1.5], 25.5);

        let out: f64 = t.evaluate(&vec![1.5]);
        assert_eq!(out, 25.5);

        t.update(&vec![1.5], -12.75);

        let out: f64 = t.evaluate(&vec![1.5]);
        assert_eq!(out, 12.75);
    }

    #[test]
    fn test_collisions() {
        let mut ds = RegularSpace::new();
        ds = ds.push(Partition::new(0.0, 9.0, 10));

        let mut t = Partitions::new(ds, 1);

        t.update(&vec![0.5], vec![1.2]);

        for value in vec![0.2, 0.5, 0.8] {
            let out: f64 = t.evaluate(&vec![value]);
            assert_eq!(out, 1.2);
        }
    }

    #[test]
    fn test_1d() {
        let mut ds = RegularSpace::new();
        ds = ds.push(Partition::new(0.0, 9.0, 10));

        let mut t = Partitions::new(ds, 1);

        for i in 0..10 {
            let input: Vec<f64> = vec![i as u32 as f64];

            let out: f64 = t.evaluate(&input);
            assert_eq!(out, 0.0);

            t.update(&input, vec![1.0]);

            let out: f64 = t.evaluate(&input);
            assert_eq!(out, 1.0);
        }
    }

    #[test]
    fn test_2d() {
        let mut ds = RegularSpace::new();
        ds = ds.push(Partition::new(0.0, 9.0, 10));
        ds = ds.push(Partition::new(0.0, 9.0, 10));

        let mut t = Partitions::new(ds, 1);

        for i in 0..10 {
            for j in 0..10 {
                let input: Vec<f64> = vec![i as u32 as f64, j as u32 as f64];

                let out: f64 = t.evaluate(&input);
                assert_eq!(out, 0.0);

                t.update(&input, vec![1.0]);

                let out: f64 = t.evaluate(&input);
                assert_eq!(out, 1.0);
            }
        }
    }

    #[test]
    fn test_3d() {
        let mut ds = RegularSpace::new();
        ds = ds.push(Partition::new(0.0, 9.0, 10));
        ds = ds.push(Partition::new(0.0, 9.0, 10));
        ds = ds.push(Partition::new(0.0, 9.0, 10));

        let mut t = Partitions::new(ds, 1);

        for i in 0..10 {
            for j in 0..10 {
                for k in 0..10 {
                    let input: Vec<f64> = vec![i as u32 as f64, j as u32 as f64, k as u32 as f64];

                    let out: f64 = t.evaluate(&input);
                    assert_eq!(out, 0.0);

                    t.update(&input, vec![1.0]);

                    let out: f64 = t.evaluate(&input);
                    assert_eq!(out, 1.0);
                }
            }
        }
    }
}
