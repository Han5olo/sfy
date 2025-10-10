# Processing scripts for SFY data

These scripts and packages are used to fetch wave and track data. They contain
signal processing routines for integrating to displacement and calculating
spectral statistics like significant wave height. You need to set up a
environment with the dependencies and then install this package. Below is the
recommended way to do this.

Set up and install the environment using e.g. conda (conda-forge) or [`mamba`](https://github.com/conda-forge/miniforge#mambaforge) (`conda` replacement).

Install the processing scripts using e.g.:

```
$ cd sfy-processing/
$ mamba env create -f ../environment.yml  # or use `conda`.
$ conda activate sfy
$ pip install -e .
```

## Usage

Specify the server and read-token in environment variables, e.g. in `.bashrc`, for Linux∕MocOS (1), Windows PowherShell (2) or CMD (3).


1. For Linux and MacOS (MacOS not tested)
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


```
sfydata track csv bug52
```

## Getting the wave data

```
sfydata axl ts bug52 --file bug52.nc
```

For more options see:
```
sfydata axl ts bug52 --help
```

