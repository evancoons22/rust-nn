use crate::dataloader::DataLoader;
use crate::linalg::*;
use crate::activation::*;
use crate::loss::*;

#[derive(Debug, PartialEq)]
pub struct Layer {
    pub input_size: usize,
    pub output_size: usize,
    pub weights: Matrix,
    pub biases: Vec<f64>,
    pub activation: Activation,
    pub activationdata: Vec<f64>,
    pub activationgrad: Vec<f64>,
}

impl Clone for Layer {
    fn clone(&self) -> Self {
        Layer { 
            input_size: self.input_size,
            output_size: self.output_size,
            weights: self.weights.clone(),
            biases: self.biases.clone(),
            activation: self.activation.clone(),
            activationdata: self.activationdata.clone(),
            activationgrad: self.activationgrad.clone(),
        }
    }
}

impl Layer { 
    pub fn forward(&mut self, inputs: &Vec<f64>) -> Vec<f64> {
        // applies network weights AND activates
        self.activationdata = self.activation.forward(&add(&(&self.weights * &inputs), &self.biases));
        self.activationdata.clone()
    }
    pub fn new(input_size: usize, output_size: usize, activation: Activation) -> Self {
        Layer {
            input_size,
            output_size,
            weights: Matrix::rand(output_size, input_size),
            biases: vec![0.0; output_size],
            activation,
            activationdata: vec![0.0; output_size],
            activationgrad: vec![0.0; output_size],
        }
    }

    pub fn weight_grad_backwards(&self, inputs: &Vec<f64>, agradnext: &Vec<f64>, loss: &LossFunction) -> Matrix {
        let mut result = Matrix::new(self.output_size, self.input_size);
        let activation = self.activationdata.clone();
        let actgrad = self.activation.backward(&inputs, loss);
        //eprintln!("input size: {:?}", &self.input_size);
        //eprintln!("output size: {:?}", &self.output_size);
        //eprintln!("inputs len: {:?}", &inputs.len()); // 2
        //eprintln!("actgrad len: {:?}", &actgrad.len()); // 4
        //eprintln!("activation len: {:?}", &activation.len()); // 2
        //eprintln!("agradnext len: {:?}", &agradnext.len()); // 2
        for j in 0..self.output_size {
            for k in 0..self.input_size {
                result.data[j][k] = actgrad[k] * activation[j] * agradnext[j];
            }
        }
        result
    }

    pub fn bias_grad_backwards(&self, inputs: &Vec<f64>, agradnext: &Vec<f64>, loss: &LossFunction) -> Vec<f64> {
        let mut result = vec![0.0; self.output_size];
        let actgrad = self.activation.backward(&inputs, loss);
        for j in 0..self.output_size {
            result[j] = actgrad[j] * agradnext[j];
        }
        result
    }


    pub fn activation_grad(&self, weights: Matrix, agradnext: &Vec<f64>, loss: &LossFunction) -> Vec<f64> {
        use crate::linalg::*;
        let mut result = vec![0.0; self.input_size];
        //let actgrad = self.activation.backward(&inputs, &LossFunction::MSE);
        let actgrad = self.activation.backward(&self.activationdata, loss);
        let tweights = transpose(weights.clone());
        let first = dot_product(&agradnext, &actgrad);
        //eprintln!("actgrad len: {:?}", &actgrad.len());
        //eprintln!("actgradnext len: {:?}", &agradnext.len());
        //eprintln!("tweights ncols: {:?}", &tweights.ncols);
        //eprintln!("length of result: {:?}", &result.len());

        for k in 0..self.input_size {
            //result[k] += dot_product(&[dot_product(&tweights.data[k], &actgrad)], &agradnext);
            //result[k] += dot_product(&tweights.data[k], &add(&actgrad, &agradnext));
            //result[k] = dot_product(&tweights.data[k], &first);
            
            //result[k] = tweights[k] * first;
            //iterate over tweights[k] and multiply each by first, then sum
            //
            result[k] = tweights.data[k].iter().map(|&x| x * first).sum();
        }
        result
    }
}

#[derive(Debug, PartialEq)]
pub struct Network {
    pub layers: Vec<Layer>,
    pub loss: LossFunction,
}

impl Network { 
    pub fn new() -> Self {
        Network {
            layers: Vec::new(),
            loss: LossFunction::MSE,
        }
    }

    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    pub fn add_layers(&mut self, layers: Vec<Layer>) {
        for layer in layers {
            self.layers.push(layer);
        }
    }

    pub fn forward(&mut self, inputs: &Vec<f64>) -> Vec<f64> {
        let mut outputs = inputs.clone();
        for layer in &mut self.layers {
            outputs = layer.forward(&outputs);
        }
        outputs
    }


    pub fn classify(&self, outputs: &Vec<f64>) -> Vec<f64> {
        outputs.iter().map(|x| if *x > 0.5 { 1.0 } else { 0.0 }).collect()
    }

    pub fn backward(&mut self, inputs: &Vec<f64>, y_true: &Vec<f64>, alpha: f64) {
        // the last layer is special... calculate the gradients for the last activations
        let mut activationgrad: Vec<f64> = vec![];
        let index: usize = self.layers.len() - 1;

        let outputs = self.layers[index].activationdata.clone();

        activationgrad.append(&mut self.loss.backward(&outputs, &y_true));


        // set the activation of the last layer equal to activationgrad
        self.layers[index].activationgrad = activationgrad.clone();

        // go through the layers backwards

        // why isn't this loop reached?
        for i in (0..self.layers.len()).rev() {
            let layer = &self.layers[i];

            let agradnext = if i == self.layers.len() - 1 {
                activationgrad.clone()
            } else {
                self.layers[i + 1].activationgrad.clone()
            };

            let input = match i {
                0 => inputs.clone(),
                _ => self.layers[i - 1].activationdata.clone(),
            };


            let weightgrad = layer.weight_grad_backwards(&input, &agradnext, &self.loss);
            //update the activation gradient that is a trait of the layer
            //
            // ACTIVATION GRAD SHOULD NOT TAKE INPUTS
            self.layers[i].activationgrad = layer.activation_grad(layer.weights.clone(), &agradnext, &self.loss);
            // update weights and biases
            //eprintln!("weightgrad: {:?}", &weightgrad.clone());
            self.layers[i].weights = self.layers[i].weights.clone() - (alpha * weightgrad);
            //self.layers[i].biases = subtract(&self.layers[i].biases, &biasgrad);
        }

        }

    pub fn train(&mut self, dataloader: &DataLoader, alpha: f64, epochs: usize, verbose: bool) {
        for _ in 0..epochs {
            self.forward(&dataloader.data[0]);
            self.backward(&dataloader.data[0], &dataloader.labels[0], alpha);
            if verbose {
                eprintln!("loss: {:?}", self.loss.getloss(&self.layers[self.layers.len() - 1].activationdata, &dataloader.labels[0]));
            }

        }
    }

    pub fn save_weights(&self, filename: &str) {
        use std::fs::File;
        use std::io::Write;
        let mut file = File::create(filename).expect("Unable to create file");
        for layer in &self.layers {
            for row in &layer.weights.data {
                for col in row {
                    file.write_all(format!("{},", col).as_bytes()).expect("Unable to write data");
                }
            }
            file.write_all(b"\n").expect("Unable to write data");
        }
    }

    pub fn load_weights(&mut self, filename: &str) {
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        let file = File::open(filename).expect("Unable to open file");
        let reader = BufReader::new(file);
        let mut weights: Vec<Vec<f64>> = Vec::new();
        for line in reader.lines() {
            let line = line.expect("Unable to read line");
            let mut row: Vec<f64> = Vec::new();
            //remove the last comma from line
            let line = line[..line.len() - 1].to_string();
            for col in line.split(",") {
                row.push(col.parse::<f64>().expect("Unable to parse float"));
            }
            weights.push(row);
        }
        let mut index = 0;
        for layer in &mut self.layers {

            for i in 0..layer.weights.nrows {
                for j in 0..layer.weights.ncols {
                    layer.weights.data[i][j] = weights[index][i * layer.weights.ncols + j];
                }
            }
            index += 1;
        }

    }

}


#[cfg(test)]
mod tests {
    use crate::linalg::*;
    use crate::nn::{Layer, Network};

    #[test]
    fn test_layer_forward() { 
        use crate::activation::Activation;
        let mut layer = Layer { 
            input_size: 2,
            output_size: 2,
            weights: Matrix { 
                nrows: 2,
                ncols: 2,
                data: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            },
            biases: vec![0.0, 0.0],
            activation: Activation::None,
            activationdata: vec![0.0, 0.0],
            activationgrad: vec![0.0, 0.0],
        };

        let inputs = vec![1.0, 2.0];
        let outputs = vec![5.0, 11.0];
        let layer_outputs = layer.forward(&inputs);
        assert_eq!(layer_outputs.clone(), outputs.clone());
        let layer2 = layer.forward(&layer_outputs);
        assert_eq!(layer2, vec![27.0, 59.0]);
    }

    #[test]
    fn network_forward_twice() { 
        use crate::activation::Activation;
        use crate::loss::LossFunction;
        let layer = Layer { 
            input_size: 2,
            output_size: 2,
            weights: Matrix { 
                nrows: 2,
                ncols: 2,
                data: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
            },
            biases: vec![0.0, 0.0],
            activation: Activation::None,
            activationdata: vec![0.0, 0.0],
            activationgrad: vec![0.0, 0.0],
        };

        let inputs = vec![1.0, 2.0];
        let mut network = Network { layers: vec![layer.clone(), layer.clone()], loss: LossFunction::MSE};
        assert_eq!(network.forward(&inputs), vec![27.0, 59.0]);

        }

    #[test]
    fn layer_forward_new() { 
        use crate::activation::Activation;
        let mut layer = Layer::new(2, 2, Activation::None);
        layer.weights = Matrix { 
            nrows: 2,
            ncols: 2,
            data: vec![vec![1.0, 2.0], vec![3.0, 4.0]],
        };
        // add 1 layer to the network
        let mut network = Network::new();
        network.add_layer(layer.clone());
        let inputs = vec![1.0, 2.0];
        let outputs = layer.forward(&inputs);
        assert_eq!(outputs.len(), 2);
    }


    #[test]
    fn class_network_test() {
        use crate::nn::Network;
        use crate::nn::Layer;
        use crate::activation::Activation;
        use crate::loss::*;
        use crate::dataloader::DataLoader;

        let mut network = Network::new();
        network.loss = LossFunction::CrossEntropy;

        network.add_layers(vec![
            Layer::new(2, 4, Activation::Relu),
            Layer::new(4, 2, Activation::Relu),
            Layer::new(2, 1, Activation::Sigmoid),
        ]);

        let dataloader = DataLoader::new(vec![
                                             vec![1.0, 1.0],
                                             vec![0.0, 0.0],
                                             vec![0.0, 1.0],
                                             vec![1.0, 0.0],],
                                        vec![
                                            vec![1.0],
                                            vec![1.0],
                                            vec![0.0],
                                            vec![0.0],], 1, false);



        //let initial_loss = network.forward(&dataloader.data[0]);
        let initial_loss = network.loss.getloss(&network.layers[network.layers.len() - 1].activationdata, &dataloader.labels[0]);

        network.train(&dataloader, 0.006, 100, false);

        let final_loss = network.loss.getloss(&network.layers[network.layers.len() - 1].activationdata, &dataloader.labels[0]);

        //eprintln!("initial network output: {:?}\n, final network output: {:?}", initial, last);
        eprintln!("initial loss: {:?}\n, final loss: {:?}", initial_loss, final_loss);
        //eprintln!("true output: {:?}\n", dataloader.labels[0]);

        //asser that initial loss is greater than final loss
        assert!(initial_loss > final_loss);

    }

}

