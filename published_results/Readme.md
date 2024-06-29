# Comparing Humpty to other forecasting methods.

This README describes the steps to set up and then compare Humpty to other methods as we demonstrate in our paper.
Author: Will Casey

(c) W. Casey 2024

# step one generate the Humpty COVID-19 baseline model

You will need to use a Linux system with bash, sed, awk, and other standard Linux tools, further python3 will also be needed.

run:

> ./00_build_humpty_baseline.sh 

This will take about 90 mins or so to process on a laptop.


# Step two you will need to build a python environment enabling the Jupiter notebook script.
Note we provide instructions for python3 (ref: https://docs.python.org/3/library/venv.html ) 
If these do not work verify your python distribution.

> python3 -m venv . 
> source ./bin/activate 
> pip3 install -r requirements.txt

## start the notebook system

> ./bin/jupyter-notebook

This step is validated by seeing a new tab within your browser.  Further the notebook server should be using the python environment previously set up, this can be validated by importing some of the libraries within the requirements.txt file (for example create a notebook, and evaluate a cell with code:
>import polars

If importing items from the requirements fails, it maybe due to the Jupyter-notebook's python kernel setting.   If you struggle with this see:
https://medium.com/@kishanck/virtual-environments-for-jupyter-notebooks-847b7a3b4da0


## Within the browser, an instance of jupyter notebooks should be running,
Now Open and run the Jupyter notebook 
load the following file:

compare_humpty_others.ipynb

Running the cells will generate each method's MSE sequences for each forecast date.  Comparisons are done as humpty vs method-X for each method, the result being pairwise MSE sequences, and total MSE plotted into a point on a log-log graph.

