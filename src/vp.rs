extern crate hashbrown;
extern crate rand;

use std::ops::*;
use std::hash::Hash;
use std::rc::Rc;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use hashbrown::HashMap;
use rand::prelude::*;

#[derive(Clone)]
pub struct Embedding<F>(pub Vec<(F, f32)>);

impl <F: Ord> Embedding<F> {
    pub fn new(mut feats: Vec<(F, f32)>) -> Self {
        feats.sort_by(|l, r| (&l.0).partial_cmp(&r.0).unwrap());
        Embedding(feats)
    }
}

impl <F> Embedding<F> {
    pub fn zero() -> Self {
        Embedding(vec![])
    }
}

#[derive(Clone,Copy)]
pub enum Regularizer {
    L1,
    L2
}

pub struct VecProp {
    pub n_iters: usize,
    pub regularizer: Regularizer,
    pub error: f32,
    pub max_terms: usize,
    pub alpha: f32,
    pub seed: u64
}

impl VecProp {

    pub fn fit<K: Hash + Eq + Clone, F: std::fmt::Debug + Hash + Eq + Clone + Ord>(
        &self, 
        graph: impl Iterator<Item=(K,K,f32)>, 
        prior: &HashMap<K,Embedding<F>>
    ) -> HashMap<K, Embedding<F>> {

        // Create graph
        let mut edges = HashMap::new();
        for (f_node, t_node, weight) in graph.into_iter() {
            let e = edges.entry(f_node.clone()).or_insert_with(|| vec![]);
            e.push((t_node.clone(), weight));
            let e = edges.entry(t_node).or_insert_with(|| vec![]);
            e.push((f_node, weight));
        }

        let mut keys: Vec<_> = edges.keys().map(|k| k.clone()).collect();
        let mut verts: HashMap<_,_> = keys.iter()
            .map(|key| {
                let e = if let Some(emb) = prior.get(key) {
                    emb.clone()
                } else {
                    Embedding::zero()
                };
                (key.clone(), e)
            })
            .collect();

        let mut rng = rand::rngs::StdRng::seed_from_u64(self.seed);
        for n_iter in 0..self.n_iters {
            eprintln!("Iteration: {}", n_iter);
            keys.shuffle(&mut rng);
            for key in keys.iter() {
                let mut features = HashMap::new();
                for (t_node, wi) in &edges[key] {

                    // Compute weighted sum of inbound
                    for (f, v) in verts[&t_node].0.iter() {
                        if let Some(nv) = features.get_mut(f) {
                            *nv += v * wi;
                        } else {
                            features.insert(f.clone(), v * wi);
                        }
                    }

                }

                // Check for the prior
                if let Some(p) = prior.get(key) {

                    // Scale the data by alpha
                    features.values_mut().for_each(|v| {
                        *v *= self.alpha;
                    });

                    // add the prior
                    for (k, v) in (p.0).iter() {
                        let nv = (1. - self.alpha) * (*v);
                        if features.contains_key(k) {
                            if let Some(v) = features.get_mut(k) {
                                *v += nv;
                            }
                        } else {
                            features.insert(k.clone(), nv);
                        }
                    }
                }

                // Normalize
                match self.regularizer {
                    Regularizer::L1 => {
                        let sum: f32 = features.values().map(|v| (*v).abs()).sum();
                        features.values_mut().for_each(|v| *v /= sum);
                    },
                    Regularizer::L2 => {
                        let sum: f32 = features.values().map(|v| (*v).powi(2)).sum();
                        features.values_mut().for_each(|v| *v /= sum.powf(0.5));
                    }
                }

                // Clean up data
                let mut features: Vec<_> = features.into_iter()
                    .filter(|(k, v)| v.abs() > self.error)
                    .collect();

                if features.len() > self.max_terms {
                    features.sort_by(|a,b| (b.1).partial_cmp(&a.1).unwrap());
                    while features.len() > self.max_terms {
                        features.pop();
                    }
                }

                if let Some(e) = verts.get_mut(key) {
                    *e = Embedding::new(features);
                }
            }
        }
        verts
    }
}

pub fn load_priors(path: &str) -> HashMap<u32,Embedding<Rc<String>>> {
    let f = File::open(path).expect("Error opening priors file");
    let br = BufReader::new(f);

    let mut prior = HashMap::new();
    let mut vocab = HashMap::new();

    for line in br.lines() {
        let line = line
            .expect("Failed to read line!");

        if let Some(idx) = line.find(" ") {
            let id: u32 = line.as_str()[0..idx].parse().unwrap();

            let mut features = HashMap::new();
            for token in line.as_str()[idx..].split_whitespace() {
                if !vocab.contains_key(token) {
                    vocab.insert(token.to_string(), Rc::new(token.to_string()));
                }

                features.insert(vocab[token].clone(), 1f32);
            }
            let size = features.len() as f32;
            let feats = features.into_iter()
                .map(|(k, v)| (k, v / size)).collect();
            prior.insert(id, Embedding::new(feats));
        }

    }
    prior
}
