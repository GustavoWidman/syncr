use linfa::DatasetBase;
use linfa::traits::{Fit, Predict};

use crate::data::entities::predictor::Model as PredictorSave;
use linfa_trees::{DecisionTree, SplitQuality};
use ndarray::{Array1, Array2, ArrayBase, Ix1, Ix2, OwnedRepr};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;

// SUPER UTILS
use super::utils::{default_block_size, fit_into_power_of_two_u32, fit_into_power_of_two_u64};

#[derive(Serialize, Deserialize)]
pub struct BlockSizePredictor {
    model: Option<DecisionTree<f64, usize>>,

    #[serde(skip)]
    model_file: String,

    // training data
    // is stored in a 2d array of (file_size, block_size, final_compression_rate)
    // file_size is used as input
    // block_size is used as output
    // final_compression_rate is used as weight
    pub data: Vec<(u64, u32, f32)>,
}

impl BlockSizePredictor {
    pub fn initialize(model_file: String) -> Result<Self, anyhow::Error> {
        if Path::new(&model_file).exists() {
            // Load the predictor from the file
            let mut file = File::open(&model_file)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            let mut model: BlockSizePredictor = serde_json::from_str(&contents)?;

            // super edge case, should never happen realistically
            if model.model_file != model_file {
                model.model_file = model_file;
            }

            Ok(model)
        } else {
            // Create a new predictor with empty model and data
            Ok(BlockSizePredictor {
                model: None,
                model_file: model_file.to_string(),
                data: Vec::new(),
            })
        }
    }

    pub fn new() -> Self {
        Self {
            model: None,
            model_file: "".to_string(),
            data: Vec::new(),
        }
    }

    pub fn rescue(model_binary: Option<PredictorSave>) -> Result<Self, anyhow::Error> {
        match model_binary {
            Some(model_binary) => Ok(bincode::deserialize(&model_binary.save)?),
            None => Ok(Self::new()),
        }
    }

    pub fn predict(&mut self, file_size: u64) -> u32 {
        match &self.model {
            Some(m) => {
                let input = Array2::from_shape_vec((1, 1), vec![
                    fit_into_power_of_two_u64(file_size) as f64,
                ])
                .unwrap();

                let output = m.predict(&input);
                let fitted = fit_into_power_of_two_u32(output[0] as u32);

                println!("Predicted block size: {}, output {}", output[0], fitted);
                return fitted;
            }
            None => {
                let training_data = self.dedup_training_data();

                if training_data.len() > 3 {
                    // auto-train
                    self.train();

                    return self.predict(file_size);
                }

                println!("Model not trained yet. Returning default block size.");
                default_block_size(file_size)
            }
        }
    }

    pub fn tune(&mut self, file_size: u64, correct_block_size: u32, final_compression_rate: f32) {
        // Add the new data point
        self.data.push((
            fit_into_power_of_two_u64(file_size),
            fit_into_power_of_two_u32(correct_block_size),
            final_compression_rate,
        ));

        self.data = self.dedup_data();

        // Retrain the model
        self.train();
    }

    pub fn train(&mut self) {
        let training_data = self.dedup_training_data();

        if training_data.len() < 3 {
            println!("Not enough data to train the model.");
            return;
        }

        // Separate the training data into input and output arrays
        let inputs = training_data
            .iter()
            .map(|(file_size, _, _)| *file_size)
            .collect::<Vec<_>>();
        let outputs = training_data
            .iter()
            .map(|(_, block_size, _)| *block_size)
            .collect::<Vec<_>>();

        // let weights = training_data
        //     .iter()
        //     .map(|(_, _, final_compression_rate)| *final_compression_rate)
        //     .collect::<Vec<_>>();

        let output: ArrayBase<OwnedRepr<f64>, Ix1> =
            Array1::from_vec(outputs.iter().map(|&x| x as f64).collect());

        let input: ArrayBase<OwnedRepr<f64>, Ix2> = Array2::from_shape_vec(
            (inputs.len(), 1),
            inputs.iter().map(|&x| x as f64).collect(),
        )
        .unwrap();

        // let weight = Array1::from_vec(weights);

        let feature_names = vec![""];

        // let dataset = DatasetBase::new(input, output).with_weights(weight);
        // let dataset = DatasetBase::new(input, output); // without weights
        let dataset = DatasetBase::new(input, output)
            .map_targets(|x| *x as usize)
            .with_feature_names(feature_names);
        // .with_weights(weight);

        // println!("{:?}", dataset);
        // println!("{:?}", dataset.records);
        // println!("{:?}", dataset.targets);

        // Create and fit the linear regression model
        // let model = LinearRegression::default().fit(&dataset).unwrap();

        let model = DecisionTree::params()
            .split_quality(SplitQuality::Entropy)
            .min_weight_leaf(0.1)
            .min_weight_split(0.2)
            .min_impurity_decrease(1e-7)
            .max_depth(Some(1000))
            .fit(&dataset)
            .unwrap();

        self.model = Some(model);
    }

    pub fn save(&self) {
        let serialized = serde_json::to_string(&self).expect("Error serializing the model.");
        let mut file = File::options()
            .write(true)
            .create(true)
            .open(&self.model_file)
            .expect("Unable to open the model file for writing.");

        // set to start
        file.seek(std::io::SeekFrom::Start(0)).unwrap();

        // truncate or pad to future file size
        file.set_len(serialized.len() as u64).unwrap();

        // overwrite (since we've sought to start)
        file.write_all(serialized.as_bytes())
            .expect("Error writing the model to file.");
    }

    pub fn save_binary(&self) -> Result<Vec<u8>, anyhow::Error> {
        Ok(bincode::serialize(&self)?)
    }

    // allows the model to wonder a value upwards or downwards of the current most optimal value
    // the value is then tested to find it's compression rate and we can conclude if it is better or worse
    // if the model has wondered once upwards and once downwards and has not improved,
    // it is considered to be stuck and the most optimal value is considered to be found
    // for that specific file size
    pub fn wonder(&self, unfit_file_size: u64) -> (u32, bool) {
        let file_size = fit_into_power_of_two_u64(unfit_file_size);

        let training_data = self.dedup_training_data();
        let deduped_data = self.dedup_data();

        let mut wonder_up = true;
        let mut wonder_down = true;

        // find file_size in training data, if none is found we can assume
        // a default value from default_block_size as the current wonder value
        let currently_predicted_size = match training_data
            .iter()
            .find(|(file_size_, _, _)| *file_size_ == file_size)
        {
            Some((_, block_size, _)) => *block_size,
            None => return (default_block_size(file_size), true),
        };

        println!("Currently predicted size: {:?}", currently_predicted_size);

        let wonder_up_value = currently_predicted_size * 2;
        let wonder_down_value = std::cmp::max(currently_predicted_size / 2, 1);

        // attempts to find either wonder values in deduped data in an attempt
        // to see if we've already wandered there

        for (_, block_size, _) in deduped_data
            .iter()
            .filter(|(file_size_, _, _)| *file_size_ == file_size)
        {
            if *block_size == wonder_up_value {
                wonder_up = false;
            }

            if *block_size == wonder_down_value {
                wonder_down = false;
            }
        }

        return match (wonder_up, wonder_down) {
            (true, true) => {
                if rand::random() {
                    (wonder_up_value, true)
                } else {
                    (wonder_down_value, true)
                }
            }
            (true, false) => (wonder_up_value, true),
            (false, true) => (wonder_down_value, true),
            (false, false) => (currently_predicted_size, false),
        };
    }

    fn dedup_training_data(&self) -> Vec<(u64, u32, f32)> {
        let mut map: HashMap<u64, (u32, f32)> = HashMap::new();

        for &(file_size, block_size, compression_rate) in self.data.clone().iter() {
            map.entry(file_size)
                .and_modify(|entry| {
                    if compression_rate > entry.1 {
                        *entry = (block_size, compression_rate);
                    }
                })
                .or_insert((block_size, compression_rate));
        }

        let mut training_data = Vec::new();
        for (file_size, (block_size, compression_rate)) in map {
            training_data.push((file_size, block_size, compression_rate));
        }
        training_data
    }

    fn dedup_data(&self) -> Vec<(u64, u32, f32)> {
        let mut map: HashMap<(u64, u32), f32> = HashMap::new();

        for &(file_size, block_size, compression_rate) in self.data.clone().iter() {
            map.entry((file_size, block_size))
                .and_modify(|entry| {
                    if compression_rate > *entry {
                        *entry = compression_rate;
                    }
                })
                .or_insert(compression_rate);
        }

        let mut training_data = Vec::new();
        for ((file_size, block_size), compression_rate) in map {
            training_data.push((file_size, block_size, compression_rate));
        }
        training_data
    }
}
