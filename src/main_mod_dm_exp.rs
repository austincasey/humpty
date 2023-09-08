use std::{path::PathBuf, fs::File, io::BufReader, cmp::min};

use crate::models::{ModelAffine::AffineAdditive, ModelTanh::ModelTanh};
use crate::models::ParameterizedModel;
use crate::viz_lib::invert_tanh;

pub fn basic_explanation(models: String, top: usize) {

    let mut path = PathBuf::from(models.as_str() );
    path.set_extension("yml"); 
    let mut data_file = File::open(path).expect("open model(s) file failed, please insure the extention is yml");
    let mut reader = BufReader::new(data_file);
    let V:  Vec<(f64, AffineAdditive<ModelTanh>, AffineAdditive<ModelTanh>)>  = serde_yaml::from_reader(reader).expect("problem reading yaml file");
    

    (0..min(top, V.len())).for_each(
        |k|
        { 
            let lsm = V[k].0; 
            let M = &V[k].1; 
            println!("rank.{k}, lsm.{lsm}, {:?}", M); 
        }
    );

}

pub fn intermediate_explanation(models: String, top: usize, pval: f64) {
    let mut path = PathBuf::from(models.as_str() );
    path.set_extension("yml"); 
    let mut data_file = File::open(path).expect("open model(s) file failed, please insure the extention is yml");
    let mut reader = BufReader::new(data_file);
    let V:  Vec<(f64, AffineAdditive<ModelTanh>, AffineAdditive<ModelTanh>)>  = serde_yaml::from_reader(reader).expect("problem reading yaml file");
    

    let mut mathematica_code = String::new();
    let mut matlab_code = String::new(); // TODO
    let mut julia_code = String::new();  // TODO 

    (0..min(top, V.len())).for_each(
        |k|
        { 
            let lsm = V[k].0; 
            let M = &V[k].1; 
            println!("rank.{k}, lsm.{lsm}:");
            M.tm.components.iter().enumerate().for_each(
                |(j, c )|
                {
                    println!("\thump {j}\n\t\tκ = {},\n\t\tα = {},\n\t\tβ = {}", c.kappa, c.alpha, c.beta );
                    let mut MQ: Vec<f64> = vec![invert_tanh( 0.25, c.alpha , c.beta ), invert_tanh( 0.5, c.alpha , c.beta ), invert_tanh( 0.75, c.alpha , c.beta ) ];
                    MQ.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    println!("\t\tmotion-quantiles: {} {} {}", MQ[0], MQ[1], MQ[2]); 
                    println!("\t\tlogistic (alt parameters):\n\t\t\tt_0 = {},\n\t\t\tX_0 = {},\n\t\t\tr = {},\n\t\t\tK = {}", -c.beta/c.alpha, c.kappa, 2.*c.alpha, 2.*c.kappa );
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

