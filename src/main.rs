use clap::{Command, Arg, value_parser, arg, ArgGroup, Subcommand};
use humpty::main_mod_dm_gen::{build_tanh_model_from_string, write_data};
use humpty::main_mod_dm_fit::*; 
use humpty::main_mod_dm_viz::*;
use humpty::main_mod_dm_exp::*;
//use std::intrinsics::offset;
use std::path::{Path, PathBuf};

fn main() {
    let m: clap::ArgMatches = cli().get_matches();

    match m.subcommand() {
        // MODULE 1 create the HDC Brain Architecture 
        Some(("gen", m)) => { 
            match m.subcommand(){
                Some(("tanh", m )) => 
                {
                    let params : String = m.get_one::<String>("PARAMS").expect("paramters are required").clone();
                    let error : f64 = m.get_one::<f64>("error" ).expect("parsing error issue").clone();
                    let time_steps : usize = m.get_one::<usize>( "tsteps").expect("parsing steps issue").clone();
                    let output  = Path::new( m.get_one::<String>("OUTPUT" ).expect( "output file name needed.").into() );
                    println!( " here with params = {params} , error ={error} steps={time_steps} output={:?}", &output );
                    // TODO ... lets improve upon this interface ...
                    //println!( " gen {output} ");
                    let ( t, d, D , M)  = build_tanh_model_from_string( time_steps, params, error, true ); 
                    let stub = output.with_extension(""); //.as_os_str().to_str().unwrap();
                    let _= write_data( format!("{}.csv", stub.as_os_str().to_str().unwrap() ).as_str(), &t, &D ); 
                    //let f = std::fs::File::open("out.yml").expect("Could not open file.");
                    //let mut scrape_config: Config = serde_yaml::from_reader(f).expect("Could not read values.");
                    let serialized = serde_yaml::to_string(&M).unwrap();
                    std::fs::write(format!("{}_model.yml", stub.as_os_str().to_str().unwrap() ).as_str() , serialized).unwrap();
                }
                _ => {}
            }
        },
        Some(("fit", m)) => {
            let input : String = m.get_one::<String>("INPUT").expect("input file required").clone();
            let output : String = m.get_one::<String>("OUTPUT").expect("output file required").clone();
            let humps : usize = m.get_one::<usize>("humps" ).expect("parsing humps issue").clone();
            let samples : usize = m.get_one::<usize>("samples" ).expect("parsing humps issue").clone();
            let reports : usize = m.get_one::<usize>("reports" ).expect("parsing reports issue").clone();
            let offset  = m.get_one::<i64>("offset");
            let limit  = m.get_one::<usize>( "limit"); 
            let strides  = m.get_one::<usize>("strides");
            println!( " fitting model ({input}, {output}, {humps}, {samples})");
            model_curve_fitting( input, output, humps, samples, reports, offset, None, None ); 
        },
        Some( ("viz", m )) => {
            match m.subcommand(){
                Some(("basic", m )) => 
                {
                    let data : String = m.get_one::<String>("DATA").expect( "data file not specified").clone();
                    let models : String = m.get_one::<String>("MODEL").expect( "MODEL file not specified").clone();
                    let output : String = m.get_one::<String>("OUTPUT").expect( "OUTPUT file not specified").clone();
                    let top : usize = m.get_one::<usize>("top" ).expect("parsing top issue").clone();
                    let offset  = m.get_one::<i64>("offset");
                    let limit  = m.get_one::<usize>( "limit"); 
                    let strides  = m.get_one::<usize>("strides");
                    basic_visualization( data, models, output, top, offset, limit, strides );
                },
                Some(("intermediate", m )) => {
                    let data : String = m.get_one::<String>("DATA").expect( "data file not specified").clone();
                    let models : String = m.get_one::<String>("MODEL").expect( "MODEL file not specified").clone();
                    let output : String = m.get_one::<String>("OUTPUT").expect( "OUTPUT file not specified").clone();
                    let item : usize = m.get_one::<usize>("item" ).expect("parsing item issue").clone();
                    let pval : f64 = m.get_one::<f64>("pval" ).expect("parsing pval issue").clone();
                    let offset  = m.get_one::<i64>("offset");
                    let limit  = m.get_one::<usize>( "limit"); 
                    let strides  = m.get_one::<usize>("strides");
                    intermediate_visualization( data, models, output, item, pval, offset, limit, strides );
                }
                _ => {}
            }
        }, 
        Some( ("exp", m )) => {
            match m.subcommand(){
                Some(("basic", m )) => 
                {
                    let models : String = m.get_one::<String>("MODEL").expect( "MODEL file not specified").clone();
                    let top : usize = m.get_one::<usize>("top" ).expect("parsing top issue").clone();
                    basic_explanation( models, top );
                },
                Some(("intermediate", m )) => {
                    let models : String = m.get_one::<String>("MODEL").expect( "MODEL file not specified").clone();
                    let top : usize = m.get_one::<usize>("top" ).expect("parsing top issue").clone();
                    let pval : f64 = m.get_one::<f64>("pval" ).expect("parsing pval issue").clone();
                    intermediate_explanation( models, top, pval );
                }
                _ => {}
            }
        } 
        _ => {}, 
    }
}





/// for generating data 
fn cli_generate_data( ) -> Command {
    Command::new("gen")
        .about( "genereate data command")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("tanh")
            .about( "specify the parameters of data generation ")
            .arg(arg!(<OUTPUT> "output file name example: out.csv") )
            .arg_required_else_help(true)
            .arg(arg!(<PARAMS> "A string of parameters for example: \"70.,60.5,0.03,30,120,-0.03\"") )
            .arg_required_else_help(true)
            .arg(                     
                Arg::new( "tsteps" )
                .short( 't')
                .long("steps")
                .default_value("100")
                .value_parser(value_parser!(usize))
            )
            .arg( 
                Arg::new( "error" )
                .short( 'e')
                .long("error")
                .default_value("1.0")
                .value_parser(value_parser!(f64))                
            )
        )
}
/// cli specification for generating data
fn cli_model_fit( ) -> Command {
    Command::new("fit")
    .about( "use nonlinear least squares fitting for data against a specified class of models")
    //.arg_required_else_help(true)
    .arg(                     
        Arg::new( "humps" )
        .short( 'n')
        .long("humps")
        .default_value("3" )
        .value_parser(value_parser!(usize))
    )
    .arg( 
        Arg::new( "samples" )
        .short( 's')
        .long("samples")
        .default_value("1000")
        .value_parser(value_parser!(usize))                
    )
    .arg(
        Arg::new( "offset" )
        .short( 'o' )
        .long( "offset" )
        .default_value( "0")
        .value_parser( value_parser!( i64 ))
    )
    .arg( 
        Arg::new( "limit" )
        .short('l') 
        .long("limit")
        .default_value( "18446744073709551615")
        .value_parser( value_parser!( usize ))
    )
    .arg( 
        Arg::new( "strides" )
        .short('x')
        .long("strides")
        .default_value( "1")
        .value_parser( value_parser!( usize ))
    )
    .arg(
        Arg::new( "reports" )
        .short('r')
        .long("reports")
        .default_value( "1" )
        .value_parser( value_parser!(usize))
    )
    .arg(arg!(<OUTPUT> "A serialized model file"))
    .arg(arg!(<INPUT> "data to consider, .. should be a list of csv files with headers" ))
    .arg_required_else_help(true)
}

fn cli_model_viz( ) -> Command {
    Command::new("viz")
    .about( "Visualize the data and model(s)")
    //.arg_required_else_help(true)
    .subcommand_required(true)
    .arg_required_else_help(true)
    .allow_external_subcommands(true)
    .subcommand(
        Command::new("basic")
        .about( "construct basic plots.")
        .arg(                     
            Arg::new( "top" )
            .short( 't')
            .long_help("select this many models from the input model file" )
            .long("top")
            .default_value("1" )
            .value_parser(value_parser!(usize))
        ) 
        .arg(
            Arg::new( "offset" )
            .short( 'o' )
            .long( "offset" )
            .default_value( "0")
            .value_parser( value_parser!( i64 ))
        )
        .arg( 
            Arg::new( "limit" )
            .short('l') 
            .long("limit")
            .default_value( "18446744073709551615")
            .value_parser( value_parser!( usize ))
        )
        .arg( 
            Arg::new( "strides" )
            .short('x')
            .long("stides")
            .default_value( "1")
            .value_parser( value_parser!( usize ))
        )
        .arg(arg!(<OUTPUT> "A png file"))
        .arg(arg!(<DATA> "data to consider, .. should be a list of csv files with headers" ))
        .arg(arg!(<MODEL> "model file, .. such as that generated in the fit proceedure "))
        .arg_required_else_help(true)
    )
    .subcommand(
        Command::new("intermediate")
        .about( "construct intermediate plots.")
        .arg(                     
            Arg::new( "item" )
            .short( 'i')
            .long_help("select this model (by specified index) from the input model file" )
            .long("item")
            .default_value("0" )
            .value_parser(value_parser!(usize))
        ) 
        .arg(                     
            Arg::new( "pval" )
            .short( 'p')
            .long_help("select crossing thrashold [0,1] for shaded regions" )
            .long("prob")
            .default_value("0.02" )
            .value_parser(value_parser!(f64))
        ) 
        .arg(
            Arg::new( "offset" )
            .short( 'o' )
            .long( "offset" )
            .default_value( "0")
            .value_parser( value_parser!( i64 ))
        )
        .arg( 
            Arg::new( "limit" )
            .short('l') 
            .long("limit")
            .default_value( "18446744073709551615")
            .value_parser( value_parser!( usize ))
        )
        .arg( 
            Arg::new( "strides" )
            .short('x')
            .long("stides")
            .default_value( "1")
            .value_parser( value_parser!( usize ))
        )
        .arg(arg!(<OUTPUT> "A png file"))
        .arg(arg!(<DATA> "data to consider, .. should be a list of csv files with headers" ))
        .arg(arg!(<MODEL> "model file, .. such as that generated in the fit proceedure "))
        .arg_required_else_help(true)
    )
}

fn cli_model_exp( ) -> Command {
    Command::new("exp")
    .about( "explain the recovered model(s)")
    //.arg_required_else_help(true)
    .subcommand_required(true)
    .arg_required_else_help(true)
    .allow_external_subcommands(true)
    .subcommand(
        Command::new("basic")
        .about( "explain the basics of recovered models with basic depth.")
        .arg(                     
            Arg::new( "top" )
            .short( 't')
            .long_help("select this many models from the input model file" )
            .long("top")
            .default_value("1" )
            .value_parser(value_parser!(usize))
        ) 
        .arg(arg!(<MODEL> "model file, .. such as that generated in the fit proceedure "))
        .arg_required_else_help(true)
    )
    .subcommand(
        Command::new("intermediate")
        .about( "explain the recovered models with intermediate depth.")
        .arg(                     
            Arg::new( "top" )
            .short( 't')
            .long_help("select this many models from the input model file" )
            .long("top")
            .default_value("1" )
            .value_parser(value_parser!(usize))
        ) 
        .arg(                     
            Arg::new( "pval" )
            .short( 'p')
            .long_help("select crossing thrashold [0,1] for shaded regions" )
            .long("prob")
            .default_value("0.02" )
            .value_parser(value_parser!(f64))
        ) 
        .arg(arg!(<MODEL> "model file, .. such as that generated in the fit proceedure "))
        .arg_required_else_help(true)
    )
}

fn cli() -> Command {
    Command::new("top-level")
        .about("CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand( 
            cli_generate_data()
        )
        .subcommand(
            cli_model_fit()
        )
        .subcommand( 
            cli_model_viz()
        )
        .subcommand(
            cli_model_exp()
        )

}


fn print_help_mesg(){
    println!( " help message . ")
}