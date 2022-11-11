#!/bin/bash

set -ex

# Set up rust environment
export CARGO_HOME=${CONDA_PREFIX}/.cargo.$(uname)
export CARGO_CONFIG=${CARGO_HOME}/config
export RUSTUP_HOME=${CARGO_HOME}/rustup

maturin build --release --strip --manylinux off --interpreter="${PYTHON}"

"${PYTHON}" -m pip install $SRC_DIR/target/wheels/${PKG_NAME}*-cp${PY_VER/\./}-cp${PY_VER/\./}*.whl --no-deps -vv
