## Payroll Management System

Internal payroll management app built with Rust (`axum`) + PostgreSQL.

## Local Run

1. Set `DATABASE_URL` in `.env`.
2. Start PostgreSQL.
3. Run:

```bash
cargo run
```

The server runs on `http://localhost:9000`.

## Docker (Self-Hosted)

Start app + database:

```bash
docker compose up --build
```

The app is exposed on `http://localhost:9000` and runs migrations on startup.
