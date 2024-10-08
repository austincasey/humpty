#!/bin/python



#L = sys.stdin.read().split('\n');
#D=[l.split(',') for l in L if len(l.strip()) > 0 ];
#E = [ (D[k][0], float( D[k][1] ) - float( "0" if (k ==1) else D[k-1][1]) , D[k][2]) for k in range(1, len(D)) ] ; D2 = D[0] + [ e[0] + str(int( e[1] )) + e[2] for e in E ]; p^Cnt( '\n'.join( D2 )) 


import getopt, sys

def usage(selfname ):
    print( f"""
    USAGE:
    {selfname} -f"1,2" will calculate detls for column 1,2 (zero based index)
    """);

def process( finput, foutput, fields ):

    L =[]
    if ( finput == None ):
        L = sys.stdin.read().split("\n" );
    else:
        L = open( finput, "r" ).read().split( "\n" );
    header= False;
    try :
        float( L[0][fields[0]] );
    except:
        header = True ;
    D = L[(1 if header else 0):];
    D1 = [ l.split(",") for l in D if (len(l.strip()) > 0 ) ] 
    D2 = [ [float( l[k] ) if (k in fields) else l[k] for k in range(len(l))]  for l in D1 ]

    # Calculate deltas
    DD = [ [int(float( D2[k][j] )) if (j in fields)
                   else
                   D2[k][j]
                   for j in range(len(D2[k])) ]
           if (k==0) else
           [int(float( D2[k][j] ) - float( D2[k-1][j])) if (j in fields)
                   else
                   D2[k][j]
                   for j in range(len(D2[k])) ]
           for k in range(len(D2)) ] 
    
    # rebuild
    if header :
        print( L[0] );
        # TODO could add info in header that delta is calculated
    else:
        ...
    DN = [ [str(f) for f in l] for l in DD ] 
    print( "\n".join([ ",".join( l ) for l in DN ] ))
    
def main():
    try:
        opts, args = getopt.getopt(sys.argv[1:], "hi:o:f:v", ["help", "input=", "output=", "fields="])
    except getopt.GetoptError as err:
        print(err) 
        usage()
        sys.exit(2)
    finput = None
    foutput = None
    fields = [] 
    verbose = False
    for o, a in opts:
        if o == "-v":
            verbose = True
        elif o in ("-h", "--help"):
            usage(sys.argv[0] )
            sys.exit()
        elif o in ("-o", "--output"):
            foutput = a
        elif o in ("-i", "--input"):
            finput = a
        elif o in ("-f", "--fields"):
            fields = [int(v) for v in a.split("," )]
        else:
            assert False, "unhanded option"
    if len( fields ):
        process( finput, foutput, fields );
    else:
        usage(sys.argv[0])
if __name__ == "__main__":
    main()
