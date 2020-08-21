//! A One-Vs-Rest decision tree using rustlearn.

use super::mlmodel::{Error, MachineLearningAlgorithm};
use rustlearn::{
    multiclass::OneVsRestWrapper,
    prelude::{Array, RowIterable},
    traits::SupervisedModel,
    trees::decision_tree::DecisionTree,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DecisionTreeClassifier {
    learner: OneVsRestWrapper<DecisionTree>,
}

impl DecisionTreeClassifier {
    pub fn new(learner: OneVsRestWrapper<DecisionTree>) -> Self {
        Self { learner }
    }
}

impl MachineLearningAlgorithm<Vec<f32>> for DecisionTreeClassifier {
    fn fit(&mut self, x: &Vec<Vec<f32>>, y: &Vec<f32>) -> Result<(), Error> {
        let result = self
            .learner
            .fit(&Array::from(x), &Array::from(y.to_owned()))
            .map_err(|msg| Error::FitError(msg.to_string()))?;
        Ok(result)
    }

    fn predict(&self, x: &Vec<Vec<f32>>) -> Result<Vec<Vec<f32>>, Error> {
        let result = self
            .learner
            .predict(&Array::from(x))
            .map_err(|msg| Error::PredictionError(msg.to_string()))?;

        Ok(array_to_2d_vec(&result))
    }
}

// FIXME: why can't we just use the from/into implementation? God this sucks
fn array_to_2d_vec(a: &Array) -> Vec<Vec<f32>> {
    let mut result = Vec::new();
    for row in a.iter_rows() {
        result.push(row.iter().collect());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::array_to_2d_vec;
    use rustlearn::prelude::Array;

    #[test]
    fn test_array_to_2d_vec() {
        let original: Vec<Vec<f32>> = vec![vec![0.0, 0.0], vec![1.0, 2.0]];
        let array = Array::from(&original);

        let recovered = array_to_2d_vec(&array);

        assert_eq!(recovered, original);
    }
}
