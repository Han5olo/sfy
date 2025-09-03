SFY in Simple
========================

## Miniconda


### What is Conda and miniconda:  
1. Conda is a package manager and environment manager.
2. Miniconda is a lightweight installer for Conda. It gives you just the basic Conda package manager and Python, without all the extra packages that come with Anaconda.


### What is Mamba

Mamba is a fast alternative to Conda.

### Install Miniconda

Follow these instructions dor Windows, Linux, macOS :  
https://www.anaconda.com/docs/getting-started/miniconda/install#windows-powershell

### Install Mamba (not needed)

conda install mamba -n base -c conda-forge


### Install sfy


#### Install git

Follow these:
https://git-scm.com/downloads


First clone the repo from github:

git clone https://github.com/gauteh/sfy.git

now navigate in the directory sfy/sfy-processing/

Install the processing scripts using e.g.:

```
$ cd sfy-processing/
$ mamba env create -f ../environment.yml  # or use `conda`.
$ conda activate sfy
$ pip install -e .
```

## Usage

Specify the server and read-token in environment variables, e.g. in `.bashrc`, for Linux∕MocOS (1), Windows PowherShell (2) or CMD (3).


1. For Linux and MacOS
```
export SFY_SERVER='https://wavebug.met.no'
export SFY_READ_TOKEN='secret' # replace with the actual token
export SFY_DATA_CACHE='/tmp/sfy-cache'
```

2. For Windows Powershell:
```
$env:SFY_SERVER = "https://wavebug.met.no"
$env:SFY_READ_TOKEN = "secret"    # or your real token
$env:SFY_DATA_CACHE = "C:\Temp\sfy-cache"
```

3. For Windows CMD: (not tested)
```
set SFY_SERVER=https://wavebug.met.no
set SFY_READ_TOKEN=secret
set SFY_DATA_CACHE=C:\Temp\sfy-cache
```

with the conda environment activate try it out with:

```
sfydata list
```

Use `sfydata --help` or `--help` on subcommands to discover which arguments and
options exists.

## Getting the CSV tracks for a buoy


bug52 is the name of the buoy and must be replaced.

```
sfydata track csv bug52
```

save the  csv ouput as csv file:  
  
```
sfydata track csv STAR01 > output.csv
```

## Plot Significant Wave height:

```
sfydata plot STAR01 hm0
```

## Getting the wave data

```
sfydata axl ts bug52 --file bug52.nc
```

For more options see:
```
sfydata axl ts bug52 --help
```









