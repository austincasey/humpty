use ndarray::Array;
use plotly::layout::{Axis, GridPattern, Layout, LayoutGrid,  TicksDirection, BarMode, Legend, RowOrder, Shape, ShapeType, ShapeLine, ShapeLayer, Annotation};
use plotly::{ImageFormat, Scatter, Plot, Bar, color::{Rgb,  NamedColor, Rgba}};
use plotly::common::{
    ColorScale, ColorScalePalette, DashType, Fill, Font, Side, Line, LineShape, Marker, Mode, Title,
};
use plotly::box_plot::{BoxMean, BoxPoints};
use plotly::common::{ErrorData, ErrorType, Orientation};
use plotly::histogram::{Bins, Cumulative, HistFunc, HistNorm};
use plotly::layout::{ BoxMode, Margin};
use plotly::{ BoxPlot, Histogram};
use std::cmp::{Ordering, min, max};
use std::path::Path;
use std::ffi::{OsStr, OsString};

use crate::models::ParameterizedModel;
use crate::models::{ModelAffine::AffineAdditive, ModelConstant::ModelConstant, ModelAdditive::ModelAdditive, ModelTanh::ModelTanh};

fn data_bound_box( D : &Vec<f64> ) -> ( f64, f64 ){
    D.iter().fold((f64::INFINITY,  f64::NEG_INFINITY) , |a, &b| {
        (match PartialOrd::partial_cmp(&a.0, &b) {
            None => a.0, //default Ignore NAN f64::NAN,
            Some(Ordering::Less) => a.0,
            Some(_) => b,
        },
        match PartialOrd::partial_cmp(&a.1, &b) {
            Some(Ordering::Greater) => a.1,
            Some(_) => b,  
            None => a.1, // default Ignore NAN f64::NAN,
        } )
    })
}

pub enum PlotAction{
    PNG( String , usize, usize, f64),
    SHOW,
    HTML( String )
} 

pub fn write_html( filename: String , mut plot : Plot ){
    /// filename can either be a stub or full html. 
    let EXT : String = String::from("html");
    let fpath : &Path = Path::new( filename.as_str() );
    let mut fpath_ext = filename.clone(); 
    fpath_ext.push_str(EXT.as_str());
    match fpath.extension() {
        Some(v) => 
            match v.to_ascii_lowercase().into_string().unwrap().as_str() {
                "html" | "htm" => {
                    plot.write_html(fpath )
                }
                _ => { 
                    println!( "\twarning: filepath {} has extension other than {EXT}, appending with {EXT}", fpath.display());
                    plot.write_html( fpath_ext  )
                } 
            },
        None => plot.write_html( fpath_ext  ),
    }   
}

pub fn write_png( filename: String , mut plot : Plot , width : usize, height : usize , scale : f64 ){
    /// filename can either be a stub or full html. 
    let EXT : String = String::from("png");
    let fpath : &Path = Path::new( filename.as_str() );
    match fpath.extension() {
        Some(v) => 
            match v.to_ascii_lowercase().into_string().unwrap().as_str() {
                "png" => {
                    plot.write_image(fpath, ImageFormat::PNG, width, height, scale)
                }
                _ => { 
                    println!( "\twarning: filepath {} has extension other than {EXT}, appending with {EXT}", fpath.display()); 
                    plot.write_image(fpath, ImageFormat::PNG, width, height, scale)
                }
            },
        None => plot.write_image(fpath, ImageFormat::PNG, width, height, scale),
    }   
}

pub fn eval_M( M : &AffineAdditive<ModelTanh> , t: &Vec<f64> ) -> Vec<f64> {
    let mut rv : Vec<f64> = Vec::new(); 
    for tx in t {
        rv.push( M.eval( *tx ) );
    }
    rv  
}

fn check_residual( D : &Vec<f64> , M : &Vec<f64> ) -> ( f64, f64, Vec<f64>){
    let mut c = 0; 
    let Z = D.len() as f64; 
    let resid : Vec<f64> = D.iter().zip( M.iter() ).map( |(a,b)| { (a -b)*(a-b ) }).collect();
    let rsum = resid.iter().fold(0., |acc,x | acc + x );
    let rsumsq = libm::sqrt( rsum ); 
    ( rsumsq, rsum/Z,  resid )
}


/// given val in [0,1] representing the fraction of motion, we wish to find the crossing for 1+tanh(alpha*t+beta). 
pub fn invert_tanh( frac_motion : f64, alpha : f64 , beta : f64 ) -> f64{
    let val = 2.0*frac_motion -1.0  ; //domain is now [-1,1]
    let t_0 = -beta / alpha ; 
    t_0 + (1.0/(2.0*alpha))*(( 1.0 -val )/ ( 1.0 + val )).ln()
}


pub fn plot_data(mut p : Plot, t: &Vec<f64>, d:&Vec<f64> ) -> Plot {
    let trace1 = Scatter::new(t.clone(), d.clone())
                            .name("data")
                            .mode(Mode::Markers)
                            .marker(Marker::new().size(3) )
                            .x_axis( "time" )
                            .y_axis( "counts" )
                            .opacity(0.75);
    p.add_trace(trace1);
    p
}

pub fn plot_model( mut p : Plot,  t: &Vec<f64>, m: &Vec<f64> , opacity_value : f64, name : &str) -> Plot {
    let trace2 = Scatter::new(t.clone(), m.clone())
                            .mode(Mode::Lines)
                            .name(name) 
                            .opacity(opacity_value );


    p.add_trace(trace2);
    p
}


fn basic_histogram(mut p : Plot, data : Vec<f64>, name : &str  ) -> Plot {
    let trace = Histogram::new(data).name(name);
    p.add_trace(trace);
    p 
}  



fn add_vert_line(mut layout : Layout , x : f64  ) -> Layout {
    layout.add_shape(
        Shape::new()
            .x_ref("x")
            .y_ref("paper")
            .shape_type(ShapeType::Line)
            .x0(x)
            .y0(0)
            .x1(x)
            .y1(1)
            .line(ShapeLine::new().color(NamedColor::Pink).width(1.)),
    );
    layout
}

fn add_shaded_region( mut layout : Layout , x1 : f64, x2 : f64 )-> Layout {
    layout.add_shape(
        Shape::new()
            .x_ref("x")
            .y_ref("paper")
            .shape_type(ShapeType::Rect)
            .x0(x1)
            .y0(0)
            .x1(x2)
            .y1(1)
            .fill_color(NamedColor::Salmon)
            .opacity(0.25)
            .layer(ShapeLayer::Above)
            .line(ShapeLine::new().width(0.1)),
    );
    layout
}

pub fn plot_a( t : &Vec<f64>, d: &Vec<f64>, m : &Vec<(f64, Vec<f64>)>, act : PlotAction, title : String, x_label : String, y_label : String   ) {
    
    let mut plot = Plot::new();
    plot = plot_data(plot, t, d );
    plot = plot_model(plot, t, &m[0].1, 0.98 , "model");
    let layout = Layout::new()
                            .title(Title::new( title.as_str() ))
                            .x_axis(Axis::new().title(Title::new( x_label.as_str())))
                            .y_axis(Axis::new().title(Title::new( y_label.as_str())));
    plot.set_layout(layout);

    match act {
        PlotAction::PNG(filename, width, height, scale ) => {
            write_png(filename, plot, width, height, scale )
        },
        PlotAction::SHOW => {
            plot.show();
            println!("{}",plot.to_inline_html(Some("simple_subplot")));
        },
        PlotAction::HTML(filename) => { 
            write_html(filename, plot)
        },
    }

}

pub fn plot_c( t : &Vec<f64>, d: &Vec<f64>, tm: &Vec<f64>, m : &Vec<f64>, act : PlotAction, title : String, x_label : String, y_label : String   ) {
 
    let mut plot = Plot::new();
    let trace1 = Scatter::new(t.clone(), d.clone())
                            .name("data")
                            .mode(Mode::Markers)
                            .marker(Marker::new().size(3) )
                            .x_axis( "time" )
                            .y_axis( "counts" )
                            .opacity(0.75);

    let trace2 = Scatter::new(tm.clone(), m.clone())
                        .mode(Mode::Lines)
                        .name("model") 
                        .opacity(0.89 );


    let layout = Layout::new()/* .grid(
        LayoutGrid::new()
            .rows(2)
            .columns(1)
            .pattern(GridPattern::Independent),
        )*/
        .title(Title::new( title.as_str() ))
        .x_axis(Axis::new().title(Title::new( x_label.as_str())))
        .y_axis(Axis::new().title(Title::new( y_label.as_str())));

    plot.set_layout(layout);


    plot.add_trace(trace1);
    plot.add_trace(trace2);
    let mut plot2 = Plot::new();
    //plot.add_traces(vec![trace1, trace2]);

    let disp: Vec<f64> = d.iter().zip( m ).map( |(a,b)| a - b ).collect();
    let trace3 = Histogram::new(disp).name("displacments histo");

    let mut new_title = title.clone();
    new_title.push_str(" histo" );
    let layout = Layout::new()/* .grid(
        LayoutGrid::new()
            .rows(2)
            .columns(1)
            .pattern(GridPattern::Independent),
        )*/
        .title(Title::new( new_title.as_str() ))
        .x_axis(Axis::new().title(Title::new( "displacments" )))
        .y_axis(Axis::new().title(Title::new( "frequnecy")));

    plot2.set_layout(layout);
    plot2.add_trace(trace3);
    match act {
        PlotAction::PNG(filename, width, height, scale ) => {
            let mut new_filename = filename.clone();
            new_filename.push_str( "hist");
            write_png(filename, plot, width, height, scale );
            write_png(new_filename, plot2, width, height, scale );
        },
        PlotAction::SHOW => {
            plot.show();
            println!("{}",plot.to_inline_html(Some("simple_subplot")));
        },
        PlotAction::HTML(filename) => {
            let mut new_filename = filename.clone();
            new_filename.push_str( "hist");
            write_html(filename, plot);
            write_html(new_filename, plot2);
        },
    }

}

// This plot is designed to show bands around the model.
pub fn plot_d(  t : &Vec<f64>, 
                d : &Vec<f64>, 
                tm: &Vec<f64>,  // local graph 
                m : &Vec<f64>,  // local model 
                ttb : &Vec<f64>, 
                dtb : &Vec<Vec<f64>>,
                act : PlotAction, 
                title : String, 
                x_label : String, 
                y_label : String   ) {
 
    let mut plot = Plot::new();

    let regions = dtb.iter()
                                                .map(|d| 
                                                    { 
                                                        Scatter::new( ttb.clone(), d.clone() )
                                                            .fill( Fill::ToZeroX  )
                                                            .fill_color(Rgba::new(255, 192, 203,0.5))
                                                            .line(Line::new().color(Rgba::new(255, 192, 203,0.7)))
                                                            .name("band")
                                                            .show_legend(false)
                                                    } );

    let trace1 = Scatter::new(t.clone(), d.clone())
                            .name("data")
                            .mode(Mode::Markers)
                            .marker(Marker::new().size(3) )
                            .x_axis( "time" )
                            .y_axis( "counts" )
                            .opacity(0.75);

    let trace2 = Scatter::new(tm.clone(), m.clone())
                        .mode(Mode::Lines)
                        .name("model") 
                        .opacity(0.89 );


    let layout = Layout::new()/* .grid(
        LayoutGrid::new()
            .rows(2)
            .columns(1)
            .pattern(GridPattern::Independent),
        )*/
        .title(Title::new( title.as_str() ))
        .x_axis(Axis::new().title(Title::new( x_label.as_str())))
        .y_axis(Axis::new().title(Title::new( y_label.as_str())));

    plot.set_layout(layout);

    regions.for_each(|r|
        plot.add_trace(r)
    );
    plot.add_trace(trace1);
    plot.add_trace(trace2);
    let mut plot2 = Plot::new();
    //plot.add_traces(vec![trace1, trace2]);

    ////////////////////////////////////////////////////////////////////////
    ///
    ///  Part 2.  plot the residual
    /// 
    let disp: Vec<f64> = d.iter().zip( m ).map( |(a,b)| a - b ).collect();

    let trace3 = Histogram::new(disp).name("displacments histo");

    let mut new_title = title.clone();
    new_title.push_str(" histo" );
    let layout = Layout::new()/* .grid(
        LayoutGrid::new()
            .rows(2)
            .columns(1)
            .pattern(GridPattern::Independent),
        )*/
        .title(Title::new( new_title.as_str() ))
        .x_axis(Axis::new().title(Title::new( "displacments" )))
        .y_axis(Axis::new().title(Title::new( "frequnecy")));

    plot2.set_layout(layout);
    plot2.add_trace(trace3);
    match act {
        PlotAction::PNG(filename, width, height, scale ) => {
            let mut new_filename = filename.clone();
            new_filename.push_str( "hist");
            write_png(filename, plot, width, height, scale );
            write_png(new_filename, plot2, width, height, scale );
        },
        PlotAction::SHOW => {
            plot.show();
            println!("{}",plot.to_inline_html(Some("simple_subplot")));
        },
        PlotAction::HTML(filename) => {
            let mut new_filename = filename.clone();
            new_filename.push_str( "hist");
            write_html(filename, plot);
            write_html(new_filename, plot2);
        },
    }

}



pub fn plot_model_with_dates( 
    t : &Vec<f64>, 
    ts : &Vec<String>, 
    d: &Vec<f64>, 
    AAM: AffineAdditive<ModelTanh>,
    p: f64, 
    act : PlotAction, 
    title : String, 
    x_label : String, 
    y_label : String   ) 
    {
        let mut plot:Plot = Plot::new(); 
        let MA: ModelAdditive<ModelTanh> = AAM.tm.clone() ; 
        let YC: ModelConstant = AAM.km.clone();
        let mx = eval_M(&AAM, &t.to_vec()); 

        let LSM = check_residual(&d, &mx.clone()).0;
        
        let (dmin, dmax ) = data_bound_box(d);
        let (tmin, tmax ) = data_bound_box(t);
        let DT : f64 = tmax - tmin ; 
        let DD : f64 = dmax - dmin ;

        let trace1 = Scatter::new(ts.clone(), d.clone())
            .name("data")
            .mode(Mode::Markers)
            .marker(Marker::new().size(3) )
            .x_axis( "x" )
            .y_axis( "counts" )
            .opacity(0.75);

        plot.add_trace(trace1);

        let trace2 = Scatter::new(ts.clone(), mx.clone())
            .mode(Mode::Lines)
            .name("model")   
            .x_axis( "x" )
            .opacity(0.98 );
        
        plot.add_trace(trace2);
  
        let mut layout = Layout::new()
            .title(Title::new( title.as_str() ))
            .x_axis(Axis::new().title(Title::new( x_label.as_str())))
            //.x_axis2(Axis::new().overlaying("x"))
            .y_axis(Axis::new().title(Title::new( y_label.as_str())));
        let mut ann_anchor : Vec<Annotation> = Vec::new(); 
  
        for tm in MA.components {
            let a = tm.alpha; 
            let b  = tm.beta; 
            let t0 = -b/a; 
            layout = add_vert_line( layout, t0 ); 
      
            let tt1 = invert_tanh( p, a, b  ); 
            let tt2 = invert_tanh(1.0 - p , a, b);
            let ttt1 = if ( tt1 > tt2 ) { tt2 } else {tt1 };
            let ttt2 = if (tt1 > tt2 ) {tt1 } else {tt2}; 
            println!( " shading regio {} {}", tt1, tt2);
            layout = add_shaded_region(layout, ttt1, ttt2);
    
            let mx2 = eval_M(&AAM, &vec![ ttt1, ttt2, t0] );
            let mtt1 : f64 = mx2[0];
            let mtt2 : f64 = mx2[1];
            let mt0 = mx2[2];
            let wdith = 800; 
    
            let mut hump_anchors = add_model_hump_annotate(t0 , b, p, ttt1, ttt2, mt0, mtt1, mtt2, (tmin, tmax), (dmin, dmax)  );
            ann_anchor.append(&mut hump_anchors )
        } 
        layout = layout.annotations(ann_anchor);

        plot.set_layout(layout);

        match act {
            PlotAction::PNG(filename, width, height, scale ) => {
                write_png(filename, plot, width, height, scale )
            },
            PlotAction::SHOW => {
                plot.show();
                println!("{}",plot.to_inline_html(Some("simple_subplot")));
            },
            PlotAction::HTML(filename) => {
                write_html(filename, plot)
            },
        }
    }

pub fn add_model_hump_annotate(t0 : f64, b: f64, p: f64, tt1: f64, tt2: f64, mt0: f64, mtt1 : f64, mtt2 :f64, DT: (f64, f64), DD : (f64,f64) ) -> Vec<Annotation> {
    let mut ann_anchor : Vec<Annotation> = Vec::new(); 
    let low_1 = DD.0 ; 
    let low_2 = DD.0 + (DD.1 - DD.0 )*0.1; 
    let low_3 = DD.0 + (DD.1 - DD.0 )*0.2; 
    let hi_3 = DD.1 - (DD.1 - DD.0 )*0.2;
    let hi_2 = DD.1 - (DD.1 - DD.0 )*0.1;
    let hi_1 = DD.1;

    let td1 = (DT.1 - DT.0)*0.1;
    let td2 = (DT.1 - DT.0)*0.2;
    let td3 = (DT.1 - DT.0)*0.3;
    let deltaD = DD.1 - DD.0;
    let detlaT = DT.1 - DT.0; 

    let angle = -90.0;// ((td1*(DD.1-DD.0)/(DT.1-DT.0))).atan2( td1 ) * ( 180.0 / std::f64::consts::PI ) ; 
    //println!( "DD={}, DT={td1}, 180./PI = {} = {angle}", td1*(DD.1-DD.0)/(DT.1-DT.0), 180.0/std::f64::consts::PI );
    let td1: f64 = 0.0 ; 

    let (vpos1, vpos2)  = if mt0 > DD.0 + deltaD*0.5  { (low_1, low_2)  } else { (hi_1, hi_2) };
    ann_anchor.push( Annotation::new()  
                .ax_ref("x")
                .ay_ref("y") 
                .x(t0) 
                .y(vpos2 - (td1*(DD.1-DD.0)/(DT.1-DT.0)))
                .show_arrow(true)
                .arrow_head(1)
                .arrow_size(1.0)
                .arrow_width(1.0)
                .ax(t0 + td1 )
                .ay(vpos2 )
                .text(format!("{:.1} γ:{:.3}", t0, b).to_string() )
                .text_angle(angle) 
                .align(plotly::layout::HAlign::Right));
            if ( tt2 - tt1 ) > (DT.1 - DT.0)*0.01 {
                ann_anchor.push( Annotation::new()// place arrow to tt1 from t0
                        .ax_ref("x")
                        .ay_ref("y")
                        .x(tt1)
                        .y(vpos1)  
                        .show_arrow(true)
                        .arrow_head(1)
                        .arrow_size(1.0)
                        .arrow_width(1.0)
                        .ax(t0)
                        .ay(vpos1)
                        .text( format!("{:.2}", 1.-p).to_string()  )
                        .text_angle(0.0));
                ann_anchor.push( Annotation::new()// place arrow to tt2 from t0
                        .x(tt2)
                        .y(vpos1)
                        .show_arrow(true)
                        .arrow_head(1)
                        .arrow_size(1.0)
                        .arrow_width(1.0)
                        .ax_ref("x")
                        .ay_ref("y")
                        .ax(t0)
                        .ay(vpos1)
                        .text( format!("{:.2}",1.- p).to_string()  )
                        .text_angle(0.0));
                ann_anchor.push( Annotation::new() // label tt1
                        .x(tt1)
                        .y(vpos2)
                        .show_arrow(true)
                        .arrow_head(1)
                        .arrow_size(1.0)
                        .arrow_width(1.0)
                        .ax_ref("x")
                        .ay_ref("y")
                        .ax(tt1+td1)
                        .ay(vpos2)
                        .text( format!("{:.1}",tt1).to_string()  )
                        .text_angle(0.0)
                        .align(plotly::layout::HAlign::Right));
                ann_anchor.push( Annotation::new() // label tt2 
                        .x(tt2)
                        .y(vpos2)  
                        .show_arrow(true)
                        .arrow_head(1)
                        .arrow_size(1.0)
                        .arrow_width(1.0)
                        .ax_ref("x")
                        .ay_ref("y")
                        .ax(tt2+td1)
                        .ay(vpos2)
                        .text( format!("{:.1}",tt2).to_string()  )
                        .text_angle(0.0)
                        .align(plotly::layout::HAlign::Right));
            }
            ann_anchor
        }

pub fn plot_model_with_markers( 
    t : &Vec<f64>,  // time series entire time domain
    d: &Vec<f64>,   // data series entire
    tm: &Vec<f64>,  // model time domain  
    mv: &Vec<f64>,  // mode value sequence
    AAM: AffineAdditive<ModelTanh>, 
    p: f64, 
    act : PlotAction, 
    title : String, 
    x_label : String, 
    y_label : String   ) 
    {
        // Here we plot the model but also the midpoints as well as time crossing for p fraction of motion.
        let mut plot = Plot::new();
    
        let MA: ModelAdditive<ModelTanh> = AAM.tm.clone() ; 
        let YC: ModelConstant = AAM.km.clone();
        
        let tm_nom = (tm.len()-1) as f64; 
        let tm_first = * tm.first().expect( "tm sequence should be non empty");
        let tm_last = * tm.last().expect("tm sequence should be non empty");
        //ERROR BELOW should be 0..len(tm) instead of tm

        let tpredict : Vec<f64> = (0..).take( * t.last().expect("") as usize ).filter(|x| { ( *x > (* tm.last().expect("") as i32) )  }).map(|x|{x as f64}).collect();
        //((* tm.last().expect("") as usize )..((* t.last().expect("") - * tm.last().expect("")) as usize )).map( |x| x as f64 ).collect();
        println!( " TPREDICT\n: {:?}", tpredict.clone());
        let mpredict = eval_M(&AAM, &tpredict.to_vec()); 
        let LSM = check_residual(&d, &mv.clone()).0;
     
        let (dmin, dmax ) = data_bound_box(d);
        let (tmin, tmax ) = data_bound_box(t);
        let DT : f64 = tmax - tmin ; 
        let DD : f64 = dmax - dmin ;
    
        //plot = plot_data(plot, t, d );
        //plot = plot_model(plot, tm, &mx, 0.98 , "model");

        let trace1 = Scatter::new(t.clone(), d.clone())
                                .name("data")
                                .mode(Mode::Markers)
                                .marker(Marker::new().size(3) )
                                .x_axis( "time" )
                                .y_axis( "counts" )
                                .opacity(0.75);
        plot.add_trace(trace1);

        let trace2 = Scatter::new(tm.clone(), mv.clone())
                            .mode(Mode::Lines)
                            .name("model") 
                            .opacity(0.89 );

        plot.add_trace(trace2);

        let trace3 = Scatter::new( tpredict.clone(), mpredict.clone()  )
            .name("prediction")
            .mode(Mode::LinesMarkers)
            .line(Line::new().dash(DashType::DashDot))
            .opacity( 0.8); 

        plot.add_trace(trace3);
        let mut layout = Layout::new()
                                .title(Title::new( title.as_str() ))
                                .x_axis(Axis::new().title(Title::new( x_label.as_str())).range(vec![t[0], t[t.len()-1]]))
                                .y_axis(Axis::new().title(Title::new( y_label.as_str())));
    
        let mut anchor: Vec<f64> = Vec::new() ; 
        let mut atextx: Vec<String> = Vec::new() ; 
        let mut ann_anchor : Vec<Annotation> = Vec::new(); 
        
        let translate = |t:f64 | {  tm_first + t * ( tm_last - tm_first )/tm_nom };



        for tm in MA.components {
            let a = tm.alpha; 
            let b  = tm.beta; 
            let t0 = -b/a; 
            layout = add_vert_line( layout, translate( t0 ) ); 
            let shaded_region = false; 
            if shaded_region { 
            // anchor.push( t0 ); 
            // atextx.push( String::from(format!("Ø {:.2} γ {:.4}", t0, b).to_string()));
            let tt1 = invert_tanh(p, a, b ); 
            let tt2 = invert_tanh(1.0 - p , a, b);
            let ttt1 = if ( tt1 > tt2 ) { tt2 } else {tt1 };
            let ttt2 = if (tt1 > tt2 ) {tt1 } else {tt2}; 
            println!( " shading regio {} {}", translate( ttt1 ), translate( ttt2 ));
            layout = add_shaded_region(layout, translate( ttt1 ), translate( ttt2 ));
    
            let mx2 = eval_M(&AAM, &vec![ ttt1, ttt2, t0] );
            let mtt1 : f64 = mx2[0];
            let mtt2 : f64 = mx2[1];
            let mt0 = mx2[2];
            let wdith = 800; 
    
            let mut hump_anchors = add_model_hump_annotate(t0 , b, p, translate( ttt1 ), translate( ttt2 ), mt0, mtt1, mtt2, (translate( tmin ), translate( tmax)), (dmin, dmax)  );
            ann_anchor.append(&mut hump_anchors )
            }
        } ;
        layout = layout.annotations(ann_anchor);
        // let trace = Scatter::new( anchor.clone(), anchor.iter().map( |x| 0.0 ).collect() )
        //         .text_array( atextx.iter().map(|x| x.as_str() ).collect() )
        //         .mode(Mode::Text)
        // plot.add_trace(trace);
    
        plot.set_layout(layout);
    
        match act {
            PlotAction::PNG(filename, width, height, scale ) => {
                write_png(filename, plot, width, height, scale )
            },
            PlotAction::SHOW => {
                plot.show();
                println!("{}",plot.to_inline_html(Some("simple_subplot")));
            },
            PlotAction::HTML(filename) => {
                write_html(filename, plot)
            },
        }
    
    }
