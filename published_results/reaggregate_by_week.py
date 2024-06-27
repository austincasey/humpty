import datetime, sys

INPUT="./data/california-cases-per-day.csv"
VALIDATE="./data/california-cases-per-week.csv"

STARTDAY="2020-01-22";
STARTDAY2="2020-01-27";

def read_data_render_week_data( T, STARTDAY, fun=lambda x : int(x) ):
    curd = datetime.datetime.fromisoformat(STARTDAY)
    curds = STARTDAY;
    weekly = 0.0;
    lastweek = None;
    lastb = 0.0;
    first = True
    rv = "";
    rv2 = "";
    for l in T:
        if ( len( l.strip() )):
            f = l.split(",");
            if ( len( f ) == 2 ):
                a, b = f;
                a = a.strip();
                b = float(b.strip());            
                if ( datetime.datetime.fromisoformat( a ) == curd ):
                    lastweek = weekly
                    weekly = lastb ;
                    delt = fun(weekly - lastweek )
                    if not first:
                        rv += f"{curds},{delt}" + "\n"
                        rv2 += f"{curds},{weekly}" + "\n"                
                    else:
                        first = False;
                    curd = curd + datetime.timedelta(days=7)
                    curds = a;
                lastb = b;
            else:
                print( f"error -- odd line: [{f}]" );
    return (rv,rv2);

def validate( ):
    T = open( INPUT ).read().split("\n" );    
    t1,_= read_data_render_week_data( T, STARTDAY, fun=lambda x : int(x) );
    t2= open( VALIDATE, "r" ).read();
    #print( (len( t1 ), len(t2 ) );
    print( (len(t1), len(t2)))
    for k in range( max( len(t1), len(t2))):
        l1 = t1[k];
        l2 = t2[k];
        if ( l1 != l2 ):
            print( f"producing non-matching output {k}" );
            print( f"<{l1}" )
            print( f">{l2}" );
            print( "" );
            return False;
    return True;




if (len( sys.argv ) == 1 ):
    validate()
elif (len( sys.argv ) == 2 ):
    STARTDAY2=sys.argv[1];
    T = open( INPUT ).read().split("\n" );    
    t1,tcum= read_data_render_week_data( T, STARTDAY2, fun=lambda x : x );
    print( tcum );
else:
    print( f"{sys.argv[0]} 2020-01-27" )

 
