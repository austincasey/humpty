
use std::error::Error;

use serde::*; 
use rand::Rng;
use rand::{seq::IteratorRandom, thread_rng}; 
use csv::Writer;
use rand_distr::{Normal, Distribution};
use crate::models::{ParameterizedModel, ModelAdditive::ModelAdditive, ModelConstant::ModelConstant, ModelTanh::ModelTanh};


#[derive(serde::Serialize)]
pub struct Row {
    time: f64, 
    count: f64
} 
pub fn write_data(path: &str, t : &Vec<f64>, d : &Vec<f64> )-> Result<(), Box<dyn Error>> {   
    let mut wtr = Writer::from_path(path)?;
    for ( x,y ) in t.iter().zip( d ){
        wtr.serialize( 
            Row { time : *x , count : *y }
        )?; 
    }
    wtr.flush()?;
    Ok(()) 
}
fn eval_tahn_model( M: &Vec<ModelTanh>, t: &Vec<f64> ) -> Vec<f64>{
    let mut rv : Vec<f64> = Vec::new(); 
    let MA: ModelAdditive<ModelTanh> = ModelAdditive::new( M.clone() );
    let YC: ModelConstant = ModelConstant::new(0.0); 
 
    for tx in t {
        rv.push( MA.eval(*tx ) + YC.eval(*tx) );
    }
    rv
}

//////////////////////////////////////////////////////////////////////////////////////////
/// Main Function Stuff
/// 


/// humpty runs estimates a multi-causal model from data.
/// this module generates data for fitting (tests and benchmarking) 
/// 
/// (C) W. Casey

pub fn generate_tanh_model( time_steps : usize, humps: usize, swing: (f64, f64 ) , error_mod : f64 , rpos : bool, verbose : bool) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<ModelTanh>){
    let mut t : Vec<f64> = (0..).take( time_steps).map( |x| x as f64 ).collect(); 
    let mut rng = rand::thread_rng();
    let (m,M) = swing;  
    let sample : Vec<(f64, f64, f64 )> = (0..).take( humps ).map( 
        |x| (   
            m + rng.gen::<f64>() * (M-m),  // K
            rng.gen::<f64>() * (time_steps as f64)  , // t_0
            (rng.gen::<f64>() - 0.5) * 4.0 // beta   
        )).collect();
    let mods : Vec<ModelTanh> = sample.iter().map( 
        |(K,alpha, beta)| 
        ModelTanh::new(*K, *alpha, if rpos {beta.abs()} else {*beta}) 
    ).collect();
    if verbose {
        println!( " sample: {:?}", sample );
        println!( " mods : {:?}", mods );
    }
    let d : Vec<f64> = eval_tahn_model(&mods, &t);
    let e : Vec<f64> = (0..).take( time_steps).map( |x| rng.gen::<f64>()* error_mod ).collect();
    let E = d.iter().zip(e).map( |(a,b)| a + b ).collect(); 
    (t, d, E ,mods)
}

pub fn build_tanh_model_from_string( time_steps : usize, des : String, error_mod : f64, verbose: bool ) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<ModelTanh>){
    let mut rng = rand::thread_rng();
    let x = des.split(",").into_iter().map( |x| x.parse::<f64>().unwrap()).collect::<Vec<f64>>();
    let mods : Vec<ModelTanh> = (0..x.len()/3).map( 
        |i| { 
            let K = x[3*i]; 
            let alpha  = x[3*i + 1];
            let beta = x[3*i + 2]; 
            ModelTanh::new(K, alpha, beta)  
        }).collect();
    if verbose {
        println!( " mods : {:?}", mods );
    }
    let mut t : Vec<f64> = (0..).take( time_steps).map( |x| x as f64 ).collect();
    let d : Vec<f64> = eval_tahn_model(&mods, &t);
 
    let normal = Normal::new(0., error_mod ).unwrap();
    let e : Vec<f64> = normal.sample_iter(&mut rand::thread_rng()).take(time_steps).collect();
    let E = d.iter().zip(e).map( |(a,b)| a + b ).collect(); 
    (t,d, E,mods )
}

