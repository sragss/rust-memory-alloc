use std::hint::black_box;

use rayon::prelude::*;
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
    tracing_subscriber::registry().with(chrome_layer).init();
    println!("Running tracing-chrome. Files will be saved as trace-<some timestamp>.json and can be viewed in chrome://tracing.");
    run_normal();
    tracing::info!("Bench Complete");
    drop(_guard);
}

fn run_normal() {
    let batch_size: usize = 40;
    let height: usize = 24;
    let num_leaves: usize = 1 << height;

    let circuit_size = 2 * num_leaves * 256;
    let total_circuit_sizes = circuit_size * batch_size;
    let circuit_size_mb = circuit_size as f64 / 8.0 / 1024.0 / 1024.0;
    let total_circuit_sizes_gb = total_circuit_sizes as f64 / 8.0 / 1024.0 / 1024.0 / 1024.0;
    println!("Circuit size: {:.2} MB", circuit_size_mb);
    println!("Total circuit sizes: {:.2} GB", total_circuit_sizes_gb);


    let results: Vec<GrandProductCircuit> = (0..batch_size).into_par_iter().map(|batch_i| {
        GrandProductCircuit::new(height, num_leaves)
    }).collect();

    let xors: Vec<BigNumber> = results.iter().map(|circuit| circuit.xor_layers()).collect();
}

#[derive(Clone, PartialEq)]
struct BigNumber {
    number: [u64; 4]
}

impl BigNumber {
    fn new(number: [u64; 4]) -> BigNumber {
        BigNumber { number }
    }

    fn one() -> BigNumber {
        BigNumber { number: [1, 1, 1, 1] }
    }

    fn random() -> BigNumber {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        BigNumber { number: [rng.gen(), rng.gen(), rng.gen(), rng.gen()] }
    }
}


struct GrandProductCircuit {
    layers: Vec<Vec<BigNumber>>
}

impl GrandProductCircuit {
    #[tracing::instrument]
    fn new(height: usize, size: usize) -> GrandProductCircuit {
        assert_eq!(height, size.trailing_zeros() as usize);
        let mut layers = Vec::new();
        let mut size = size;
        for _ in 0..height {
            layers.push(vec![BigNumber::one(); size]);
            size /= 2;
        }
        GrandProductCircuit { layers }
    }

    fn xor_layers(&self) -> BigNumber {
        let mut xor_result = BigNumber::one();
        for layer in &self.layers {
            for number in layer {
                xor_result.number.iter_mut().zip(number.number.iter()).for_each(|(a, b)| *a ^= *b);
            }
        }
        xor_result
    }

}

