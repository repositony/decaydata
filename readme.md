# IAEA decay data CLI (`ddata`)

Command line tool to retrieve decay data from the IAEA chart of nuclides

```text
Usage: decaydata <nuclides> [options]

Arguments:
  [nuclides]...          List of nuclide names

Options:
  -v, --verbose...       Verbose logging (-v, -vv)
  -q, --quiet            Supress all log output (overrules --verbose)
  -n, --no-colour        Turn off table colours
  -h, --help             Print help (see more with '--help')

Data options:
  -r, --rad <rad>        Type of decay radiation
  -s, --sort <property>  Sort records by property ['energy', 'intensity']
      --fetch            Query IAEA directly rather than pre-fetched data

Output files:
  -o, --output <name>    Prefix for output files
  -t, --text             Text based table
  -j, --json             JSON output format
  -m, --mcnp             MCNP distribution cards
  -i, --id <num>         Starting MCNP distribution number
      --csv              Fetch raw CSV directly (internet required)

Note: --help shows more information and examples
```

Help is printed with the `-h` flag, and `--help` will show default values,
examples, and any important behaviour.

## Install

Download and unpack the latest binary executable release [here](https://github.com/repositony/decaydata/releases/latest) for running in a terminal/powershell.

## Overview

This tool allows for rapid collection of [IAEA chart of nuclides](https://www-nds.iaea.org/relnsd/vcharthtml/VChartHTML.html)
from the command line.

Raw CSV data fetched from the IAEA API and parsed into something usable. This
may be done through direct calls to the IAEA API, or through the pre-fetched
data (default, recommended).

`ddata` supports all IAEA decay data types:

| Decay radiation type | IAEA symbol |
| -------------------- | ----------- |
| alpha                | a           |
| beta plus            | bp          |
| beta minus           | bm          |
| gamma                | g           |
| electron             | e           |
| x-ray                | x           |

Note that selecting 'gamma' will provide all photon data, including X-rays. This
is consistent with the data retrieved via the horrible IAEA API.

## Examples

### Basic use

A table of decay data is always printed for reference unless the `--quiet` flag
is used.

Nuclides may be given in the following formats:

| User nuclide           | Returns             | Note                           |
| ---------------------- | ------------------- | ------------------------------ |
| co60 Co-60 CO60 Co60m0 | Co60m0              | Decay from Ground state        |
| co60m co60m1 co60*     | Co60m1              | Decay from First excited state |
| be                     | Be7m0 Be11m0 Be14m0 | Elements expand to any ground state with data |

For example:

```bash
# Print IAEA decay data for ground state Cobalt-60 and Cesium-137 
ddata co60 cs137
```

To be explicit:

- Nuclides are `element` `number` `state`, where the `number` and `state` are optional
- Elements are expanded to find all nuclides with relevant decay data
- Nuclides are case-insensitive
- Dividers such as `-` in Co-60 are ignored
- Nuclides unknown or without relevant decay data are ignored
- FISPACT-II style metastable markers assumed to map m->m1, n->m2, etc..

### Choosing output formats

The following output formats are supported:

| Output format   | Flag                        |
| --------------- | --------------------------- |
| Utf-8 text file | `-t`/`--text`               |
| JSON            | `-j`/`--json`               |
| MCNP SDEF       | `-m`/`--mcnp`               |
| Raw CSV         | `--csv` (internet required) |

For example:

```bash
# Equivalent: creates 'decay_data.i', 'decay_data.json', 'decay_data.txt'
ddata co60 --mcnp --json --text
ddata co60 -m -j -t
ddata co60 -mjt
```

The `--text`, `--json`, and `--mcnp` files contain only nuclides with decay data
of energy-intensity parirs.

The `--mcnp` flag writes a source distribution of decay data for each nuclide.

Note that for MCNP SDEF, the distribution cards need an id. These are generated
sequentially from the value passed to `--id`. Defaults to `100`.

```bash
# Start from SI/SP 20 instead of 100
ddata co60 cs137 --mcnp --id 20
```

[!WARNING]
Note that the CSV data are the unparsed horror show direct from the IAEA API.

### Choosing output file prefix/name

Prefix for output files defaults to `decay_data`.

Files are automatically named `<name>.<ext>` where `<ext>` is the appropriate
extension.

For example:

```bash
# Change the prefix to myname
ddata co60 --mcnp --text --output myname
```

This generates `myname.i`/`mynmame.txt` instead of
`decay_data.i`/`decay_data.txt`.

### Choosing decay data type

`ddata` supports all IAEA decay data types.

The default is "gamma", however this may be changed using the `--rad`/`-r`
argument.

```bash
# Choose radiation type:
ddata co60  --rad gamma        [default]
ddata co60  --rad xray
ddata am241 --rad alpha
ddata na22  --rad beta-plus
ddata co60  --rad beta-minus
ddata na22  --rad electron
```

Note that the IAEA API returns any photon emission for `gamma`, including X-rays.

- For X-ray data only, use `--rad x-ray`
- For gamma-only, take it up with the IAEA

### Choosing decay data order

By default, all decay data are ordered by energy.

To order by relative intensity (descending), use the `--sort`/`-s` argument.

For example:

```bash
# Choose output data sorting:
ddata co60 cs137 --sort energy        [defualt]
ddata co60 cs137 --sort intensity
```

This applies to all output files.

### IAEA data options

It is **strongly recommended** to use the pre-compiled decay data (defualt).

If the absolute latest data are required and performance is not a concern, the
data may be fetched directly from the IAEA chart of nuclides API with the
`-f`/`--fetch` flag.

For example:

```bash
# Force decay data to be fetched direct from the IAEA chart of nuclides API
ddata co60 --fetch ...
```

This obviously requires an internet connection.
