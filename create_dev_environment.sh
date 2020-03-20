#!/bin/bash
docker-compose down
sudo rm -rf ./devenv/influxdb
docker-compose up -d
docker exec influxdb influx -execute 'create database environment;'
