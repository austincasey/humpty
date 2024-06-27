use std::cmp::min;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
//use std::marker::Tuple;
use std::path::PathBuf;
 
use csv::Writer;
use libm::sqrt;
use ndarray::s;
use ndarray::ShapeBuilder;
use polars::frame::row::Row;
use polars::prelude::RollingQuantileParams;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::main_mod_dm_fit::data_fit_package;
use crate::main_mod_dm_fit::model_fit;
use crate::main_mod_dm_fit::reload_data;
use crate::models::ModelAffine::AffineAdditive;
use crate::models::ModelTanh::ModelTanh;

use crate::main_mod_dm_viz::*;
use crate::viz_lib::eval_M;


/*
support the following columns.
residual      construct basic plots with rolling quantile for residual.
disp          construct basic plots with rolling quantile for displacments.
skew          construct basic plots with rolling quantile for skewness.
csv [forcast]           drop a csv file.
*/


#[derive(serde::Serialize)]
pub struct RowOut {
    time: f64,
    model: f64, 
    data: f64,
} 

pub fn serialize_csv_data(path: &str, t :&Vec<f64>, m :&Vec<f64>, d :&Vec<f64> ) -> Result<(), Box<dyn Error>>  {
    let mut wtr = Writer::from_path(path)?;
    for ( (tx,dx), mx ) in t.iter().zip( d ).zip( m ){
        wtr.serialize( 
            RowOut { time : *tx , model : *mx, data: *dx }
        )?; 
    }
    wtr.flush()?;
    Ok(()) 
}
#[derive(serde::Serialize)]
pub struct RowOut2 {
    time: f64,
    model: f64, 
} 
pub fn serialize_csv_data2(path: &str, t :&Vec<f64>, m :&Vec<f64> ) -> Result<(), Box<dyn Error>>  {
    let mut wtr = Writer::from_path(path)?;
    for (tx,mx) in t.iter().zip( m ){
        wtr.serialize( 
            RowOut2 { time : *tx , model : *mx }
        )?; 
    }
    wtr.flush()?;
    Ok(()) 
}

//drop_csv_data
pub fn drop_csv_data(
    models: String,
    output: String,
    item: usize,
    offset: Option<i64>,
    limit: Option<usize>,
    strides: Option<usize>,
    pmod: partial_model,
) {
    let mut path = PathBuf::from(models.as_str()); // get the model file.
    path.set_extension("yml");
    let mut data_file =
        File::open(path).expect("open model(s) file failed, please insure the extention is yml");
    let mut reader = BufReader::new(data_file);
    let VX: data_fit_package<ModelTanh> =
        serde_yaml::from_reader(reader).expect("problem reading yaml file");
    let data_fit_package {
        load_metadata: lmd,
        fits: VV,
    } = VX;
    let ((tspan, tsplice), dsplice, (tfull, dfull)) = reload_data(lmd);

    // read the model file.
    // step 4: calculate the clipping slice or viz slice
    let eoffset = offset.unwrap_or(0_i64) as usize;
    let elimit: usize = limit.unwrap_or(dfull.shape()[0]);
    let delimit : usize = limit.unwrap_or( dfull.shape()[0]).min( dfull.shape()[0]);
    let estrides = strides.unwrap_or(1);
    // All time/data in the viz slice
    let tall: Vec<f64> = {
        if elimit <= delimit {
         tfull
        .slice(s![eoffset..(eoffset+elimit); estrides])
        .to_vec()
    } else {
        (eoffset..(eoffset+elimit)).step_by(estrides).map(|x|{x as f64}).collect()
    }};
   
    let dall: Vec<f64> = dfull
        .slice(s![eoffset..(eoffset+delimit); estrides])
        .to_vec();
    if (estrides > 1) {
        panic!(" estrides Not yet supported ");
    }

    let mut path_output = PathBuf::from(output.as_str());
    path_output.set_extension("csv");
    let output_stem = path_output.clone();

    (item..(item + 1)).into_par_iter().for_each(|k| {
        let model_fit {
            humps: humps,
            fitted_model: fitted_model,
            initial_model: initial_model,
            residual_total: residual_total,
            residual_per_point: residual_per_point,
        } = &VV[k];

        let mxall: Vec<f64> = eval_M(&fitted_model, &tall);
        let PX = path_output.to_str().unwrap();
        if elimit <= delimit { 
            serialize_csv_data(PX, &tall, &mxall, &dall );
        }   else {
            serialize_csv_data2(PX, &tall, &mxall );
        }

        println!("writing file {}", PX);
    });
}


#[derive(serde::Serialize)]
pub struct RowOutResid {
    time: f64,
    model: f64, 
    data: f64,
    residual: f64,
    disp : f64,
    skew : f64
} 
pub fn serialize_csv_res_data(path: &str, t :&Vec<f64>, m :&Vec<f64>, d :&Vec<f64>, r: &Vec<f64>, rpp:&Vec<f64> , skew : &Vec<f64>) -> Result<(), Box<dyn Error>>  {
    let mut wtr = Writer::from_path(path)?;
    for (((((tx,mx), dx ) , rx), rppx), sx) in t.iter().zip( d ).zip( m ).zip( r ).zip(rpp).zip(skew){
        wtr.serialize( 
            RowOutResid { time : *tx , model : *mx, data: *dx, residual: *rx, disp: *rppx, skew: *sx }
        )?; 
    }
    wtr.flush()?;
    Ok(()) 
}

pub fn residual_disp_skew_csv(
    models: String,
    output: String,
    item: usize,
    offset: Option<i64>,
    limit: Option<usize>,
    strides: Option<usize>,
    pmod: partial_model,
) {
    let mut path = PathBuf::from(models.as_str()); // get the model file.
    path.set_extension("yml");
    let mut data_file =
        File::open(path).expect("open model(s) file failed, please insure the extention is yml");
    let mut reader = BufReader::new(data_file);
    let VX: data_fit_package<ModelTanh> =
        serde_yaml::from_reader(reader).expect("problem reading yaml file");
    let data_fit_package {
        load_metadata: lmd,
        fits: VV,
    } = VX;
    let ((tspan, tsplice), dsplice, (tfull, dfull)) = reload_data(lmd);

    // read the model file.
    // step 4: calculate the clipping slice or viz slice
    let eoffset = offset.unwrap_or(0_i64) as usize;
    let elimit: usize = limit.unwrap_or(dfull.shape()[0]).min( dfull.shape()[0]);
    let delimit : usize = limit.unwrap_or( dfull.shape()[0]).min( dfull.shape()[0]);
    let estrides = strides.unwrap_or(1);
    // All time/data in the viz slice
    let tall: Vec<f64> = {
        if elimit <= delimit {
         tfull
        .slice(s![eoffset..(eoffset+elimit); estrides])
        .to_vec()
        } else {
            println!( " elimit {elimit} and delimit {delimit}");
            panic!( "not supported ");
        
            (eoffset..(eoffset+elimit)).step_by(estrides).map(|x|{x as f64}).collect()
        }};
   
    let dall: Vec<f64> = dfull
        .slice(s![eoffset..(eoffset+delimit); estrides])
        .to_vec();
    if (estrides > 1) {
        panic!(" estrides Not yet supported ");
    }

    let mut path_output = PathBuf::from(output.as_str());
    path_output.set_extension("csv");
    let output_stem = path_output.clone();

    let ETS: Vec<(usize, &f64)> = tsplice
        .iter()
        .enumerate()
        .filter(|(k, g)| ((**g as usize) >= eoffset) && ((**g as usize) < (eoffset + elimit)))
        .collect();

    let pre = 0_usize; // no flanking regions
    let post = 0_usize; // no flanking regions
    (item..(item + 1)).into_par_iter().for_each(|k| {
        let model_fit {
            humps: humps,
            fitted_model: fitted_model,
            initial_model: initial_model,
            residual_total: residual_total,
            residual_per_point: residual_per_point,
        } = &VV[k];
    
        let (rsumsqxx, rsumsq_ppxx, residxx, resid1xx) = fitted_model.residual(&tall, &dall);
    
        let mxall: Vec<f64> = eval_M(&fitted_model, &tall);

        let res_viz: Vec<f64> = resid1xx.clone();
        ////
        /// m_3( m ) = sum_j^m( r[j] - <r[:m]> )^3 / ((m-1) [r[:m])
        let mu3: Vec<f64> = (1..)
            .take(res_viz.len() + 1)
            .map(|m| {
                let avg_resid: f64 = res_viz.iter().take(m).sum::<f64>() / (m as f64);
                let resid_disp: Vec<f64> = res_viz.iter().take(m).map(|x| *x - avg_resid).collect();
                let resid_disp_sq: Vec<f64> = resid_disp.iter().map(|x| (*x) * (*x)).collect();
                let resid_disp_cu: Vec<f64> =
                    resid_disp.iter().map(|x| (*x) * (*x) * (*x)).collect();
                let sv = sqrt(resid_disp_sq.iter().sum::<f64>() / (m as f64));
                let mut dnom: f64 = sv * sv * sv * ((m - 1) as f64);
                if dnom <= 0.0 {
                    dnom = 1.0;
                }
                let m_3: f64 = resid_disp_cu.iter().sum::<f64>() / (dnom);
                m_3
            })
            .collect();

        let PX = path_output.to_str().unwrap();
        if elimit <= delimit { 
            serialize_csv_res_data( PX, &tall, &mxall, &dall, &residxx, &resid1xx, &mu3);
        }   else {
            panic!("not supported ");
        }

        println!("writing file {}", PX);
    });

}

/* 
/////////////////////////////////
/// TODO : clean up this function its copied from residual_visualization but needs very little of the present code.
///
pub fn disp_visualization(
    models: String,
    output: String,
    item: usize,
    offset: Option<i64>,
    limit: Option<usize>,
    strides: Option<usize>,
    pmod: partial_model,
) {
    let mut path = PathBuf::from(models.as_str()); // get the model file.
    path.set_extension("yml");
    let mut data_file =
        File::open(path).expect("open model(s) file failed, please insure the extention is yml");
    let mut reader = BufReader::new(data_file);
    let VX: data_fit_package<ModelTanh> =
        serde_yaml::from_reader(reader).expect("problem reading yaml file");
    let data_fit_package {
        load_metadata: lmd,
        fits: VV,
    } = VX;
    let ((tspan, tsplice), dsplice, (tfull, dfull)) = reload_data(lmd);

    // read the model file.
    // step 4: calculate the clipping slice or viz slice
    let eoffset = offset.unwrap_or(0_i64) as usize;
    let elimit: usize = limit.unwrap_or(dfull.shape()[0]);
    let estrides = strides.unwrap_or(1);
    // All time/data in the viz slice
    let tall: Vec<f64> = tfull
        .slice(s![eoffset..(eoffset+elimit); estrides])
        .to_vec();
    let dall: Vec<f64> = dfull
        .slice(s![eoffset..(eoffset+elimit); estrides])
        .to_vec();

    if (estrides > 1) {
        panic!(" estrides Not yet supported ");
    }
    // model (time) clipped to viz slice.  tsplice is the model support (in global data index), now we further restrict that to viz_slice
    let ETS: Vec<(usize, &f64)> = tsplice
        .iter()
        .enumerate()
        .filter(|(k, g)| ((**g as usize) >= eoffset) && ((**g as usize) < (eoffset + elimit)))
        .collect();
    let etsplice: Vec<f64> = ETS.iter().map(|(a, b)| **b).collect();
    let etslice: Vec<usize> = ETS.iter().map(|(a, b)| *a).collect();

    let mut path_output = PathBuf::from(output.as_str());
    path_output.set_extension("");
    let output_stem = path_output.clone();
    path_output.set_extension("png");

    (item..(item + 1)).into_par_iter().for_each(|k| {
        let model_fit {
            humps: humps,
            fitted_model: fitted_model,
            initial_model: initial_model,
            residual_total: residual_total,
            residual_per_point: residual_per_point,
        } = &VV[k];
        let mx = eval_M(&fitted_model, &etsplice);
        //(* flanking prediction *)
        let partial_model {
            before: pre,
            after: post,
        } = pmod;

        let ipre: Vec<usize> = tall
            .iter()
            .enumerate()
            .filter(|(k, g)| **g < *etsplice.first().unwrap())
            .map(|(k, v)| k)
            .collect();
        let ipre: Vec<usize> = ipre.iter().rev().take(pre).rev().map(|k| *k).collect();
        let tpre: Vec<f64> = tall
            .iter()
            .filter(|g| **g < *etsplice.first().unwrap())
            .map(|v| *v)
            .collect();
        let tpre: Vec<f64> = tpre.iter().rev().take(pre).rev().map(|v| *v).collect(); //tpre[ 0.max(tpre.len() - pre).. ].to_vec();
        let tpost: Vec<f64> = tall
            .iter()
            .filter(|g| **g > *etsplice.last().unwrap())
            .map(|v| *v)
            .take(post)
            .collect();
        let ipost: Vec<usize> = tall
            .iter()
            .enumerate()
            .filter(|(k, g)| **g > *etsplice.last().unwrap())
            .map(|(k, g)| k)
            .take(post)
            .collect();

        let mxall: Vec<f64> = eval_M(&fitted_model, &tall);
        let mxpre: Vec<f64> = eval_M(&fitted_model, &tpre);
        let mxpost: Vec<f64> = eval_M(&fitted_model, &tpost);

        let lsm = residual_total;
        // local resid
        //let ( rsumsqxx,rsumsq_ppxx,residxx, resid1xx ) = fitted_model.residual_mat(&tslice, &dsplice);
        // global resid
        let (rsumsqxx, rsumsq_ppxx, residxx, resid1xx) = fitted_model.residual(&tall, &dall);

        let i_viz: Vec<usize> = ipre
            .iter()
            .chain(etslice.iter().chain(ipost.iter()))
            .map(|k| *k)
            .collect();
        //let res_pre : Vec<f64> = ipre.iter().map(|k|{resid1xx[*k]}).collect();
        //let res_mod : Vec<f64> = etslice.iter().map( |k | { resid1xx[*k] }).collect();
        //let res_post : Vec<f64> = ipost.iter().map(|k|{resid1xx[*k]}).collect();
        ///////////////
        /// form envelopes
        let res_viz: Vec<f64> = i_viz.iter().map(|k| resid1xx[*k]).collect();
        //let dat_viz : Vec<f64> = i_viz.iter().map(|k|{dall[*k]}).collect();

        viz_lib2::plot_e(
            &tall,
            &res_viz,
            viz_lib2::PlotAction::PNG(
                format!("{}_{}", output_stem.to_str().unwrap(), k).into(),
                800,
                600,
                2.0,
            ),
            String::from(format!("model fit {} \n LSM {} ", k, lsm)),
            String::from("time "),
            String::from("displacements"),
        );
        println!("writing file {}", output_stem.to_str().unwrap())
    });
}

/////////////////////////////////
/// TODO : clean up this function its copied from residual_visualization but needs very little of the present code.
///
pub fn skew_visualization(
    models: String,
    output: String,
    item: usize,
    offset: Option<i64>,
    limit: Option<usize>,
    strides: Option<usize>,
    pmod: partial_model,
) {
    let mut path = PathBuf::from(models.as_str()); // get the model file.
    path.set_extension("yml");
    let mut data_file =
        File::open(path).expect("open model(s) file failed, please insure the extention is yml");
    let mut reader = BufReader::new(data_file);
    let VX: data_fit_package<ModelTanh> =
        serde_yaml::from_reader(reader).expect("problem reading yaml file");
    let data_fit_package {
        load_metadata: lmd,
        fits: VV,
    } = VX;
    let ((tspan, tsplice), dsplice, (tfull, dfull)) = reload_data(lmd);

    // read the model file.
    // step 4: calculate the clipping slice or viz slice
    let eoffset = offset.unwrap_or(0_i64) as usize;
    let elimit: usize = limit.unwrap_or(dfull.shape()[0]);
    let estrides = strides.unwrap_or(1);
    // All time/data in the viz slice
    let tall: Vec<f64> = tfull
        .slice(s![eoffset..(eoffset+elimit); estrides])
        .to_vec();
    let dall: Vec<f64> = dfull
        .slice(s![eoffset..(eoffset+elimit); estrides])
        .to_vec();

    if (estrides > 1) {
        panic!(" estrides Not yet supported ");
    }
    // model (time) clipped to viz slice.  tsplice is the model support (in global data index), now we further restrict that to viz_slice
    let ETS: Vec<(usize, &f64)> = tsplice
        .iter()
        .enumerate()
        .filter(|(k, g)| ((**g as usize) >= eoffset) && ((**g as usize) < (eoffset + elimit)))
        .collect();
    let etsplice: Vec<f64> = ETS.iter().map(|(a, b)| **b).collect();
    let etslice: Vec<usize> = ETS.iter().map(|(a, b)| *a).collect();

    let mut path_output = PathBuf::from(output.as_str());
    path_output.set_extension("");
    let output_stem = path_output.clone();
    path_output.set_extension("png");

    (item..(item + 1)).into_par_iter().for_each(|k| {
        let model_fit {
            humps: humps,
            fitted_model: fitted_model,
            initial_model: initial_model,
            residual_total: residual_total,
            residual_per_point: residual_per_point,
        } = &VV[k];
        let mx = eval_M(&fitted_model, &etsplice);
        //(* flanking prediction *)
        let partial_model {
            before: pre,
            after: post,
        } = pmod;

        let ipre: Vec<usize> = tall
            .iter()
            .enumerate()
            .filter(|(k, g)| **g < *etsplice.first().unwrap())
            .map(|(k, v)| k)
            .collect();
        let ipre: Vec<usize> = ipre.iter().rev().take(pre).rev().map(|k| *k).collect();
        let tpre: Vec<f64> = tall
            .iter()
            .filter(|g| **g < *etsplice.first().unwrap())
            .map(|v| *v)
            .collect();
        let tpre: Vec<f64> = tpre.iter().rev().take(pre).rev().map(|v| *v).collect(); //tpre[ 0.max(tpre.len() - pre).. ].to_vec();
        let tpost: Vec<f64> = tall
            .iter()
            .filter(|g| **g > *etsplice.last().unwrap())
            .map(|v| *v)
            .take(post)
            .collect();
        let ipost: Vec<usize> = tall
            .iter()
            .enumerate()
            .filter(|(k, g)| **g > *etsplice.last().unwrap())
            .map(|(k, g)| k)
            .take(post)
            .collect();

        let mxall: Vec<f64> = eval_M(&fitted_model, &tall);
        let mxpre: Vec<f64> = eval_M(&fitted_model, &tpre);
        let mxpost: Vec<f64> = eval_M(&fitted_model, &tpost);

        let lsm = residual_total;
        // local resid
        //let ( rsumsqxx,rsumsq_ppxx,residxx, resid1xx ) = fitted_model.residual_mat(&tslice, &dsplice);
        // global resid
        let (rsumsqxx, rsumsq_ppxx, residxx, resid1xx) = fitted_model.residual(&tall, &dall);

        let i_viz: Vec<usize> = ipre
            .iter()
            .chain(etslice.iter().chain(ipost.iter()))
            .map(|k| *k)
            .collect();
        //let res_pre : Vec<f64> = ipre.iter().map(|k|{resid1xx[*k]}).collect();
        //let res_mod : Vec<f64> = etslice.iter().map( |k | { resid1xx[*k] }).collect();
        //let res_post : Vec<f64> = ipost.iter().map(|k|{resid1xx[*k]}).collect();
        ///////////////
        /// form envelopes
        let res_viz: Vec<f64> = i_viz.iter().map(|k| resid1xx[*k]).collect();
        //let dat_viz : Vec<f64> = i_viz.iter().map(|k|{dall[*k]}).collect();
        ////
        /// m_3( m ) = sum_j^m( r[j] - <r[:m]> )^3 / ((m-1) [r[:m])
        let mu3: Vec<f64> = (1..)
            .take(res_viz.len() + 1)
            .map(|m| {
                let avg_resid: f64 = res_viz.iter().take(m).sum::<f64>() / (m as f64);
                let resid_disp: Vec<f64> = res_viz.iter().take(m).map(|x| *x - avg_resid).collect();
                let resid_disp_sq: Vec<f64> = resid_disp.iter().map(|x| (*x) * (*x)).collect();
                let resid_disp_cu: Vec<f64> =
                    resid_disp.iter().map(|x| (*x) * (*x) * (*x)).collect();
                let sv = sqrt(resid_disp_sq.iter().sum::<f64>() / (m as f64));
                let mut dnom: f64 = sv * sv * sv * ((m - 1) as f64);
                if dnom <= 0.0 {
                    dnom = 1.0;
                }
                let m_3: f64 = resid_disp_cu.iter().sum::<f64>() / (dnom);
                m_3
            })
            .collect();

        viz_lib2::plot_e(
            &tall,
            &mu3,
            viz_lib2::PlotAction::PNG(
                format!("{}_{}", output_stem.to_str().unwrap(), k).into(),
                800,
                600,
                2.0,
            ),
            String::from(format!("model fit {} \n LSM {} ", k, lsm)),
            String::from("time "),
            String::from("skew"),
        );
        println!("writing file {}", output_stem.to_str().unwrap())
    });
}
*/