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

use crate::models::ModelAffine::AffineAdditive;
use crate::models::ModelTanh::ModelTanh;
use crate::viz_lib;
use crate::viz_lib::*;
use crate::viz_lib2;

use crate::main_mod_dm_fit::{
    data_fit_load_metadata, data_fit_package, load_data, model_fit, reload_data,
};

// to calcualte the residual
use ndarray::{array, aview1, Axis};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct partial_model {
    before: usize,
    after: usize, // flanks( 0, 0 ) is as is, flanks( 10, 0) is retroduction of 10 points, flanks( 0, 10) is prediction of ten points.
}

impl partial_model {
    pub fn new(before: usize, after: usize) -> Self {
        Self { before, after }
    }
}

pub fn basic_visualization(
    models: String,
    output: String,
    top: usize,
    offset: Option<i64>,
    limit: Option<usize>,
    strides: Option<usize>,
    pmod: partial_model,
    title: Option<String>, 
    xlabel: Option<String>,
    ylabel: Option<String>
) {
    // step 1: read the model file.
    let mut path = PathBuf::from(models.as_str());
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
    // step 2: prepare output stubs.
    let mut path_output = PathBuf::from(output.as_str());
    path_output.set_extension("");
    let output_stem = path_output.clone();

    // step 3: reload the data ( used in the model )
    // tsclice is are a range (local intrensic idex), tplice is extrinsic index (index in original data), dsplice is sliced data,
    // tfull is all time steps in datafile, dfull is all data in data file.
    let ((tslice, tsplice), dsplice, (tfull, dfull)) = reload_data(lmd);

    // step 4: calculate the clipping slice or viz slice
    let eoffset = offset.unwrap_or(0_i64) as usize;
    let elimit: usize = limit.unwrap_or(dfull.shape()[0]);
    let estrides = strides.unwrap_or(1);
    // restrict time/data to the viz slice
    let tall: Vec<f64> = tfull
        .slice(s![eoffset..(eoffset+elimit); estrides])
        .to_vec();
    let dall: Vec<f64> = dfull
        .slice(s![eoffset..(eoffset+elimit); estrides])
        .to_vec();

    if (estrides > 1) {
        panic!(" estrides Not yet supported ");
    }
    // model (time) clipped to viz slice
    let ETS: Vec<(usize, &f64)> = tsplice
        .iter()
        .enumerate()
        .filter(|(k, g)| ((**g as usize) >= eoffset) && ((**g as usize) < (eoffset + elimit)))
        .collect();
    let etsplice: Vec<f64> = ETS.iter().map(|(a, b)| **b).collect();
    let etslice: Vec<usize> = ETS.iter().map(|(a, b)| *a).collect();

    println!( " basic with {:?}", &title );
    (0..min(top, VV.len())).into_par_iter().for_each(|k| {
        //step 4 clipping plane for model
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

        let mxall: Vec<f64> = eval_M(&fitted_model, &tall);
        let mxpre: Vec<f64> = eval_M(&fitted_model, &tpre);
        let mxpost: Vec<f64> = eval_M(&fitted_model, &tpost);

        println!(">tpre {:?}", &tpre);
        println!(">mpre {:?}", &mxpre);
        println!(">tpost {:?}", &tpost);
        println!(">mpost {:?}", &mxpost);

        let lsm = residual_total;
        let (rsumsqxx, rsumsq_ppxx, residxx, resid1xx) =
            fitted_model.residual_mat(&tslice, &dsplice);

        let title_string = match( &title ){
            Some(t) => t.clone(),
            None => format!(
                "model fit {}.{} LSM/data-point {:.5}",
                models.as_str().replace(".yml", ""),
                k,
                residual_per_point
            ) //format!("{}_fit_{}", output_stem.to_str().unwrap(), k).into(),
        };
        let xlabel_string = match( &xlabel ){
            Some(s) => s.clone(),
            None => String::from("time "),
        };
        let ylabel_string = match( &ylabel ){
            Some(s) => s.clone(),
            None => String::from("quantity"),
        };
        viz_lib2::plot_c(
            &tall,
            &dall,
            &etsplice,
            &mx,
            &tpre,
            &mxpre,
            &tpost,
            &mxpost,
            viz_lib2::PlotAction::PNG(
                format!("{}", output_stem.to_str().unwrap()).into(),
                800,
                600,
                2.0,
            ),
            String::from(&title_string),
            xlabel_string,
            ylabel_string,
        );
        println!("writing file {}", output_stem.to_str().unwrap())
    });
}

pub fn vec_there_and_back(v: &Vec<f64>) -> Vec<f64> {
    let mut val_there_and_back: Vec<f64> = v.clone(); // uses the viz_clip
    let mut val_back: Vec<f64> = v.clone().iter().rev().map(|x| *x).collect();
    val_there_and_back.append(&mut val_back);
    val_there_and_back
}

pub fn vec_envelope_to_region(vhi: &Vec<f64>, c: &Vec<f64>, vlo: &Vec<f64>) -> Vec<f64> {
    let mut val_there_and_back: Vec<f64> = vhi
        .iter()
        .enumerate()
        .map(|(j, x)| c[j].min(c[j] + *x))
        .collect();
    let mut data_back = vlo
        .clone()
        .iter()
        .enumerate()
        .rev()
        .map(|(j, x)| c[j].max(c[j] + *x))
        .collect();
    val_there_and_back.append(&mut data_back);
    val_there_and_back
}

pub fn residual_visualization(
    models: String,
    output: String,
    item: usize,
    offset: Option<i64>,
    limit: Option<usize>,
    strides: Option<usize>,
    pmod: partial_model,
    title: Option<String>, 
    xlabel: Option<String>,
    ylabel: Option<String>
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
    let ((tspan, tsplice), dsplice, (tfull, dfull)) = reload_data(lmd.clone());

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

    // step 3: reload the data ( used in the model )
    // tsclice is are a range (local intrensic idex), tplice is extrinsic index (index in original data), dsplice is sliced data,
    // tfull is all time steps in datafile, dfull is all data in data file.
    let ((tslice ,tsplice),
          dsplice  , 
         (tfull, dfull)) =  reload_data( lmd );

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
        let t_viz: Vec<f64> = i_viz.iter().map(|k| tall[*k]).collect();
        let m_viz: Vec<f64> = eval_M(&fitted_model, &t_viz);
        // Here is the proceedure for visualizing the growing residual.
        let quantile_params = vec![0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];
        let qpn = quantile_params.len();
        //let res_clip: Vec<f64> = viz_clip.map( |k| { resid1xx[k] } ).collect();
        let quantiles_running = rolling_quantiles(res_viz, quantile_params); // clipped like viz_clip
                                                                             // now must subtract the moving model values from the qunatiles running.

        let time_vizslice_there_and_back: Vec<f64> = vec_there_and_back(&t_viz.to_vec());

        // ABOVE HAVE TO BE TRIMED TO THE VIZ SLICE

        let regions_all: Vec<Vec<f64>> = (0..(qpn / 2))
            .map(|k| {
                let data_vizslice_there_and_back: Vec<f64> = vec_envelope_to_region(
                    &quantiles_running[k],
                    &m_viz,
                    &quantiles_running[qpn - (k + 1)],
                );
                data_vizslice_there_and_back
            })
            .collect();
        let title_string = match( &title ){
            Some(t) => t.clone(),
            None => format!("{}_fit_{}", output_stem.to_str().unwrap(), k).into(),
        };
        let xlabel_string = match( &xlabel ){
            Some(s) => s.clone(),
            None => String::from("time "),
        };
        let ylabel_string = match( &ylabel ){
            Some(s) => s.clone(),
            None => String::from("residual"),
        };
        viz_lib2::plot_d(
            &tall,
            &dall,
            &etsplice,
            &mx,
            &tpre,
            &mxpre,
            &tpost,
            &mxpost,
            &time_vizslice_there_and_back,
            &regions_all,
            viz_lib2::PlotAction::PNG(
                format!("{}", output_stem.to_str().unwrap()).into(),
                800,
                600,
                2.0,
            ),
            String::from(format!("model fit {} \n LSM {} ", k, lsm)),
            xlabel_string,
            ylabel_string,
        );

        println!("writing file {}", output_stem.to_str().unwrap())
    });
}

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
    title: Option<String>, 
    xlabel: Option<String>,
    ylabel: Option<String>
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
        let title_string = match( &title ){
            Some(t) => t.clone(),
            None => String::from(format!("model fit {} \n LSM {} ", k, lsm)),
        };
        let xlabel_string = match( &xlabel ){
            Some(s) => s.clone(),
            None => String::from("time "),
        };
        let ylabel_string = match( &ylabel ){
            Some(s) => s.clone(),
            None => String::from("displacements"),
        };
        viz_lib2::plot_e(
            &tall,
            &res_viz,
            viz_lib2::PlotAction::PNG(
                format!("{}", output_stem.to_str().unwrap()).into(),
                800,
                600,
                2.0,
            ),
            title_string,
            xlabel_string,
            ylabel_string,
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
    title: Option<String>, 
    xlabel: Option<String>,
    ylabel: Option<String>
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
        let title_string = match( &title ){
            Some(t) => t.clone(),
            None => String::from(format!("model fit {} \n LSM {} ", k, lsm)),
        };
        let xlabel_string = match( &xlabel ){
            Some(s) => s.clone(),
            None => String::from("time "),
        };
        let ylabel_string = match( &ylabel ){
            Some(s) => s.clone(),
            None => String::from("skew"),
        };      
        viz_lib2::plot_e(
            &tall,
            &mu3,
            viz_lib2::PlotAction::PNG(
                format!("{}", output_stem.to_str().unwrap() ).into(),
                800,
                600,
                2.0,
            ),
            title_string,
            xlabel_string,
            ylabel_string,
        );
        println!("writing file {}", output_stem.to_str().unwrap())
    });
}
/* 
    let elimit = match( limit ){
        Some(x) => x,
        None => dfull.shape()[0],
    };
    let estrides = match( strides ){
        Some( x ) => x ,
        None => 1  
    };
    let eoffset = match(offset ){
        Some(x) => x as usize,
        None => 0_usize,
    };
*/
pub fn intermediate_visualization(
    models: String,
    output: String,
    item: usize,
    pval: f64,
    offset: Option<i64>,
    limit: Option<usize>,
    strides: Option<usize>,

    title: Option<String>, 
    xlabel: Option<String>,
    ylabel: Option<String>
) {
    let mut path = PathBuf::from(models.as_str());
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
    //let V:  Vec<(f64, AffineAdditive<ModelTanh>, AffineAdditive<ModelTanh>)>

    let ((tspan, tsplice), dsplice, (tfull, dfull)) = reload_data(lmd);

    println!(" {}, {}, {}", models, output, item);
    println!(" {:?}\n{:?}\n", tspan, dsplice);
    // read the model file.

    println!(" we are here {:#?}", VV);

    let mut path_output = PathBuf::from(output.as_str());
    path_output.set_extension("");
    let output_stem = path_output.clone();
    path_output.set_extension("png");

    let elimit = match (limit) {
        Some(x) => x,
        None => dfull.shape()[0],
    };
    let estrides = match (strides) {
        Some(x) => x,
        None => 1,
    };
    let eoffset = match (offset) {
        Some(x) => x as usize,
        None => 0_usize,
    };

    let k = item;
    {
        let model_fit {
            humps: humps,
            fitted_model: fitted_model,
            initial_model: initial_model,
            residual_total: residual_total,
            residual_per_point: residual_per_point,
        } = &VV[k];

        let mx = eval_M(&fitted_model, &tspan.as_slice().to_vec());
        let lsm = residual_total;
        let lsmpp = residual_per_point;

        let title_string = match( &title ){
            Some(t) => t.clone(),
            None => String::from(format!("model fit {} \n LSM {} ", k, lsm)),
        };
        let xlabel_string = match( &xlabel ){
            Some(s) => s.clone(),
            None => String::from("time "),
        };
        let ylabel_string = match( &ylabel ){
            Some(s) => s.clone(),
            None => String::from("quantity"),
        }; 
        viz_lib::plot_model_with_markers(
            &tfull.slice(s![eoffset..elimit; estrides]).to_vec(),
            &dfull.slice(s![eoffset..elimit; estrides]).to_vec(),
            &tsplice.as_slice().to_vec(),
            &mx,
            fitted_model.clone(),
            0.20,
            PlotAction::PNG(
                format!("{}", output_stem.to_str().unwrap()).into(),
                800,
                600,
                2.0,
            ),
            title_string,
    xlabel_string,
            ylabel_string,
        );
        println!("writing file {}", output_stem.to_str().unwrap())
    }
}
 
//use polars::{df, prelude::{CsvReader, PolarsResult, DataFrame, SerReader}};

use polars::prelude::QuantileInterpolOptions;
use polars::{df, prelude::*};
#[test]
fn testit() {
    let quantile_params = vec![0.0, 0.1, 0.5, 0.9, 1.0];
    let data = vec![1., 2., 3., 43., 5., 6., 7., 78.];
    rolling_quantiles(data, quantile_params);
}
pub fn rolling_quantiles(data: Vec<f64>, quantile_params: Vec<f64>) -> Vec<Vec<f64>> {
    // EDIT POINT .. TRYING TO TEST THE ORDER STATS OR quantiles.

    let df: DataFrame = df!("stats" => data.as_slice()).unwrap();
    print!("stats {:?}", df["stats"]);

    let mut quantiles: Vec<Vec<f64>> = (1..df.height() + 1)
        .into_iter()
        .map(|row| {
            let val: Vec<f64> = quantile_params
                .iter()
                .map(|cut| {
                    let dstar = df.slice(0, row)["stats"]
                        .quantile_as_series(*cut, QuantileInterpolOptions::Linear)
                        .unwrap();
                    dstar.f64().expect("issues").get(0).expect("issue ")
                })
                .collect();
            val
        })
        .collect();

    let qtraces: Vec<Vec<f64>> = (0..quantile_params.len())
        .map(|k| (0..quantiles.len()).map(|v| quantiles[v][k]).collect())
        .collect();
    print!("fruit mids {:?}", df["stats"]);
    print!(" quantile {:?}", quantiles);
    print!(" qutraces {:?}", qtraces);
    qtraces
}

//use polars::{df, prelude::{CsvReader, PolarsResult, DataFrame, SerReader}};
/* 
use polars::{prelude::*, df};
use polars::prelude::QuantileInterpolOptions;
#[test]
fn testit(){
    let quantile_params = vec![ 0.0, 0.1, 0.5, 0.9, 1.0 ]; 
    let data = vec![1.,2.,3.,43.,5.,6.,7.,78.];
    rolling_quantiles( data , quantile_params );
}
fn rolling_quantiles( data: Vec<f64> , quantile_params : Vec<f64>) -> Vec<Vec<f64>> { // EDIT POINT .. TRYING TO TEST THE ORDER STATS OR quantiles. 
   
    let df: DataFrame = df!("stats" => data.as_slice()).unwrap();
    print!( "stats {:?}", df["stats"]);
   
    let mut quantiles : Vec<Vec<f64>> = (1..df.height()+1).into_iter().map(|row| {
        let val : Vec<f64> = quantile_params.iter().map(
            |cut|
            {
                let dstar = df.slice(0, row)["stats"].quantile_as_series(*cut, QuantileInterpolOptions::Linear ).unwrap();
                dstar.f64().expect("issues").get(0).expect("issue ")
            }
        ).collect();
        val
    }).collect();

    let qtraces : Vec<Vec<f64>> = (0..quantile_params.len()).map( |k|{ (0..quantiles.len()).map(|v|{quantiles[v][k] }).collect() } ).collect();
    print!( "fruit mids {:?}", df["stats"] );
    print!( " quantile {:?}", quantiles );
    print!( " qutraces {:?}", qtraces ); 
    qtraces 
}
    */