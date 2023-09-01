use clap::{Command, Arg, value_parser, arg, ArgGroup, Subcommand};
use humpty2::main_mod_data_gen::{build_tanh_model_from_string, write_data};
use std::path::{Path, PathBuf};

fn main() {
    let m: clap::ArgMatches = cli().get_matches();

    match m.subcommand() {
        // MODULE 1 create the HDC Brain Architecture 
        Some(("generate", m)) => { 
            match m.subcommand(){
                Some(("tanh", m )) => 
                {
                    let params : String = m.get_one::<String>("PARAMS").expect("paramters are required").clone();
                    let error : f64 = m.get_one::<f64>("error" ).expect("parsing error issue").clone();
                    let time_steps : usize = m.get_one::<usize>( "tsteps").expect("parsing steps issue").clone();
                    let output  = Path::new( m.get_one::<String>("OUTPUT" ).expect( "output file name needed.").into() );
                    println!( " here with params = {params} , error ={error} steps={time_steps} output={:?}", &output );
                    // TODO ... lets improve upon this interface ...
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
            println!( " fitting model ({input}, {output}, {humps}, {samples})")
        },
        _ => {},
    }
}



/// for generating data 
fn cli_generate_data( ) -> Command {
    Command::new("generate")
        .about( "genereate data command")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("tanh")
            .about( "specify the parameters of data generation ")
            .arg(arg!(<PARAMS> "A string of parameters for example: \"70.,60.5,0.03,30,120,-0.03\"") )
            .arg_required_else_help(true)
            .arg(arg!(<OUTPUT> "output file name example: out.csv") )
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
    .subcommand_required(true)
    .arg_required_else_help(true)
    .allow_external_subcommands(true)
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
    .arg(arg!(<OUTPUT> "A serialized model file"))
    .arg(arg!(<INPUT> "data to consider, .. should be a list of csv files with headers" ))
    .arg_required_else_help(true)
}

fn cli() -> Command {
    Command::new("top-level")
        .about("CLI")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand( // MODULE 1 : create HDC Brain Architecture
            cli_generate_data()
        )
        .subcommand(
            cli_model_fit()
        )

}


fn print_help_mesg(){
    println!( " help message . ")
}