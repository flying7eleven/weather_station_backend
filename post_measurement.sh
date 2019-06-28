#!/bin/bash
curl -v -d '{"sensor":"DEADBEEF", "temperature":32.1, "humidity": 56.1, "pressure": 1003.5}' -H "Content-Type: application/json" -X POST http://localhost:8000/v1/sensor/measurement
