#!/bin/bash
sudo service nginx start
sudo mysql -h db -u root -pishocon -e 'CREATE DATABASE IF NOT EXISTS ishocon2;' &&
    sudo mysql -h db -u root -pishocon -e "CREATE USER IF NOT EXISTS ishocon IDENTIFIED BY 'ishocon';" &&
    sudo mysql -h db -u root -pishocon -e 'GRANT ALL ON *.* TO ishocon;' &&
    cd ~/data && tar -jxvf ishocon2.dump.tar.bz2 && sudo mysql -h db -u root -pishocon ishocon2 <~/data/ishocon2.dump

echo 'setup completed.'
cd /home/ishocon/webapp/rust/
export PATH=$PATH:/home/ishocon/.cargo/bin
# flamegraphを出すにはここをコメントから戻してcargo watchを外す
# cargo build
# cargo flamegraph --dev -- 8080
cargo watch -x 'run -- 8080'
