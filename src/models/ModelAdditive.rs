use nalgebra::{Const, DVector};
use serde::{Serialize, Deserialize};
use rand_distr::{Geometric, Distribution};
use varpro::prelude::SeparableModelBuilder;
use super::{ParameterizedModel, VarProAdapter, ModelTanh};

/// The idea here is to paste together other models implementing the paramterized model
/// 
/// 
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModelAdditive<M> where M: ParameterizedModel + Clone  + VarProAdapter{ 
    pub components : Vec<M>, 
}
impl<M> ModelAdditive<M> where M : ParameterizedModel + Clone  + VarProAdapter{
    pub fn new(components: Vec<M>) -> Self { Self { components } }

    // Note, it appears that there is no great way to extend varpro into the models themselves, 
    //       the calling convention is that each model requires a fuctions such as:
    //       fn f1(x: &DVector<f32>, a: f32, b: f32) -> DVector<f32>
    //       fn f1_da(x: &DVector<f32>, a: f32, b: f32) -> DVector<f32>
    //       fn f1_db(x: &DVector<f32>, a: f32, b: f32) -> DVector<f32>
    // Had they made the convetion:
    //       fn f1(x: &DVector<f32>, params : Vec<f32>) -> DVector<f32>
    // This would have been easier tocreate the interface within a traite, however not all base functions have same number of arguments.
    // Another option would be to treat all functions as if they take 10 inputs, and disregard the ones they do not take, but I don't like that either.
    // 
    // I choose to put the varpro model interface here, as it seems the top level model where that can be orgnaized.




    pub fn separable_labels( &self ) -> Vec<String> {
        let mut rv : Vec<String> = Vec::new();
        self.components.iter()
            .enumerate()
            .for_each( 
                | (k, m) | 
                { 
                    let y = format!("{}", k); 
                    rv.append( &mut m.separable_labels(Some(y)).toVec() )
                } 
        );
        rv
    }

    pub fn get_separable_params( &self ) -> Vec<f64> {
        let mut rv : Vec<f64> = Vec::new();
        self.components.iter().for_each(
            |m|
            {rv.append( &mut m.get_nonlinear_params());}
        );
        rv
    }

    pub fn build_varpro_separable_model (&self,  
        tspan: nalgebra::Matrix<f64, nalgebra::Dyn, Const<1>, nalgebra::VecStorage<f64, nalgebra::Dyn, Const<1>>>, 
        ) -> varpro::model::SeparableModel<f64>{
            let labels = self.separable_labels();
            let model = SeparableModelBuilder::<f64>::new(labels.as_slice() )
            .independent_variable(tspan);
            let first_mod = self.components.get(0).unwrap();
            let model_first: varpro::model::builder::SeparableModelBuilderProxyWithDerivatives<f64> = 
            {
                let suffix = format!("_{}", 0 );
                let base_labels = first_mod.separable_labels(Some( suffix )).toVec();
                let VPFA = first_mod.separable_eval();
                let VPFAG = first_mod.separable_eval_grad();
                let acc = VPFA.start_add_to_model_builder(base_labels.clone(), model);
                VPFAG.add_to_model_builder(base_labels, acc)
            };

            let model_rest = self.components.iter().enumerate().skip(1).fold(
                model_first,
                |acc,(k, base_fn_model)|
                {   
                    let suffix = format!("_{}", k );
                    let base_labels = base_fn_model.separable_labels(Some( suffix )).toVec();
                    let VPFA = base_fn_model.separable_eval();
                    let VPFAG = base_fn_model.separable_eval_grad();
                    let acc = VPFA.add_to_model_builder(  base_labels.clone() , acc );
                    VPFAG.add_to_model_builder(base_labels, acc )
                }
            );

            let model_last = model_rest.invariant_function(|x|DVector::from_element(x.len(),1.))
                .initial_parameters(self.get_separable_params())
                .build()
                .unwrap();
            model_last 

}




}

impl<M> ParameterizedModel for ModelAdditive<M> 
where M : ParameterizedModel + Clone + VarProAdapter{
    fn get_all_params(&self) -> Vec<f64>{
        self.components.iter()
            .flat_map(|m| m.get_all_params())  
            .collect()
    }
    /// this function is designed around varpro which provides linear and separable prameters vectors upon return.
    fn set_all_params( &mut self, p : &[f64]){
        let mut k = 0; 
        self.components.iter_mut().for_each(
            |c| 
            {
                let num_pars = c.get_all_params().len();  // for tanh : kappa, alpha, beta
                c.set_all_params( &p[k..(k+ num_pars)]);
                k += num_pars;
            }
        )
    } 
    fn get_nonlinear_params(&self) -> Vec<f64>{
        self.components.iter()
        .flat_map(|m| m.get_nonlinear_params())  
        .collect()
    }
    fn eval( &self, t : f64 ) -> f64{
        self.components
        .iter()
        .map( |x| x.eval( t ) ).collect::<Vec<f64>>()
        .iter_mut()
        .fold( 
            0.0, 
            move |acc, el | -> f64 { acc  +  *el }
        )
    }
    fn get_copy( &self ) -> Self {
        Self::new( self.components.iter().map( |x| x.get_copy() ).collect() )
    }
    fn mute( &mut self, rng : &rand::rngs::ThreadRng, mag : f64  , var : Option<&Self>){
        match var {
            Some( v )=> {
                let mut c = 0; 
                for x in &mut self.components{
                    x.mute( rng, mag , Some( &v.components[c] )) ;
                    c += 1;
                }
            }, 
            _ => {
                for x in &mut self.components{
                    x.mute( rng, mag , None) ;
                }
            }

        }   
    }
    /// 
    /// This function will generate a random set of models of type M.  
    ///      How long is the string? 
    ///      Distributed geom( 0.25 ) or continuation probability is 0.75
    fn random_model( rng : &rand::rngs::ThreadRng ) -> Self {
        let geo = Geometric::new(0.25).unwrap();
        let v = geo.sample(&mut rand::thread_rng()) + 1;
        Self::new( (0..v).map( |_| M::random_model(rng) ).collect() )
    }

}


#[test]
fn test_x( ){
    let mut rng: rand::rngs::ThreadRng = rand::thread_rng() ;

    let V : ModelAdditive<crate::models::ModelTanh::ModelTanh> = ParameterizedModel::random_model(&rng);
    println!("{:#?}", V );
    println!( " params: {:#?}", V.separable_labels() );
    println!( "   asvec:{:#?}", V.get_separable_params())
}