use nalgebra::{DVector, Scalar};
use num_traits::Float;
//extern crate nalgebra as na;
use varpro::{model::{*, builder::SeparableModelBuilderProxyWithDerivatives}, prelude::SeparableModelBuilder};

pub mod ModelConstant;
pub mod ModelTanh; 
pub mod ModelAdditive;
pub mod ModelAffine; 

pub trait ParameterizedModel{
    fn get_all_params(&self) -> Vec<f64>;
    fn set_all_params(&mut self, &[f64]);
    fn get_nonlinear_params(&self) -> Vec<f64> ;
    fn eval(&self, t : f64 ) -> f64 ; 
    fn get_copy( &self ) -> Self ;
    fn mute( &mut self, rng: &rand::rngs::ThreadRng, mag : f64 , var : Option<&Self>);   
    fn random_model( rng : &rand::rngs::ThreadRng ) -> Self ; 
    //fn eval_grad( &self, t: f64, P : Vec<f64>) -> Vec<f64>; 
}

/// these are aimed at autograd - however I've done nothing with it yet.  W.C.
pub trait SymbolicModel
{
    fn eval_seperable<X: Float + Scalar>( &self , t: &DVector<X>, p: DVector<X> ) -> DVector<X>  ;
    fn eval_separable_grad<X: Float + Scalar>( &self, t: &DVector<X>, p: DVector<X>, k: usize ) -> DVector<X> ; 
}

/// these structures are aimed at linking our models crate to the varpro methods.
/// 
/// HOWTO expand for a basis function with more than four inputs.  You will need to add the ARGN into the enums and cover the cases in the implementaiotns to develop the VarPro model builder
pub enum VarProAdaptEval {
    ARG1( Box<dyn Fn( &DVector<f64>, f64 ) -> DVector<f64>> ), 
    ARG2( Box<dyn Fn( &DVector<f64>, f64, f64) -> DVector<f64>> ), 
    ARG3( Box<dyn Fn( &DVector<f64>, f64, f64, f64) -> DVector<f64>> ),
    ARG4( Box<dyn Fn( &DVector<f64>, f64, f64, f64, f64) -> DVector<f64>> ),
}

impl VarProAdaptEval {
    pub fn start_add_to_model_builder( self, labels : Vec<String>, VPM :  SeparableModelBuilder<f64>) -> SeparableModelBuilderProxyWithDerivatives<f64>{
        match self { 
            VarProAdaptEval::ARG1(f) => VPM.function( labels, f),
            VarProAdaptEval::ARG2(f) => VPM.function( labels, f) ,
            VarProAdaptEval::ARG3(f) => VPM.function( labels, f),
            VarProAdaptEval::ARG4(f) => VPM.function( labels, f),
            //M.function( )
        }
    }
    pub fn add_to_model_builder( self, labels : Vec<String>, VPM :  SeparableModelBuilderProxyWithDerivatives<f64>) -> SeparableModelBuilderProxyWithDerivatives<f64>{
        match self { 
            VarProAdaptEval::ARG1(f) => VPM.function( labels, f),
            VarProAdaptEval::ARG2(f) => VPM.function( labels, f) ,
            VarProAdaptEval::ARG3(f) => VPM.function( labels, f),
            VarProAdaptEval::ARG4(f) => VPM.function( labels, f),
            //M.function( )
        }
    }
}
pub enum VarProAdaptGradEval {
    ARG1( [Box<dyn Fn( &DVector<f64>, f64 ) -> DVector<f64>>;1] ),
    ARG2( [Box<dyn Fn( &DVector<f64>, f64, f64) -> DVector<f64>>;2] ), 
    ARG3( [Box<dyn Fn( &DVector<f64>, f64, f64, f64) -> DVector<f64>>;3] ),
    ARG4( [Box<dyn Fn( &DVector<f64>, f64, f64, f64, f64) -> DVector<f64>>;4] ),
}

impl VarProAdaptGradEval {
    pub fn add_to_model_builder( self, labels : Vec<String>, VPM : SeparableModelBuilderProxyWithDerivatives<f64> ) -> SeparableModelBuilderProxyWithDerivatives<f64> 
    {
        match self {
            VarProAdaptGradEval::ARG1([f1]) => 
            {
                VPM.partial_deriv(labels[0].clone(), f1 )
            },
            VarProAdaptGradEval::ARG2([f1,f2]) =>            
            {
                VPM.partial_deriv(labels[0].clone(), f1 )
                    .partial_deriv(labels[1].clone(), f2 )
            },
            VarProAdaptGradEval::ARG3([f1,f2,f3]) =>             
            {
                VPM.partial_deriv(labels[0].clone(), f1 )
                    .partial_deriv(labels[1].clone(), f2 )
                    .partial_deriv(labels[2].clone(), f3 )

            },
            VarProAdaptGradEval::ARG4([f1,f2,f3,f4]) =>             
            {
                VPM.partial_deriv(labels[0].clone(), f1 )
                .partial_deriv(labels[1].clone(), f2 )
                .partial_deriv(labels[2].clone(), f3 )
                .partial_deriv(labels[3].clone(), f4)
            }
        }
    }
}
pub enum VarProAdaptLabels {
    ARG1( [String;1] ),
    ARG2( [String;2] ),
    ARG3( [String;3] ),
    ARG4( [String;4] ),
}

/// this implementation produces a vector of
impl VarProAdaptLabels {
    pub fn toVec(&self) ->Vec<String>{
        match self{
            VarProAdaptLabels::ARG1(X) => X.iter().map( |x| x.clone()).collect(),
            VarProAdaptLabels::ARG2(X) =>  X.iter().map( |x| x.clone()).collect(),
            VarProAdaptLabels::ARG3(X) =>  X.iter().map( |x| x.clone()).collect(),
            VarProAdaptLabels::ARG4(X) =>  X.iter().map( |x| x.clone()).collect(),
        }
    }
}

pub trait VarProAdapter
{
    fn separable_labels(&self, suffix: Option<String> ) -> VarProAdaptLabels;
    fn separable_eval(&self) -> VarProAdaptEval;
    fn separable_eval_grad(&self ) -> VarProAdaptGradEval ;

} 
