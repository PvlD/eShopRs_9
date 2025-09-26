-- Create users table.
create table if not exists users
(
    id           integer primary key autoincrement,
    sub          text not null unique,
    username     text not null unique,
    email        text not null unique,
    access_token text not null,
    refresh_token text,
    street text,
    city text,
    state text,
    country text,
    zip text 
);
