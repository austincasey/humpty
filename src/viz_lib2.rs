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
use serde::{Serialize, Deserialize};


#[derive(Debug)]
pub struct line_style {
    color : NamedColor, 
    dtype : DashType,
    opacity : f64,
    width : f64,
    size : usize, 
}

impl line_style {
    pub fn color(&self) -> NamedColor {
        self.color
    }
    pub fn dtype(&self) -> &DashType {
        &self.dtype
    }
    pub fn opacity(&self) -> f64 {
        self.opacity
    }
    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

#[derive(Debug)]
pub struct graph_style {
    data : line_style,
    model : line_style,
    prediction : line_style, 
    postdiction : line_style 
}

impl graph_style {
    pub fn default_graph_style( ) -> graph_style{
        graph_style { 
            data : line_style{color:NamedColor::Blue, dtype:DashType::Dot, opacity:1.0, width:3.0 ,size: 3 },
            model : line_style{color:NamedColor::Orange, dtype:DashType::Solid, opacity:1.0, width:1.0, size: 1  },
            prediction : line_style{color:NamedColor::OrangeRed, dtype:DashType::DashDot, opacity:1.0, width:1.0 , size:1 },
            postdiction : line_style{color:NamedColor::OrangeRed, dtype:DashType::LongDashDot, opacity:1.0, width:1.0 , size:1 },
        }
    }
    pub fn data( &self ) -> &line_style {
        &self.data 
    }
    pub fn model(&self) -> &line_style {
        &self.model
    }
    pub fn prediction(&self) -> &line_style {
        &self.prediction
    }
    pub fn postdiction(&self) -> &line_style {
        &self.postdiction
    }
}


//////////////////////////////////////////
/// 
/// 
/// 
/// 
/// 


fn data_bound_box( D : &Vec<f64> ) -> ( f64, f64 ){ // returns min and max for data vector D
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
/////////////////////////////////////////////////////////
/// 
/// 

pub enum PlotAction{
    PNG( String , usize, usize, f64),
    SHOW,
    HTML( String )
} 

pub struct viz_graph {
    pub plot : Plot, //  = Plot::new();
    pub style : graph_style,
}

impl viz_graph {
    pub fn new( ) -> viz_graph{
        viz_graph{plot:Plot::new(), style: graph_style::default_graph_style() }
    }
    pub fn finalize( self, act : PlotAction ){
        match act {
            PlotAction::PNG(filename, width, height, scale ) => {
                write_png(filename, self.plot, width, height, scale );
            },
            PlotAction::SHOW => {
                self.plot.show();
                println!("{}",self.plot.to_inline_html(Some("simple_subplot")));
            },
            PlotAction::HTML(filename) => {
                write_html(filename, self.plot);
            },
        }
    }
}

//////////////////////////////////////////////////////////////////////////////
/// 
/// layout augmentation 
/// 
/// 

pub fn add_vert_line_(mut layout : Layout , x : f64, color : NamedColor, width : f64 ) -> Layout {
    layout.add_shape(
        Shape::new()
            .x_ref("x")
            .y_ref("paper")
            .shape_type(ShapeType::Line)
            .x0(x)
            .y0(0)
            .x1(x)
            .y1(1)
            .line(ShapeLine::new().color(color).width(width)),
    );
    layout
}
pub fn add_vert_line( mut layout : Layout , x : f64 ) -> Layout {
    add_vert_line_( layout, x, NamedColor::Pink, 1.0 )
}

pub fn add_vert_shaded_region_( mut layout : Layout , x1 : f64, x2 : f64, color :NamedColor, width :f64 , opacity : f64 )-> Layout {
    layout.add_shape(
        Shape::new()
            .x_ref("x")
            .y_ref("paper")
            .shape_type(ShapeType::Rect)
            .x0(x1)
            .y0(0)
            .x1(x2)
            .y1(1)
            .fill_color(color)
            .opacity(opacity )
            .layer(ShapeLayer::Above)
            .line(ShapeLine::new().width(width)),
    );
    layout
}
pub fn add_vert_shaded_region( mut layout : Layout, x1: f64, x2: f64 ) -> Layout{
    add_vert_shaded_region_(layout , x1 , x2 , NamedColor::Salmon , 0.1 , 0.25 )
}


////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////
/// 
/// 
/// plot augmentation 
/// 
/// 

pub fn plot_trace_(mut p : Plot, t: &Vec<f64>, d:&Vec<f64>, markmode: Mode , marksize: usize, name: String, color: NamedColor , opacity: f64  ) -> Plot {
    let trace1 = Scatter::new(t.clone(), d.clone())
        .mode(markmode)
        .marker(Marker::new().size(marksize ).color(color) 
        )
        .name(name)
        .opacity(opacity);
    p.add_trace(trace1);
    p
}

pub fn plot_data(mut p : Plot, t: &Vec<f64>, d:&Vec<f64>, G: &graph_style  ) -> Plot {
    let G = graph_style::default_graph_style( ) ; // TODO : replaced with viz_graph 
    plot_trace_( p , t, d , Mode::Markers, G.data().size() as usize, "data".into(), G.data().color(), G.data().opacity() )
}

pub fn plot_model( mut p : Plot,  t: &Vec<f64>, m: &Vec<f64> , name : &str , G: &graph_style ) -> Plot {
    // TODO crossover using the viz_graph.graph_style
    plot_trace_( p , t, m , Mode::Lines, 3, name.into(), G.model().color(), G.model().opacity()  )
}

pub fn plot_projection( mut p : Plot, t: &Vec<f64>, m: &Vec<f64>, opacity_value: f64, name : &str , d : i8, G: &graph_style )-> Plot {
    let ltype =  if d == 0 { Line::new().dash(DashType::Dot) } else { Line::new().dash(DashType::DashDot )}; 
    let trace1 = Scatter::new(t.clone(), m.clone())
        .mode(Mode::LinesMarkers)
        //.marker(Marker::new().size(marksize ) )
        .name(name)
        .line(ltype) 
        .marker(
            Marker::new()
            .color( G.prediction().color() ) // Rgb::new(225, 205, 144))
            .size( G.prediction().size() ) 
        );
    p.add_trace(trace1);
    p
}

pub fn envelope_curve( hi : &Vec<f64>, lo : &Vec<f64> , disp : &Vec<f64> ) -> (Vec<f64> , Vec<f64> ){
    // given hi uppers envelope about zero (ex variance), and lo lower envelop about zero, make then hug curve disp.
    let hi_disp = hi.iter().enumerate().map( |(j,x)| { disp[j].max( disp[j] + *x  )}).collect();
    let lo_disp = lo.iter().enumerate().map( |(j,x)| { disp[j].min( disp[j] + *x  )}).collect();
    ( hi_disp, lo_disp )
}

pub fn add_shaded_region_between_two_curves_( mut p : Plot, t: &Vec<f64>, hi: &Vec<f64>, lo: &Vec<f64> , fillcolor: Rgba, linecolor: Rgba, region_name: String ,  bool_legend : bool) -> Plot 
{
    let mut time_there_and_back: Vec<f64> = t.clone();
    let mut time_back : Vec<f64> = t.clone().iter().rev().map(|x|{*x}).collect();
    time_there_and_back.append( &mut time_back ) ;
    
    let mut data_there_and_back :Vec<f64> = hi.iter().map(|x| { *x }).collect(); 
    let mut data_back = lo.iter().rev().map( | x| {*x}).collect();
    data_there_and_back.append( &mut data_back );

    let region = Scatter::new( time_there_and_back, data_there_and_back )
                            .fill( Fill::ToZeroX  )
                            .fill_color(fillcolor)
                            .line(Line::new().color(linecolor))
                            .name(region_name)
                            .show_legend(bool_legend);

    p.add_trace(region);
    p
}

pub fn add_shaded_region_between_two_curves( mut p : Plot, t: &Vec<f64>, hi: &Vec<f64>, lo: &Vec<f64> ) -> Plot{
    add_shaded_region_between_two_curves_( p , t , hi  , lo , Rgba::new(255, 192, 203,0.5), Rgba::new(255, 192, 203,0.7), String::from("band") , false )
}
/////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////
/// 
/// 
/// 

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

/////////////////////////////////////////////////////////////////////////////
/// 
/// 
/// 

pub fn plot_c( 
    t : &Vec<f64>,       d: &Vec<f64>, 
    tm: &Vec<f64>,       m : &Vec<f64>,
    tpre: &Vec<f64>,     mpre: &Vec<f64>,
    tpost: &Vec<f64>,    mpost: &Vec<f64>,
    act : PlotAction, title : String, x_label : String, y_label : String   ) {
    let mut view = viz_graph::new();
    let layout = Layout::new()
        .title(Title::new( title.as_str() ))
        .x_axis(Axis::new().title(Title::new( x_label.as_str())))
        .y_axis(Axis::new().title(Title::new( y_label.as_str())));
    view.plot.set_layout(layout);
    view.plot = plot_data( view.plot, t, d , &view.style); 
    view.plot = plot_model(view.plot, tm, m,  "model" , &view.style);
    view.plot = plot_projection( view.plot, tpre, mpre, 0.9 , "postdictio", 1 , &view.style);
    view.plot = plot_projection( view.plot, tpost, mpost, 0.9, "prediction", 0, &view.style );
    view.finalize( act );
}

pub fn plot_c_histo( t : &Vec<f64>, d: &Vec<f64>, tm: &Vec<f64>, m : &Vec<f64>, act : PlotAction, title : String, x_label : String, y_label : String   ) {
    let mut view = viz_graph::new();
    let disp: Vec<f64> = d.iter().zip( m ).map( |(a,b)| a - b ).collect();
    let trace3 = Histogram::new(disp).name("displacments histo");
    let mut new_title = title.clone();
    new_title.push_str(" histo" );
    let layout = Layout::new()
        .title(Title::new( new_title.as_str() ))
        .x_axis(Axis::new().title(Title::new( "displacments" )))
        .y_axis(Axis::new().title(Title::new( "frequnecy")));

    view.plot.set_layout(layout);
    view.plot.add_trace(trace3);
    view.finalize( act );
}

// This plot is designed to show bands around the model.
pub fn plot_d(  t : &Vec<f64>, 
                d : &Vec<f64>, 
                tm: &Vec<f64>,  // local graph 
                m : &Vec<f64>,  // local model 
                tpre: &Vec<f64>,     mpre: &Vec<f64>,
                tpost: &Vec<f64>,    mpost: &Vec<f64>,
                ttb : &Vec<f64>, 
                dtb : &Vec<Vec<f64>>,
                act : PlotAction, 
                title : String, 
                x_label : String, 
                y_label : String   ) {
    let mut view = viz_graph::new();
    let layout = Layout::new()
                .title(Title::new( title.as_str() ))
                .x_axis(Axis::new().title(Title::new( x_label.as_str())))
                .y_axis(Axis::new().title(Title::new( y_label.as_str())));
    view.plot.set_layout(layout);
     
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

    regions.for_each(|r|
        view.plot.add_trace(r)
    );
    view.plot = plot_data( view.plot, t, d , &view.style ); 
    view.plot = plot_model(view.plot, tm, m,  "model" , &view.style );
    view.plot = plot_projection( view.plot, tpre, mpre, 0.9 , "postdictio", 1, &view.style );
    view.plot = plot_projection( view.plot, tpost, mpost, 0.9, "prediction", 0 , &view.style );
    view.finalize( act );
}

/////////////////////
/// simple plot to plot disp and skew
/// 
pub fn plot_e(  t : &Vec<f64>, 
                d : &Vec<f64>, 
                act : PlotAction, 
                title : String, 
                x_label : String, 
                y_label : String   ) {
    let mut view = viz_graph::new();
    let layout = Layout::new()
                .title(Title::new( title.as_str() ))
                .x_axis(Axis::new().title(Title::new( x_label.as_str())))
                .y_axis(Axis::new().title(Title::new( y_label.as_str())));
    view.plot.set_layout(layout);

    view.plot = plot_data( view.plot, t, d , &view.style ); 
    view.finalize( act );
}

// This plot is designed to show bands around the model.
pub fn plot_d_histo(  t : &Vec<f64>, 
                d : &Vec<f64>, 
                tm: &Vec<f64>,  // local graph 
                m : &Vec<f64>,  // local model 
                ttb : &Vec<f64>, 
                dtb : &Vec<Vec<f64>>,
                act : PlotAction, 
                title : String, 
                x_label : String, 
                y_label : String   ) {
 
    let mut plot2 = Plot::new();
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
            write_png(new_filename, plot2, width, height, scale );
        },
        PlotAction::SHOW => {
            plot2.show();
            println!("{}",plot2.to_inline_html(Some("simple_subplot")));
        },
        PlotAction::HTML(filename) => {
            let mut new_filename = filename.clone();
            new_filename.push_str( "hist");
            write_html(new_filename, plot2);
        },
    }
    
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
            layout = add_vert_shaded_region(layout, translate( ttt1 ), translate( ttt2 ));
    
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

////////////////////////////////////////////////////////
/// aux functions 
/// 
/// 
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