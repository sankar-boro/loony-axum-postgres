#!/usr/bin/env bash

curl 'http://localhost:8000/api/auth/signup' \
  -H 'Content-Type: application/json' \
  --data-raw '{"fname":"Arun","lname":"","email":"sankar.boro@yahoo.com","password":"sankar"}' | jq

# sec-ch-ua: Stands for "Secure Client Hints - User Agent."