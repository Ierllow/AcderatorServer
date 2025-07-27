#!/usr/bin/env bash
set -eux

export DEBIAN_FRONTEND=noninteractive

# ===== default values =====
DB_NAME="acderator"
DB_USER="acderator"
DB_PASS="acderatorpass"
MASTER_DATA_PATH="static/master/data.json"
# ==========================

while [[ $# -gt 0 ]]; do
  case "$1" in
    --db-name)
      DB_NAME="$2"
      shift 2
      ;;
    --db-user)
      DB_USER="$2"
      shift 2
      ;;
    --db-pass)
      DB_PASS="$2"
      shift 2
      ;;
    --master-data-path)
      MASTER_DATA_PATH="$2"
      shift 2
      ;;
    -h|--help)
      cat <<EOF
Usage: $0 [options]

Options:
  --db-name NAME             Database name
  --db-user USER             Database user
  --db-pass PASS             Database password
  --master-data-path PATH    Master data json path
  -h, --help                 Show this help
EOF
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

apt-get update
apt-get install -y curl build-essential pkg-config libssl-dev mariadb-server

if ! command -v cargo >/dev/null 2>&1; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y
fi

source "$HOME/.cargo/env"

service mysql start || service mariadb start

mysql -u root -e "CREATE DATABASE IF NOT EXISTS \`${DB_NAME}\`;"
mysql -u root -e "CREATE USER IF NOT EXISTS '${DB_USER}'@'localhost' IDENTIFIED BY '${DB_PASS}';"
mysql -u root -e "GRANT ALL PRIVILEGES ON \`${DB_NAME}\`.* TO '${DB_USER}'@'localhost'; FLUSH PRIVILEGES;"

cat > .env <<EOF
DATABASE_URL=mysql://${DB_USER}:${DB_PASS}@127.0.0.1:3306/${DB_NAME}
MASTER_DATA_PATH=${MASTER_DATA_PATH}
EOF

mysql -u "$DB_USER" -p"$DB_PASS" "$DB_NAME" < migrations/20250727114537_acderator_master_table.sql
mysql -u "$DB_USER" -p"$DB_PASS" "$DB_NAME" < migrations/20250727114537_acderator_transaction_table.sql

source "$HOME/.cargo/env"
cargo run