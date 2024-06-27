use kernel_density_estimation::prelude::*;

use plotly::common::Mode;
use plotly::{Plot, Scatter};



#[test]
fn check() {
    // Create a distribution.
    let observationsa: Vec<f32> = vec![
        87.71199143429587,0.1627164660895907,-22.32224174541609,-152.7204213431422,-207.04795132368568,46.515130305094054,23.539716026178795,6.966122146574904,31.372907437348022,9.39586939375156,9.584433474165765,9.715942895421962,10.731117880218214,11.424437743839269,12.130103572710633,12.742892044160929,13.352033159968187,14.157519961068981,15.341518857649037,16.480715611411668,17.748754319356596,21.263114270240393,224.61451546879297,220.80616646974516,219.41010069797545,217.21039416231852,12.978894025497327,13.077403147309504,12.602595255065758,12.235278108694354,12.121031591182122,12.074029028596275,12.028498009853237,11.95794376909597,19.04150784676607,18.90572683455861,18.74310814224482,24.82732903200831,24.740201377100913,24.669458185903316,30.157608745260266,31.799701156163326,33.205632763019885,25.380886580828648,25.41027946383864,25.323100138551933,25.284389650398545,25.368419459386768,24.775514828915913,24.785077914070918,25.238696512441813,25.30921317157617,18.609933075838118,18.549316029332086,25.00795046489019,25.004550776626104,25.01425418244108,18.522110778311845,18.55970778341588,18.60268150701258,25.118451839023408,25.11625955115872,25.115998523167924,25.12083365573608,25.12960766235553,25.135778246661655,25.13641657311932,25.131024199872808,25.116126005388697,25.0945694655879,25.0754951387174,25.050752469130632,25.02274452741557,24.995310493452134,24.967325284760502,24.950899299324828,24.965980170898586,25.05082770322127,25.28338066199672,25.394222284086485,25.365613231004904,25.316539028685526,25.27317759971246,24.915868428536157,25.532316398674375,25.538197984558924,37.62686912137646,24.88653440758908,25.53273661498462,25.530509939665617,25.500959390779865,24.873111538246768,25.512764785618003,25.52344301988544,25.523069645026283,25.521815903944386,25.34652153730161,25.3392885233432,25.33980437625797,25.53829083544313,25.555711690858313,25.35740452177786,25.214022533836864,25.232122919721128,25.22215354823627,24.962149252096495,24.958280037764062,24.957130030914005,25.52248683338987,25.522412101229776,25.515681472319553,24.945900095285943,25.337903565072168,11.711767940148405,24.93846015242302,25.34183739650824,25.16655529565032,25.342028725395465,24.92754636968004,25.340754832448617,25.339461611241113,25.208550106451412,25.45774791060632,10.005056084992619,25.21934469347848,11.707006002302288,18.477901379277892,11.702524568054914,11.702514088983273,25.212082440209546,25.348123043722094,32.12704523102795,24.72546374116435,25.515710983980522,29.465435455247086,-2.4632777289343384,25.402571993144797,11.703354769436105,11.703529892258501,37.228950577891645,0.9758634943178109,11.703711914832642,25.50418436220595,25.34760647786804,11.703462918477806,25.346664203240422,0.27452953389229834,-0.015242083296324174,11.702041768689435,25.515682155667424,-417.120050692583,18.001818270601486,-0.40765747229015326,-924.4088665292207,-7.343368164981593,-0.34726361507271103,18.39671364631854,-41.250216468115305,2.345165999543743,-192.25602686246947,15.79413009419075,48.38008545403212,3.0403905991157814
    ];
    
    let observations :Vec<f32> = observationsa.into_iter().filter(|x|{ ( *x > 0.0 ) && (*x<30.0) } ).collect();
    // map( |x| {x.max(0.0).min(50.0) }).collect();
   
    // Observations are plotted as points along the X axis.
    let x1 = observations.clone();
    let y1 = vec![0.0; observations.len()];

    // Each KDE uses a bandwidth of 1.
    let bandwidth = Box::new(|_: &[f32]| 1.0);

    // Initialize a KDE for each kernel type.
    let kde1 = KernelDensityEstimator::new(observations.clone(), &bandwidth, Epanechnikov);
    let kde2 = KernelDensityEstimator::new(observations.clone(), &bandwidth, Normal);
    let kde3 = KernelDensityEstimator::new(observations.clone(), &bandwidth, Uniform);
    let kde4 = KernelDensityEstimator::new(observations.clone(), &bandwidth, Triangular);
    let kde5 = KernelDensityEstimator::new(observations.clone(), &bandwidth, Quartic);
    let kde6 = KernelDensityEstimator::new(observations.clone(), &bandwidth, Triweight);
    let kde7 = KernelDensityEstimator::new(observations.clone(), &bandwidth, Tricube);
    let kde8 = KernelDensityEstimator::new(observations.clone(), &bandwidth, Cosine);
    let kde9 = KernelDensityEstimator::new(observations.clone(), &bandwidth, Logistic);
    let kde10 = KernelDensityEstimator::new(observations.clone(), &bandwidth, Sigmoid);
    let kde11 = KernelDensityEstimator::new(observations.clone(), &bandwidth, SilvermanKernel);

    // Create a grid of points to evaluate each KDE on.
    let dataset: Vec<f32> = (0..101)
        .into_iter()
        .map(|x| x as f32 * (30. / 100.))
        .collect();

    // Evaluate PDFs.
    let x2 = dataset.clone();
    let y2 = kde1.pdf(&dataset);
    let x3 = dataset.clone();
    let y3 = kde2.pdf(&dataset);
    let x4 = dataset.clone();
    let y4 = kde3.pdf(&dataset);
    let x5 = dataset.clone();
    let y5 = kde4.pdf(&dataset);
    let x6 = dataset.clone();
    let y6 = kde5.pdf(&dataset);
    let x7 = dataset.clone();
    let y7 = kde6.pdf(&dataset);
    let x8 = dataset.clone();
    let y8 = kde7.pdf(&dataset);
    let x9 = dataset.clone();
    let y9 = kde8.pdf(&dataset);

    let x10 = dataset.clone();
    let y10 = kde9.pdf(&dataset);
    let x11 = dataset.clone();
    let y11 = kde10.pdf(&dataset);
    let x12 = dataset.clone();
    let y12 = kde11.pdf(&dataset);

    // Plot the observations and each of the PDFs.
    let trace1 = Scatter::new(x1, y1).mode(Mode::Markers).name("Data");
    let trace2 = Scatter::new(x2, y2).mode(Mode::Lines).name("Epanechnikov");
    let trace3 = Scatter::new(x3, y3).mode(Mode::Lines).name("Normal");
    let trace4 = Scatter::new(x4, y4).mode(Mode::Lines).name("Uniform");
    let trace5 = Scatter::new(x5, y5).mode(Mode::Lines).name("Triangular");
    let trace6 = Scatter::new(x6, y6).mode(Mode::Lines).name("Quartic");
    let trace7 = Scatter::new(x7, y7).mode(Mode::Lines).name("Triweight");
    let trace8 = Scatter::new(x8, y8).mode(Mode::Lines).name("Tricube");
    let trace9 = Scatter::new(x9, y9).mode(Mode::Lines).name("Cosine");
    let trace10 = Scatter::new(x10, y10).mode(Mode::Lines).name("Logistic");
    let trace11 = Scatter::new(x11, y11).mode(Mode::Lines).name("Sigmoid");
    let trace12 = Scatter::new(x12, y12).mode(Mode::Lines).name("Silverman");

    // Render the plot.
    let mut plot = Plot::new();
    plot.add_trace(trace1);
    plot.add_trace(trace2);
    plot.add_trace(trace3);
    plot.add_trace(trace4);
    plot.add_trace(trace5);
    plot.add_trace(trace6);
    plot.add_trace(trace7);
    plot.add_trace(trace8);
    plot.add_trace(trace9);
    plot.add_trace(trace10);
    plot.add_trace(trace11);
    plot.add_trace(trace12);
    plot.show();
}