use std::cmp::min;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::models::ModelAffine::AffineAdditive;
use crate::models::ModelTanh::ModelTanh;
use crate::viz_lib;
use crate::viz_lib::*;
use crate::main_mod_dm_fit::load_data;

pub fn basic_visualization(data: String, models: String, output: String, top: usize, 
        offset :Option<&i64>,  limit : Option<&usize>, strides : Option<&usize> ) 
{
    let ( tspan, dspan )= load_data(&data, offset, limit, strides);

    println!( " {}, {}, {}, {}", data, models, output, top );  
    println!( " {:?}\n{:?}\n", tspan, dspan );
    // read the model file. 
 
    let mut path = PathBuf::from(models.as_str() );
    path.set_extension("yml"); 
    let mut data_file = File::open(path).expect("open model(s) file failed, please insure the extention is yml");
    let mut reader = BufReader::new(data_file);
    let V:  Vec<(f64, AffineAdditive<ModelTanh>, AffineAdditive<ModelTanh>)>  = serde_yaml::from_reader(reader).expect("problem reading yaml file");

    let mut path_output = PathBuf::from( output.as_str());
    path_output.set_extension("");
    let output_stem = path_output.clone();
    path_output.set_extension("png");


    (0..min(top, V.len())).into_par_iter().for_each(
        |k|
        { 
            let mx = eval_M( &V[k].1 , &tspan.as_slice().to_vec());
            let lsm = V[k].0;
            viz_lib::plot_c(
                &tspan.as_slice().to_vec(), 
                &dspan.as_slice().to_vec(), 
                &mx, 
                PlotAction::PNG(format!( "{}_fit_{}", output_stem.to_str().unwrap(), k).into(), 800, 600, 2.0),
                String::from( format!( "model fit {} \n LSM {} ", k, lsm )), 
                String::from( "time "), 
                String::from( "quantity" ) 
            );
            println!("writing file {}", output_stem.to_str().unwrap())
        }
    );


}
 
pub fn intermediate_visualization(data: String, models: String, output: String, item: usize, pval : f64, 
    offset :Option<&i64>,  limit : Option<&usize>, strides : Option<&usize> ) 
{
let ( tspan, dspan )= load_data(&data, offset, limit, strides);

println!( " {}, {}, {}, {}", data, models, output, item );  
println!( " {:?}\n{:?}\n", tspan, dspan );
// read the model file. 

let mut path = PathBuf::from(models.as_str() );
path.set_extension("yml"); 
let mut data_file = File::open(path).expect("open model(s) file failed, please insure the extention is yml");
let mut reader = BufReader::new(data_file);
let V:  Vec<(f64, AffineAdditive<ModelTanh>, AffineAdditive<ModelTanh>)>  = serde_yaml::from_reader(reader).expect("problem reading yaml file");
println!( " wer eare here {:#?}", V );

let mut path_output = PathBuf::from( output.as_str());
path_output.set_extension("");
let output_stem = path_output.clone();
path_output.set_extension("png");


let k = item; 
    { 
        let selected_model = &V[k].1; 
        let mx = eval_M( &selected_model , &tspan.as_slice().to_vec());
        let lsm = V[k].0;
        viz_lib::plot_model_with_markers(
            &tspan.as_slice().to_vec(), 
            &dspan.as_slice().to_vec(), 
            selected_model.clone(), 
            0.20,
            PlotAction::PNG(format!( "{}_model_{}", output_stem.to_str().unwrap(), k).into(), 800, 600, 2.0),
            String::from( format!( "model fit {} \n LSM {} ", k, lsm )), 
            String::from( "time "), 
            String::from( "quantity" ) 
        );
        println!("writing file {}", output_stem.to_str().unwrap())
    }



}
