CREATE TABLE users (
    uid serial PRIMARY KEY NOT NULL,
    fname VARCHAR(25) NOT NULL,
    lname TEXT,
    images TEXT,
    email VARCHAR(50) UNIQUE NOT NULL,
    phone VARCHAR(15),
    password VARCHAR(260),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP WITH TIME ZONE
);

CREATE TABLE subscription(
    uid serial PRIMARY KEY NOT NULL,
    user_id INT NOT NULL,
    subscribed_id INT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE users_cache(
    uid serial PRIMARY KEY NOT NULL,
    user_id INT NOT NULL,
    users JSON,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);