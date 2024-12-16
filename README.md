# Efficient Task Dispatching Framework for Serverless Edge Computing

HLiu

[https://www.overleaf.com/project/6702934a2c3c44e6d73c4000](https://www.overleaf.com/project/6702934a2c3c44e6d73c4000)

## Experiments

### Preparation

1. Make sure the rust environment work. install rust by `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Make sure the python 3 environment work.

### Experiment 1 initial test on Raspberry 3

set to 1024 MB memory, 1 core, and 3% execution cap

`cargo run --bin rpi.rs`

The data are in `/docs/Experiment-1-rpi.xlsx`.

The plot script is `/scripts/exp-1-diagram.py`.

### Experiment 2

Start an e-computer: `cargo run --bin e_computer 100000`

Start an e-router: `cargo run --bin e_router RP 63789:1 63792:2 63795:2 63810:3`

or `cargo run --bin e_router_v3 63789:1 63792:2 63795:2 63810:3`

the number before the colon is the port number of an e-computer. the number after the colon is the hops between the e-router and the e-computer, which is not implemented yet.

The data are in `/scripts/exp-2-per-95` and `/docs/Experiment-2.xlsx`.

The LI and RP are in e_router.rs. The RR is in e_router_v3.rs.

`/scripts/manager-95.py` is the client manager for this experiment.

`/scripts/per95.py` is for calculation 95th percentile delay.

`/scripts/per95-diagram.py` is for plot.
