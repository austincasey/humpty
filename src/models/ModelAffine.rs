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
    pub fn curve_fit( &mut self , 
        tspan : &Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>>, 
        data : &Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>>, 
    ) -> Result<bool, String>{

        let model = self.tm.build_varpro_separable_model( tspan.clone() );
        //println!( " MODEL:  {:?}", model); 
        //println!(" forming problem"); 
        let problem = LevMarProblemBuilder::new(model)
            .observations(data.clone())
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
        //now we must splice these together for parameter adjustment.
        let mut RVV: Vec<f64> = Vec::new();
        let mut k1 = 0; 
        let mut k2 = 0;
        self.tm.components.iter().for_each(
            |c| 
            {
                let nx = c.get_nonlinear_params().len(); 
                RVV.push( coeff[k1] );
                k1 += 1;
                (k2..(k2+nx)).for_each( 
                    |jj|  
                    RVV.push( alpha[jj]) 
                );
                k2 += nx; 
            }
        );
        RVV.push( coeff[k1] );
        self.set_all_params( RVV.as_slice() );

        Ok( true ) 
    }   


    pub fn residual_mat( & self , 
        tspan : &Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>>, 
        data : &Matrix<f64, Dyn, Const<1>, nalgebra::VecStorage<f64, Dyn, Const<1>>>, 
    ) -> (f64, f64, Vec<f64>, Vec<f64>)     {
        let M :Vec<f64> = tspan.iter().map( |t| self.eval(*t) ).collect();
        let ML = M.len();
        let resid1 : Vec<f64> = data.iter().zip( M.iter() ).map( |(a,b)| { (a -b) }).collect();
        let resid : Vec<f64> = data.iter().zip( M.iter() ).map( |(a,b)| { (a -b)*(a-b ) }).collect();
        let rsum = resid.iter().fold(0., |acc,x | acc + x );
        let rsumsq = libm::sqrt( rsum ); 
        let rsumsq_pp = rsumsq / (ML as f64 );
        ( rsumsq,rsumsq_pp,resid, resid1 )
    }
<<<<<<< HEAD
    ////////////////////////////
    /// residual calle on the model will evaluate values into M
    ///     M the model eval on given timespan
    ///     resid1:  then calcualte the disp vector as data_i - Model_i
    ///     resid :  squared disp vector (resid1_i^2)
    ///     rsum :   summed squared disp
    ///     rsumsq :  sqrt( rsum )
    ///     rsumsq_pp : rsumsq / len(M)
    /// returns rsumsq, rsumsq_pp, resid, resid1 in that order.
    
=======

>>>>>>> refs/remotes/origin/main
    pub fn residual( & self , 
        tspan : &Vec<f64>, 
        data : &Vec<f64>, 
    ) -> (f64, f64, Vec<f64>, Vec<f64>)     {
        let M :Vec<f64> = tspan.iter().map( |t| self.eval(*t) ).collect();
        let ML = M.len();
        let resid1 : Vec<f64> = data.iter().zip( M.iter() ).map( |(a,b)| { (a -b) }).collect();
        let resid : Vec<f64> = data.iter().zip( M.iter() ).map( |(a,b)| { (a -b)*(a-b ) }).collect();
        let rsum = resid.iter().fold(0., |acc,x | acc + x );
        let rsumsq = libm::sqrt( rsum ); 
        let rsumsq_pp = rsumsq / (ML as f64 );
        ( rsumsq,rsumsq_pp,resid, resid1 )
    }
    /// 
    /// This function will generate a random set of models of type M.  
    ///      How long is the string? 
    ///      Distributed geom( 0.25 ) or continuation probability is 0.75
    pub fn random_model_given_humps( humps: usize , rng : &rand::rngs::ThreadRng ) -> Self {
        Self::new(  ModelAdditive::<M>::random_model_given_humps( humps, rng ), ModelConstant::random_model(rng))
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
    fn set_all_params(&mut self, p:  &[f64]) {
        self.tm.set_all_params( &p[..(p.len()-1)]);  // seq shoudl be: kappa_1, alpha_1, beta_1, ... k_n, a_n, b_n, K_all
        self.km.set_all_params( &p[(p.len()-1)..(p.len()) ]); 
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


#[test]
fn test_1(){
    let Y = [1.2, 3.3, 4.4, 5.5, 6.6 ];
    println!( "{:?}", &Y[(Y.len()-1)..(Y.len())]);
    println!( "{:?}", &Y[..(Y.len()-1)]);
}