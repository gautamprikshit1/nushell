#!/usr/bin/env bash
# Usage: Just run `./install-all.sh` in nushell root directory
set -euo pipefail

echo "-----------------------------------------------------------------"
echo "Installing nushell (nu) with dataframes and all the plugins"
echo "-----------------------------------------------------------------"
echo ""

echo "Install nushell from local..."
echo "----------------------------------------------"
cargo install --path . --features=dataframe

NU_PLUGINS=(
    'nu_plugin_inc'
    'nu_plugin_gstat'
    'nu_plugin_query'
    'nu_plugin_example'
    'nu_plugin_custom_values'
    'nu_plugin_formats'
)

for plugin in "${NU_PLUGINS[@]}"
do
    echo ''
    echo "----------------------------------------------"
    echo "Install plugin $plugin from local..."
    echo "----------------------------------------------"
    cd crates/"$plugin" && cargo install --path . && cd ../../
done
