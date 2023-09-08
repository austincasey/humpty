use std::{cmp::min, path::{Path, PathBuf}, io::BufWriter};
use nalgebra::{DVector, Matrix, Dyn, Const};
use ndarray::Array;
use csv::Writer;
use polars::prelude::{CsvReader, PolarsResult, DataFrame, SerReader};
use rayon::prelude::IntoParallelIterator;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use rayon::iter::ParallelIterator;
use std::fs::File;
use std::io::Write;


use crate::models::{ModelAffine::AffineAdditive, ModelTanh::ModelTanh};

pub fn readcsv(path : &String) -> PolarsResult<DataFrame> { 

    let dataset = CsvReader::from_path(path)?
                                .has_header(true)
                                //.with_dtypes(Some(Arc::new( schema)) )
                                //.with_try_parse_dates(true)
                                .finish();
    dataset
}

pub fn load_data( input : &String,
    offset: Option<&i64>,
    limit: Option<&usize>,
    strides : Option<&usize> ) -> ( Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>> , Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>> )
    {
        let verbose = false; 
        let df = readcsv( &input ).expect("problem opening file ...");
        let offset_data : i64 = match offset{
            Some(skip) => skip.clone(),
            None => 0,
        };
        let limit_data = match limit{
            Some(x) => min( offset_data as usize + x ,df.height()),
            None => df.height(),
        };
        let strides_data = match strides{
            Some(x) => x.clone(),
            None => 1,
        };
        let df = df.slice( offset_data as i64, limit_data ); 
        // the data frame is now trimmed ..
        let N = df.shape().0;  // number of rows (samples)
        if verbose { println!(" df {:?}", df ); }
        // POLARS Data frame to NDARRAY
        let D = match df.column( "count" ) {
            Ok(CD) => CD.f64().unwrap().to_ndarray().unwrap(),
            Err(_) => df.get_columns().last().unwrap().f64().unwrap().to_ndarray().unwrap(),
        };
        let time = Array::linspace(0., (N-1) as f64, N);
        println!( "time domain {:?}", time );
        println!( "data domain {:?}", D ); 
        println!(" .. ** .. ** .. ** .. " );

        let tspan: Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>> = DVector::from_vec( time.to_vec() );
        let dspan = DVector::from_vec( D.to_vec());
        (tspan, dspan )
}

pub fn model_curve_fitting( input : String, 
                            output : String, 
                            humps : usize, 
                            samples : usize , 
                            reports : usize,
                            offset: Option<&i64>,
                            limit: Option<&usize>,
                            strides : Option<&usize>
                        ){

    let ( tspan , dspan ) = load_data(&input,offset,limit,strides );
   
    let mut parlist = (0..samples).into_par_iter().map(
        |k|
        {
            let mut m2 = AffineAdditive::<ModelTanh>::random_model_given_humps( humps, &mut rand::thread_rng() );  
            let m2init = m2.clone();
            m2.curve_fit(&tspan, &dspan);
            let ( rsumsq,rsumsq_pp,resid ) = m2.residual(&tspan, &dspan); 
            (rsumsq, m2, m2init)
        });
    let mut list  : Vec<(f64, AffineAdditive<ModelTanh>, AffineAdditive<ModelTanh>)> = parlist.collect();

    list.sort_by(
        |a, b| 
        a.0.partial_cmp(&b.0).unwrap()
    );

    (0..reports).for_each(
        |k|
        {
            println!( "{:#?}", list[k] );
        }
    );

    // Create a file
    let mut path = PathBuf::from(output.as_str() );
    path.set_extension("yml");
 

    let mut data_file = File::create(path).expect("creation failed");
    let mut writer = BufWriter::new(data_file);
    serde_yaml::to_writer(&mut writer, &list[0..reports]).expect("serde yaml serialization fails.");
    writer.flush().expect("error finalizing serde yaml buffer.");

 
}


