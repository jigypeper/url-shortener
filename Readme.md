# URL Shortener

## Development

Run with `cargo run --features dotenv`.

Start database with:

```shell
docker build -t url-shortener-db:latest database
docker run --rm --env POSTGRES_PASSWORD=password url-shortener-db:latest
```  
Note:
on MacOS it may be necessary to re-create the database image and add the role postgres.
To run the tests, create the test_url_shortener database:  

```shell
psql "host=127.0.0.1 port=5432 user=postgres password=password dbname=postgres"
create database test_url_shortener;
# change to the db
\c test_url_shortener
# create the schema:
CREATE TABLE link
(
    id                 text PRIMARY KEY,
    url                text  NOT NULL,
    development_fields jsonb NOT NULL DEFAULT '{}'
);

# start the database and run cargo test
cargo test
```
