# Scylla

#### Start Scylla Server

~ sudo systemctl start scylla-server

#### Create table users

CREATE KEYSPACE sankar WITH replication = {
    'class': 'SimpleStrategy',
    'replication_factor': 1
};

describe tables;

DROP KEYSPACE sankar;