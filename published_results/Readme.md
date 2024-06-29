# comparing Humpty to other forecasting methods.

This readme describes the steps to set up and then compare Humpty to other methods as we demonstrait in our paper.

# step one generate the Humpty COVID-19 baseline model

You will need to use a Linux system with bash, sed, awk, and other standard Linux tools, further python3 will also be needed.

run:

> ./00_build_humpty_baseline.sh 

This will take about 30 mins or so on a laptop.


# Step two you will need to build a python environment enabling the Jupiter notebook script.

> pip3 install virtualenv 
> virtualenv . 
> source ./bin/activate 
> pip3 install -r requirements.txt

## start the notebook system

> ./bin/jupyter-notebook

This step is validated by seeing a new tab within your browser. 
If you struggle with this see:
https://medium.com/@kishanck/virtual-environments-for-jupyter-notebooks-847b7a3b4da0


## Within the browser, an instance of jupyter notebooks should be running,
Now Open and run the Jupyter notebook 
compare_humpty_delphi_others.ipynb

