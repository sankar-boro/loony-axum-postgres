#!/bin/bash

rm $HOME/.bin/v1_api
cp ./target/release/loony_axum_postgres $HOME/.bin/v1_api
sudo chmod +x $HOME/.bin/v1_api