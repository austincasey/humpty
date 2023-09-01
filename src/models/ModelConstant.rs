
use std::sync::Arc;

use num_traits::Float;
use serde::{Serialize, Deserialize};
use crate::models::ParameterizedModel; 

use rand_distr::{Normal, Distribution};
use nalgebra::{DVector, Scalar, geometry::Scale1};
 
////////////////
/// 
/// constant model
/// 
#[derive(Debug, Serialize, Deserialize, Clone, Copy )]
pub struct ModelConstant {
    k : f64, 
}
impl ModelConstant { 
    pub fn new(k: f64) -> Self { Self { k } }
}        

impl ParameterizedModel for ModelConstant{
    fn get_all_params(&self) -> Vec<f64>{
        return vec![ self.k ]
    }
    fn set_all_params(&mut self, V: &[f64]){
        self.k = V[0]
    }
    fn get_nonlinear_params( &self ) -> Vec<f64> {
        return vec![ ]
    }
    fn eval( &self, t : f64 ) -> f64{
        self.k 
    }

    fn get_copy( &self ) -> Self {
        Self::new( self.k )
    }
    fn mute( &mut self, rng : &rand::rngs::ThreadRng, mag : f64, var : Option<&Self> ){
        let normal = Normal::new(self.k, mag * match var { Some(v) => v.k, _=>1.0 }).unwrap();
        self.k = normal.sample(&mut rand::thread_rng());
    }
    fn random_model( rng : &rand::rngs::ThreadRng ) -> Self {
        let normal = Normal::new(0.0, 1.0 ).unwrap();
        Self::new( normal.sample( &mut rand::thread_rng() )) 
    } 

}

