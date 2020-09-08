#!/bin/bash

set -x

SCRIPT_DIR=$(
    cd $(dirname $0)
    pwd
)
DIR=$SCRIPT_DIR/../admin/
docker run --rm mysql:5.7 mysql -uroot -pishocon -hhost.docker.internal -P13306 -e 'CREATE DATABASE ishocon2;'
cd $DIR/
tar -jxvf ishocon2.dump.tar.bz2
docker run --rm -i mysql:5.7 mysql -uroot -pishocon -hhost.docker.internal -P13306 ishocon2 <$DIR/ishocon2.dump
