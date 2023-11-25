use std::{cmp::min, path::{Path, PathBuf}, io::BufWriter};
use nalgebra::{DVector, Matrix, Dyn, Const};
use ndarray::{Array, s};
use csv::Writer;
use polars::prelude::{CsvReader, PolarsResult, DataFrame, SerReader};
use rayon::prelude::IntoParallelIterator;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use rayon::iter::ParallelIterator;
use std::fs::File;
use std::io::Write;
use crate::models::{ModelAffine::AffineAdditive, ModelTanh::ModelTanh, ParameterizedModel, VarProAdapter};

use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone )]
pub struct data_slice{
    offset: Option<i64>,
    limit: Option<usize>,
    strides : Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone )]
pub struct data_fit_load_metadata{
    input : String,  //this should probably be a global path or URI
    slice : data_slice, 
    colname : String 
}

#[derive(Debug, Serialize, Deserialize, Clone )]
pub struct model_fit<M> where M: ParameterizedModel + Clone  + VarProAdapter {
    pub humps : usize,
    pub fitted_model : AffineAdditive<M>,
    pub initial_model : AffineAdditive<M>,
    pub residual_total : f64, 
    pub residual_per_point : f64,
} 

#[derive(Debug, Serialize, Deserialize, Clone )]
pub struct data_fit_package<M> where M: ParameterizedModel + Clone  + VarProAdapter  {
    pub load_metadata : data_fit_load_metadata,
    pub fits : Vec<model_fit<M>>
}
///////////////////
/// TODO here the data_fit_package can furnish several types of anlaysis.
/// * analysis of variance of changepoints.
///     Fixing the hump size, and running lots of samples calcualte the variance in Change Points as the a funciton of residual.
/// * regularized fit.
///     Finding the best of each hump index, and presented with a cost function (in hump index) calculate the best regularized model.
/// 

pub fn readcsv(path : &String) -> PolarsResult<DataFrame> { 

    let dataset = CsvReader::from_path(path)?
                                .has_header(true)
                                //.with_dtypes(Some(Arc::new( schema)) )
                                //.with_try_parse_dates(true)
                                .finish();
    dataset
}

pub fn reload_data( ds : data_fit_load_metadata ) -> (
    ( Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>> , Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>> ),
    Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>>,
    ( ndarray::ArrayBase<ndarray::OwnedRepr<f64>, ndarray::Dim<[usize; 1]>>, ndarray::ArrayBase<ndarray::OwnedRepr<f64>, ndarray::Dim<[usize; 1]>> )){
    let data_fit_load_metadata{ input, slice , colname }: data_fit_load_metadata= ds;
    let data_slice{ offset, limit, strides } = slice; 
    load_data( &input, offset, limit, strides, Some( &colname ))
}

pub fn load_data( input : &String,
    offset: Option<i64>,
    limit: Option<usize>,
    strides : Option<usize>,
    colname : Option<&String> ) -> (
        ( Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>> , Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>> ),
        Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>>,
        ( ndarray::ArrayBase<ndarray::OwnedRepr<f64>, ndarray::Dim<[usize; 1]>>, ndarray::ArrayBase<ndarray::OwnedRepr<f64>, ndarray::Dim<[usize; 1]>> ))
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
        //let df = df.slice( offset_data as i64, limit_data ); 
        // the data frame is now trimmed ..
        let data_column = match colname {
            Some(X) => X.clone(),
            None => String::from( "count" ),
        };
        if verbose { println!(" df {:?}", df ); }
        // POLARS Data frame to NDARRAY
        
        let DFULL: ndarray::ArrayBase<ndarray::ViewRepr<&f64>, ndarray::Dim<[usize; 1]>> = match df.column( &data_column ) {
            Ok(CD) => CD.f64().unwrap().to_ndarray().unwrap(),
            Err(_) => { 
                println!( "ERROR no column by name {data_column}, instead will use the last data column by default.");
                df.get_columns().last().unwrap().f64().unwrap().to_ndarray().unwrap()
            }
        };
        let NFULL = DFULL.shape()[0];
        let TFULL: ndarray::ArrayBase<ndarray::OwnedRepr<f64>, ndarray::Dim<[usize; 1]>>  = Array::linspace( 0., (NFULL-1) as f64, NFULL );


        let DSLICE = DFULL.slice(s![(offset_data as usize)..(limit_data); strides_data]);
        let NSLICE = DSLICE.shape()[0];
        let TSLICE = TFULL.slice( s![(offset_data as usize)..(limit_data); strides_data]);//Array::linspace(0., (N-1) as f64, N);

        //println!( "time domain {:?}", time );
        //println!( "data domain {:?}", DSLICE ); 


        let tsplice: Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>> = DVector::from_vec( TSLICE.to_vec() );
        let dslice = DVector::from_vec( DSLICE.to_vec());
        let tslice :  Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>> = DVector::from_vec( (0..NSLICE).map(|x|{x as f64}).collect() );
        (   (tslice, tsplice ), // tslice is indexing for dslice, tsplice is indexing in D. 
            dslice,               // spliced time (indexes of full time .. ( k...N+k) and sliced data), 
            (TFULL, DFULL.to_owned()))  // full time full data
}

pub fn model_curve_fitting( input : String, 
                            output : String, 
                            humps : usize, 
                            samples : usize , 
                            reports : usize,
                            data_column: &String,
                            offset: Option<i64>,
                            limit: Option<usize>,
                            strides : Option<usize>
                        ){
    let ds = data_slice{ offset: offset.clone(), limit: limit.clone(), strides: strides.clone()};
    let md: data_fit_load_metadata = data_fit_load_metadata{ input:input.clone(), slice:ds , colname: data_column.clone()};
    let (( tspan ,texact), dspan  , (tspanfull, dspanfull)) = load_data(&input,offset,limit,strides, Some( &data_column.clone()) );
    //let N = dspan.shape().0;
    let mut parlist = (0..samples).into_par_iter().map(
        |k|
        {
            let mut m2 = AffineAdditive::<ModelTanh>::random_model_given_humps( humps, &mut rand::thread_rng() );  
            let m2init = m2.clone();
            m2.curve_fit(&texact, &dspan);
            let ( rsumsq,rsumsq_pp,resid , resid1) = m2.residual_mat(&texact, &dspan); 
            (rsumsq, rsumsq_pp, m2, m2init)
        });
    let mut list  : Vec<(f64, f64, AffineAdditive<ModelTanh>, AffineAdditive<ModelTanh>)> = parlist.collect();

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

    let mfits : Vec<model_fit<ModelTanh>> = list.iter().take(reports).map(|(r, rpp, m, minit)|
    {
        model_fit {
            humps: humps,
            fitted_model: m.clone(),
            initial_model: minit.clone(),
            residual_total : *r, 
            residual_per_point : *rpp 
        } 
    }).collect();

    let model_pack = data_fit_package{ load_metadata: md, fits: mfits};
    // Create a file
    let mut path = PathBuf::from(output.as_str() );
    path.set_extension("yml");
 

    let mut data_file = File::create(path).expect("creation failed");
    let mut writer = BufWriter::new(data_file);

    //old method:
    // serde_yaml::to_writer(&mut writer, &list[0..reports]).expect("serde yaml serialization fails.");

    serde_yaml::to_writer(&mut writer, &model_pack).expect("serde yaml serialization fails.");
    writer.flush().expect("error finalizing serde yaml buffer.");

 
}


