#!/bin/bash
curl -v -d '{"temperature":27.05,"humidity":37.95,"pressure":1011.72,"raw_voltage":713,"charge":51.13,"sensor":"DEADBEEF","firmware_version":"0.0.1-dev"}' -H "Content-Type: application/json" -X POST http://127.0.0.1:5471/v1/sensor/measurement
