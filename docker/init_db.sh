#!/bin/bash
set -x
/usr/bin/mysqld_safe --skip-grant-tables &
sleep 5
mysql -u root -pishocon -e 'CREATE DATABASE ishocon2;'
mysql -u root -pishocon ishocon2 </tmp/ishocon2.dump/ishocon2.dump
