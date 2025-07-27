# AcderatorServer

Backend server for [Acderator](https://github.com/Ierllow/Acderator).

This project is a Rust based server for Acderator.  
It provides APIs for authentication, score handling, and user data management, and uses a MySQL compatible database for storage.

## Setup

You can use `setup.sh` to prepare the environment and start the server.

```bash
bash scripts/setup.sh
```

You can customize the database name, user name, password, and master data path by editing the variables at the top of `setup.sh` .

## setup.sh

The `setup.sh` script performs the following steps:

- installs required system packages
- installs Rust
- starts MariaDB
- creates the database and user
- creates the `.env` file
- runs database migrations
- starts the server with `cargo run`

## Maintenance Mode

Maintenance mode is controlled by the `MAINTENANCE_MODE` constant in `src/common/config.rs`.
When enabled, the server returns a MessagePack `503 Service Unavailable` response before request handlers run.

## Request Protection

Request body size is capped by the `REQUEST_BODY_LIMIT_BYTES` constant in `src/common/config.rs`.
The default is `65536`.

Requests per client IP are limited by the `RATE_LIMIT_MAX_REQUESTS` and `RATE_LIMIT_WINDOW_SECONDS` constants in `src/common/config.rs`.
The default is `120` requests per `60` seconds. Set `RATE_LIMIT_MAX_REQUESTS=0` in code to disable rate limiting.

## Debug UI

<img width="1175" height="668" alt="image" src="https://github.com/user-attachments/assets/2b936b10-5b96-4d8e-aaca-0fd1dd1f4b43" />
   
Start the server with `cargo run --features debug-ui` and open `/debug` in a browser to inspect API request and response data.  
The page sends MessagePack requests and decodes MessagePack responses for development use.  
Open `/debug/master` to inspect local master data version, table counts, song rows, and raw JSON.

## Database Structure
  
The database is separated into two types of tables:

- **Master Tables**  
  These tables store predefined application data such as songs, scoring rules, judge settings, and other fixed configuration values.

- **Transaction Tables**  
  These tables store runtime data such as user accounts, login sessions, gameplay sessions, and score records.

## License  
AcderatorServer is under MIT [LICENSE](LICENSE).  
