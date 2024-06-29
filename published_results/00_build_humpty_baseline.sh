HUMPTY=../target/release/humpty
INPUT=../data/california-cases-per-day.csv
TOTALTIME=1142;
AHEAD=28
J2=182
J3=336
J4=581
J5=735
J6=875
J7=1050

# ORGANIZATION CHECKING 
if [ -z $HUMPTY ] ; then
	echo "Cannot find humpty binary.  you may need to build it using cargo." 
	exit 8
fi 
if [ -z $INPUT ] ; then
	echo "Cannot find the input file:$INPUT, please note the relative file path expected"
	exit 9
fi
if [ $(python3 -c "print(1)") -ne 1 ]; then 
       echo "python3 is needed"
       exit 11
fi        
# ARGUMENT CHECKING
if [ $# -ne 0 ] ; then
	echo "example:"
	echo ">$0 "
	echo " * create directory humpty_baseline"
	echo " * to build the humpty baseline model for COVID forecasting"
	echo " * will be built in: $(pwd)/humpty_baseline"
	exit 3
fi 

BASELINE="humpty_baseline"
mkdir -p $BASELINE 
if [ $? -ne 0 ] ; then 
	echo "cannot overwrite existing object $BASELINE, select another argument" 
	exit 14
fi


BUILD=1;
# FIRST we create the Humpty baseline model for COVID projections and forecasts here, 
# Note that we provide time points for the humpty model to make its jumps 
# from k to k+1 active growth modes.  These jumps were selected in retrospective analysis, 
# but are fixed to time points when a human in the loop (at the forecast date) would be 
# able to do the same, and are selected in conservative manner to ensure that evidence of 
# novel surge would be clear at the interceding time point.  In the ideal, an expert 
# who maintains the model on a daily basis would have already compared 
# hypothesis testing, a humpty-k vs humpty-(k+1) to determine if/when the addition of new 
# parameters is justifiable within the decision context of other information, such as  
# leading indicators that novel surge in other regions of the world anticipate the need 
# to increment the parameters of humpty.
#  
# In our paper we have defined primitive regularization terms which can potentially 
# help to fully automated a self aware model as our future work.  This future work will 
# focus on 1) persistence of historical parameters from the baseline, and recent divergence,
# 2) Variation in Residual fit and skew statistics using historical parameters vs current parameter estimates. 
#
# For this comparison, our jump points are labeled JK as the time point (days since first 
# collected) when the model introduces the Kth hump.  
#
if [ $BUILD -eq 1 ] ; then
        CH=1;
        for PREFIX in $(seq 1 1142); do
                if [ $PREFIX -eq $J2 ]; then
                        CH=2;
                fi
                if [ $PREFIX -eq $J3 ]; then
                        CH=3;
                fi
                if [ $PREFIX -eq $J4 ]; then
                        CH=4;
                fi
                if [ $PREFIX -eq $J5 ]; then
                        CH=5;
                fi
                if [ $PREFIX -eq $J6 ]; then
                        CH=6;
                fi
                if [ $PREFIX -eq $J7 ]; then
                        CH=7;
                fi
                HUMP=$CH;
                MODEL="${BASELINE}/compPRE${PREFIX}_HUMP${HUMP}.yml"
                FOREA="${BASELINE}/comp_fore_at_PRE${PREFIX}_HUMP${HUMP}.csv"
                VIEW="${BASELINE}/view_${PREFIX}_${HUMP}.png"
                $HUMPTY fit -n$HUMP --limit ${PREFIX} ${MODEL} ${INPUT}
                SKEWA="${BASELINE}/comp_stats_PRE${PREFIX}_HUMP${HUMP}.cvs"
                $HUMPTY csv skew -l${PREFIX} ${SKEWA} ${MODEL}
                $HUMPTY csv fore -l$((${PREFIX} + ${AHEAD} )) $FOREA $MODEL
                RESID1=$(./target/release/humpty exp intermediate $MODEL | grep "lsm" | tr ":" " " | awk -F"lsm." '{print $2}' )
                RESID=$(python3 -c "print( '{:.2f}'.format( $RESID1 )  )" )
                echo ">RESID: $RESID" 
                RESIDPP=$(python3 -c "print('{:.2f}'.format( $RESID / $PREFIX))")
                $HUMPTY viz basic -l$((${PREFIX} + ${AHEAD})) -T"model (prefix: $PREFIX, humps: $HUMP) LSM ${RESID} RPP:${RESIDPP}" -Y"cum case load" $VIEW $MODEL
                echo "BASELINE ($PREFIX, $CH)"
        done
fi

# Second, here we patch the data files (such as forecasts) that humpty generates, so 
# they can be smoothly compared with the other methods within a sharable python Jupiter 
# notebooks, so that other scientists can repeat such study as needed.
BUILD=2;
FIXIT=./fix.sed
echo "making fixit" 
if [ $BUILD -eq 2 ] ; then 
        # First create the mapping between line number and date.
        cat $INPUT | awk -F"," '(NF>1){print $1}'  | cat -n -  | awk '(NF>1){print int($1-1) " " $2 }' > time_index.txt
	cat time_index.txt | sed 's/[ \t]*/s\/^/' | sed 's/[ \t]\+/.0,\//' | sed 's/$/,\//' >  $FIXIT
	echo "FIXIT is calculated" 
	for f1 in $(ls ${BASELINE}/comp_fore_at_PRE* | grep -v "delta"); do
		PREI=$(echo $f1 | sed 's/.*PRE//' | sed 's/_.*//') 
		PRED=$(echo $f1 | sed 's/.*PRE//' | sed 's/_.*/.0,/' | sed -f ${FIXIT} | sed 's/,//' ) 
		f2="${f1%.csv}_delta.csv"
                echo "using $f to create $f2"
                cat $f1 | python3 humpty_baseline_delta.py -f1 > $f2

		F2=${BASELINE}/fore${PRED}.csv;
		F1=${BASELINE}/fore${PRED}CUM.csv;
		echo '"ahead","geo_value","quantile","value","forecaster","forecast_date","data_source","signal","target_end_date","incidence_period"' > $F1 
		echo '"ahead","geo_value","quantile","value","forecaster","forecast_date","data_source","signal","target_end_date","incidence_period"' > $F2
		PERIOD="day"	
		cat $f1 | tail -n ${AHEAD} | sed -f $FIXIT | awk -F"," '{print (NR -1 )",\"ca\",0.5," $2 ",\"HUMPTY-baseline\",XXXBASEDATEXXX, \"jhu-csse (cum)\", \"confirmed_incidence_num(cum)\","$1",\"YYYPERIODYYY\"" }' | sed "s/XXXBASEDATEXXX/${PRED}/" | sed "s/YYYPERIODYYY/$PERIOD/"  >> $F1
		cat $f2 | tail -n ${AHEAD} | sed -f $FIXIT | awk -F"," '{print (NR-1)",\"ca\",0.5," $2 ".0 ,\"HUMPTY-baseline\",XXXBASEDATEXXX, \"jhu-csse\",\"confirmed_incidence_num\","$1",\"YYYPERIODYYY\"" }' | sed "s/XXXBASEDATEXXX/${PRED}/" | sed "s/YYYPERIODYYY/$PERIOD/" >> $F2
		echo "will patch fie ${f1} : PRE:i[${PREI}] : PRED=[${PRED}]" 
		

		
	done
	rm $FIXIT
	
fi

