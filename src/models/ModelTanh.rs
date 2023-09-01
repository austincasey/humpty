use nalgebra::DVector;
use plotly::common::Mode;
use serde::{Serialize, Deserialize};

use crate::models::ParameterizedModel; 
use rand_distr::{Normal, Distribution};

use super::{SymbolicModel, VarProAdapter, VarProAdaptEval, VarProAdaptGradEval, VarProAdaptLabels}; 


//////////////
///
///  Tanh model
/// 
#[derive(Debug, Serialize, Deserialize, Clone , Copy)]
pub struct ModelTanh{
    pub kappa : f64,
    pub alpha : f64, 
    pub beta : f64,
} 

impl ModelTanh{
    pub fn new(kappa: f64, alpha: f64, beta : f64) -> Self { Self { kappa, alpha, beta }}

    
}


impl ParameterizedModel for ModelTanh{
    fn get_all_params(&self) -> Vec<f64>{
        return vec![ self.kappa, self.alpha, self.beta ]
    } 
    fn set_all_params( &mut self, V : &[f64]){
        self.kappa = V[0];
        self.alpha = V[1];
        self.beta = V[2];
    } 
    fn get_nonlinear_params( &self ) -> Vec<f64> {
        return  vec![self.alpha, self.beta ]
    }
    fn eval( &self, t : f64 ) -> f64{
        self.kappa *( 1.0 + libm::tanh( self.alpha * t + self.beta ))
    } 

    fn get_copy( &self ) -> Self {
        Self::new( self.kappa, self.alpha, self.beta )
    }
    fn mute( &mut self, rng : &rand::rngs::ThreadRng, mag : f64 , var : Option<&Self>){
        //println!( "\t before: {:?}",self);
        let normal_kappa = Normal::new(self.kappa, mag * match var { Some(v) => v.kappa, _=>1.}).unwrap();
        self.kappa = normal_kappa.sample(&mut rand::thread_rng());
        let normal_alpha = Normal::new(self.alpha, mag* match var { Some(v) => v.alpha, _=>1.}).unwrap();
        self.alpha = normal_alpha.sample(&mut rand::thread_rng());
        let normal_beta = Normal::new(self.beta, mag* match var { Some(v) => v.beta, _=>1.}).unwrap();
        self.beta = normal_beta.sample(&mut rand::thread_rng()); 
        //println!( "\t after: {:?}", self );
    }
    fn random_model( rng : &rand::rngs::ThreadRng ) -> Self {
        let normal_kappa = Normal::new(0.0, 1.0).unwrap();
        let kappa = normal_kappa.sample(&mut rand::thread_rng());
        let normal_alpha = Normal::new(0.0, 1.0 ).unwrap();
        let alpha = normal_alpha.sample(&mut rand::thread_rng());
        let normal_beta = Normal::new(0.0, 1.0).unwrap();
        let beta = normal_beta.sample(&mut rand::thread_rng()); 
        Self::new(kappa, alpha, beta )
    }

}
 
//////////////////////////////
/// This interface is designed to adapt the Tanh model into 
/// VarPro formats.
/// W.C.
impl VarProAdapter for ModelTanh{
    fn separable_labels(&self, suffix: Option<String> ) -> VarProAdaptLabels {
        let V = match suffix{
            Some(V) => V,
            None => String::from(""),
        };
        VarProAdaptLabels::ARG2(
            [
                String::from(format!("alpha{}", V.clone() )), 
                String::from(format!("beta{}", V.clone()  ))
            ]
        )
    }

    fn separable_eval(&self) -> VarProAdaptEval { 
        VarProAdaptEval::ARG2( 
            Box::new(
                | t: &DVector<f64>, alpha: f64, beta: f64 | -> DVector<f64> 
                {
                    t.map(|t| 1.0 + libm::tanh( alpha * t + beta )) 
                }
            )
        )
    }

    fn separable_eval_grad(&self ) -> VarProAdaptGradEval {
        // this function exits for varpro application
        VarProAdaptGradEval::ARG2(
            [
                Box::new(
                    | t: &DVector<f64>, alpha: f64, beta: f64 | -> DVector<f64> 
                    {
                        t.map(|t| (t)/( ( ( t*alpha + beta).cosh()  ).powi(2) ))
                    }
                ),
                Box::new(
                    | t: &DVector<f64>, alpha: f64, beta: f64 | -> DVector<f64> 
                    {
                        t.map(|t| (1.0)/( ( ( t*alpha + beta).cosh()  ).powi(2) ))
                    }
                )                      
            ]
        )
    }
}



#[test]
fn test1(){
    println!( " dog ");
    let mut rng: rand::rngs::ThreadRng = rand::thread_rng() ;
    let M : ModelTanh = ParameterizedModel::random_model(&rng);
    println!( "{:#?}", M);
    let VarProAdaptLabels::ARG2(X) =  M.separable_labels(Some( String::from( "_double_check"))) 
    else {todo!()};
    println!( "{:#?}", X );
    let VarProAdaptEval::ARG2(F ) = M.separable_eval() else {todo!()};
    let tspan: DVector<f64> = DVector::from_vec( vec![1.0, 2.0, 3.0, 4.0] );
    let Y = F( &tspan , 0.5, 0.75 );
    println!( "t={:#?} -> Y={:#?}", tspan, Y ); 
    let VarProAdaptGradEval::ARG2( G ) = M.separable_eval_grad() else {panic!()};
    G.iter().zip(X.iter()).for_each(
        |(g, l) |
        {
            let dfdl = g( &tspan , 0.5, 0.75 );
            println!( "DF/D{} = {:#?}", l, dfdl )
        }
    );
    println!( "CHECKS OUT WITH OCTAVE.  spot checked on Aug28")
}