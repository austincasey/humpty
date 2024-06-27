
for k in $(seq 1 163); do
    MODELSTUB=tmp/train_PRE${k}_
    c=1; 
    for f in $(ls ${MODELSTUB}*.yml ); do
	if [ $c -eq 1 ]; then 
	   echo ";;  ${f}"
           ./target/release/humpty exp intermediate $f | grep 't_0' | awk '{print $3}' | sed 's/,//' | sort -n > /tmp/tmptmp.txt
	   TPS=$(cat /tmp/tmptmp.txt | wc | awk '{print $1}')
	   cat /tmp/tmptmp.txt
	   for j in $(seq $TPS 8); do
	       echo "0.0"
	   done
	   echo ";; done ${TPS}"
	fi
	c=$((c+1)); 
    done
done
	     

# | grep -v ";;  tmp" | tr "\n" "," | sed 's/;; done[^,]*,/\n/g' > P.m
