use nalgebra::{Matrix, Dyn, Const};
use serde::{Serialize, Deserialize};
use varpro::solvers::levmar::{LevMarProblemBuilder, LevMarSolver};
use varpro::prelude::LeastSquaresProblem;

use crate::models::{ParameterizedModel, ModelAdditive::ModelAdditive, ModelConstant::ModelConstant};

use super::VarProAdapter;

#[derive(Debug, Serialize, Deserialize, Clone )]
pub struct AffineAdditive<M> where M : ParameterizedModel + Clone  + VarProAdapter {
    pub tm : ModelAdditive<M>,  
    pub km : ModelConstant
}

impl<M> AffineAdditive<M> where M : ParameterizedModel + Clone + VarProAdapter {
    pub fn new(tm: ModelAdditive<M>, km: ModelConstant) -> Self { Self { tm, km } }

    /// EDIT POINT - We are here.
    fn var_pro_curve_fit( &mut self , 
        tspan : Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>>, 
        data : Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>>, 
    ) -> Result<bool, String>{

        let model = self.tm.build_varpro_separable_model( tspan );
        //println!( " MODEL:  {:?}", model); 
        //println!(" forming problem"); 
        let problem = LevMarProblemBuilder::new(model)
            .observations(data)
            .build()
            .unwrap(); 

        //println!( " problem : {:?} ", &problem);  
        let (solved_problem, report) = LevMarSolver::new().minimize(problem);

        if ! report.termination.was_successful(){
            return Err( String::from("NevMarSolv -- Error"))
        } 
        //println!( " \n\n ----- \n\n ");
        let alpha = solved_problem.params();
        let coeff = solved_problem.linear_coefficients().unwrap();
        let A : Vec<f64> = alpha.as_slice().into();
        let C : Vec<f64> = coeff.as_slice().into();
        //interprete_model(K, A, C);
        // TODO WRITE VALUES BACK.
        Ok( self) 
    }   


}

impl<M> ParameterizedModel for AffineAdditive<M> 
where M : ParameterizedModel + Clone  + VarProAdapter{
    fn get_all_params(&self)  -> Vec<f64> where M : ParameterizedModel{
        let mut rv = self.tm.get_all_params();
        let mut rv2 = self.km.get_all_params();
        rv.append( &mut rv2) ;
        rv
    } 
    fn get_nonlinear_params(&self )-> Vec<f64> {
         let mut rv = self.tm.get_nonlinear_params();
         let mut rv2 = self.km.get_nonlinear_params();
         rv.append( &mut rv2 );
         rv 
    }
    fn eval( &self, t : f64 ) -> f64{
        self.tm.eval( t ) + self.km.eval(t )
    }
    fn get_copy( &self ) -> Self {
        Self::new( self.tm.get_copy(), self.km.get_copy() )
    }
    fn mute( &mut self, rng : &rand::rngs::ThreadRng, mag : f64 , var : Option<&Self>){
        self.tm.mute( rng, mag , match var {Some(v) => Some(&v.tm), _ => None });
        self.km.mute( rng, mag , match var {Some(v) => Some(&v.km), _ => None } );   
    }
    /// 
    /// TODO: this function currently creates an empty model, but could generate something more interesting
    fn random_model( rng : &rand::rngs::ThreadRng ) -> Self {
        Self::new(  ModelAdditive::random_model(rng), ModelConstant::random_model(rng))
    }


 


}
