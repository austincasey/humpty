use std::{path::PathBuf, fs::File, io::BufReader, cmp::min};

use crate::models::{ModelAffine::AffineAdditive, ModelTanh::ModelTanh};
use crate::models::ParameterizedModel;
use crate::viz_lib::invert_tanh;
use crate::main_mod_dm_fit::{reload_data, load_data, data_fit_load_metadata, data_fit_package, model_fit};


pub fn basic_explanation(models: String, top: usize) {

    let mut path = PathBuf::from(models.as_str() );
    path.set_extension("yml"); 
    let mut data_file = File::open(path).expect("open model(s) file failed, please insure the extention is yml");
    let mut reader = BufReader::new(data_file);
    let VX : data_fit_package<ModelTanh> = serde_yaml::from_reader(reader).expect("problem reading yaml file");
    let data_fit_package{ load_metadata: lmd,  fits: VV } = VX;
    //let V:  Vec<(f64, AffineAdditive<ModelTanh>, AffineAdditive<ModelTanh>)>  

    (0..min(top, VV.len())).for_each(
        |k|
        { 
            let model_fit{ humps : humps,fitted_model : fitted_model, initial_model : initial_model, residual_total : residual_total,residual_per_point : residual_per_point} = &VV[k] ;
            let lsm = residual_total; 
            let M = &fitted_model; 
            println!("rank.{k}, lsm.{lsm}, {:?}", M); 
        }
    );

}

pub fn intermediate_explanation(models: String, top: usize, pval: f64) {
    let mut path = PathBuf::from(models.as_str() );

    path.set_extension("yml"); 
    let mut data_file = File::open(path).expect("open model(s) file failed, please insure the extention is yml");
    let mut reader = BufReader::new(data_file);
    let VX : data_fit_package<ModelTanh> = serde_yaml::from_reader(reader).expect("problem reading yaml file");
    let data_fit_package{ load_metadata: lmd,  fits: VV } = VX;
    //let V:  Vec<(f64, AffineAdditive<ModelTanh>, AffineAdditive<ModelTanh>)>  

    let mut mathematica_code = String::new();
    let mut matlab_code = String::new(); // TODO
    let mut julia_code = String::new();  // TODO 

    (0..min(top, VV.len())).for_each(
        |k|
        { 

            let model_fit{ humps : humps,fitted_model : fitted_model, initial_model : initial_model, residual_total : residual_total,residual_per_point : residual_per_point} = &VV[k] ;

            let lsm = residual_total; 
            let M = &fitted_model; 
            println!("rank.{k}, lsm.{lsm}:");
            let mut SortComp: Vec<ModelTanh> = M.tm.components.iter().map( |x| {x.clone()}).collect();
            SortComp.sort_by( | c , d |{ (-c.beta/c.alpha ).partial_cmp( &(-d.beta/d.alpha) ).unwrap()} );
            SortComp.iter().enumerate().for_each(
                |(j, c )|
                {
                    println!("\thump {j}\n\t\tκ = {},\n\t\tα = {},\n\t\tβ = {}", c.kappa, c.alpha, c.beta );
                    let mut MQ: Vec<f64> = vec![invert_tanh( 0.25, c.alpha , c.beta ), invert_tanh( 0.5, c.alpha , c.beta ), invert_tanh( 0.75, c.alpha , c.beta ) ];
                    MQ.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    println!("\t\tmotion-quantiles: {} {} {}", MQ[0], MQ[1], MQ[2]); 
                    let t_0 = -c.beta/c.alpha;
                    let X_0 = c.kappa;
                    let r = 2.*c.alpha;
                    let K = 2.*c.kappa;
                    println!("\t\tlogistic (alt parameters):\n\t\t\tt_0 = {},\n\t\t\tX_0 = {},\n\t\t\tr = {},\n\t\t\tK = {}", t_0, X_0, r, K );
                    mathematica_code.push_str( format!( "\ng{k}x{j} = κ ( 1 + Tanh[α t + β ] ) /. {{ κ -> {}, α -> {}, β -> {} }} (* hump {k}x{j} *)", c.kappa, c.alpha, c.beta  ).as_str());
                    matlab_code.push_str( format!( "\nfunction V = hump{k}x{j}( t )\n\tV= {} *( 1. + tanh({}*t + {} ) );\nend", c.kappa, c.alpha, c.beta ).as_str() );
                }
            );
            let constant_val = M.km.eval( 0. ); 
            println!( "\toffset: K = {constant_val}");

            mathematica_code.push_str( format!( "\ng{k}x{} = K /. {{ K-> {} }} (* constant offset *)", M.tm.components.len(), constant_val  ).as_str());
            matlab_code.push_str( format!( "\nfunction V = hump{k}x{}(t)\n\tV = {} + 0.*t \nend", M.tm.components.len(), constant_val ).as_str() ); 

            mathematica_code.push_str( format!( "\ntstart = -10;\ntend=200;\nPlot[{{ {} }}, {{t, tstart, tend}}]" , (0..(M.tm.components.len() + 1)).map( |x| String::from( format!( "g{}x{}", k, x ))).collect::<Vec<String>>().join( ", " )  ).as_str());
            matlab_code.push_str( format!( "\ntstart = -10;\ntend=200;\nTD=tstart:1.0:tend;\nplot(TD, [ {} ]')" , (0..(M.tm.components.len() + 1)).map( |x| String::from( format!( "hump{}x{}(TD)", k, x ))).collect::<Vec<String>>().join( "; " )  ).as_str());
           
            mathematica_code.push_str( format!( "\nPlot[ {} , {{t, tstart, tend}}]" , (0..(M.tm.components.len() + 1)).map( |x| String::from( format!( "g{}x{}", k, x ))).collect::<Vec<String>>().join( " + " )  ).as_str());
            matlab_code.push_str( format!( "\nplot( TD, {} ) " , (0..(M.tm.components.len() + 1)).map( |x| String::from( format!( "hump{}x{}(TD)", k, x ))).collect::<Vec<String>>().join( " + " )  ).as_str());
 
            println!( "(* Mathematica Code *)\n{}", mathematica_code );
            println!( "// matlab or octave code\n{}", matlab_code );
         
        }
    );
}

