## Payroll Management System

Internal payroll management app built with Rust (`axum`) + PostgreSQL.

## Installation and Run Guide

### 1) Clone the repository

```bash
git clone https://github.com/AsherDMckoy/Payroll-System-Belize.git
cd Payroll-System-Belize
```

If you are running from a local copy with a different folder name, just `cd` into that folder.

### 2) Prerequisites

- `git`
- Docker Engine
- Docker Compose v2 (`docker compose`)
- Optional for local non-Docker run: Rust toolchain (`rustup`, `cargo`) + PostgreSQL

Check:

```bash
docker --version
docker compose version
```

### 3) Run with Docker (HTTP)

Build and start app + database:

```bash
docker compose up --build
```

If your Docker setup requires elevated privileges, use:

```bash
sudo docker compose up --build
```

Application URL:

- `http://localhost:9000`

### 4) Run with Docker (HTTPS / TLS)

This project includes HTTPS support through a Docker TLS reverse proxy (`Caddy`) with local/internal certificates.

Build and start app + database + TLS proxy:

```bash
docker compose -f docker-compose.yml -f docker-compose.https.yml up --build
```

Application URL:

- `https://localhost:9443`

Notes:

- In HTTPS mode, cookies are set with `Secure` (`COOKIE_SECURE=true` is injected through `docker-compose.https.yml`).
- Browsers may show a certificate warning for local/internal certificates; this is expected in local demo mode.

### 5) Verify the application

Health endpoints:

- `GET /health` (liveness)
- `GET /ready` (readiness + DB check)

Example checks:

```bash
curl http://localhost:9000/health
curl http://localhost:9000/ready
```

For HTTPS mode:

```bash
curl -k https://localhost:9443/health
curl -k https://localhost:9443/ready
```

### 6) Stop containers

HTTP stack:

```bash
docker compose down
```

HTTPS stack:

```bash
docker compose -f docker-compose.yml -f docker-compose.https.yml down
```

### 7) Optional: local run without Docker

1. Create a `.env` file:

```env
DATABASE_URL=postgres://postgres@localhost/payroll_db
RUN_MIGRATIONS_ON_STARTUP=true
APP_HOST=0.0.0.0
PORT=9000
COOKIE_SECURE=false
```

2. Start PostgreSQL locally.
3. Run:

```bash
cargo run
```

The server runs on `http://localhost:9000`.

### Docker service summary

- `db`: PostgreSQL 18
- `app`: Rust Axum application
- `https-proxy` (optional): Caddy TLS terminator (`docker-compose.https.yml`)

## Lecturer Admin Login

Use this account to access the system for grading/demo:

- Full Name: `Tobi McGuire`
- Username: `Bullly2003`
- Password: `SpideySenes5!`

## Generated Reports Directory

The `reports/` directory contains sample generated exports:

- `reports/sample-employee-profile-report.pdf` (employee profile sample PDF)
- `reports/sample-payroll-summary-belizebankfriendlyformat.csv` (payroll summary sample CSV)

The payroll summary CSV export is generated in a Belize Bank-friendly layout intended for payroll submission from a business account via the Belize Bank web application workflow.

## Database Schema (ERD)

```mermaid
erDiagram
    DEPARTMENTS {
        int id PK
        string name
        string code
        timestamp created_at
    }

    EMPLOYEES {
        int id PK
        string employee_number UK
        string first_name
        string last_name
        string email UK
        int department_id FK
        string employment_type
        string status
        date hire_date
        date termination_date
        string bank_account_number
        string bank_routing_number
        timestamp created_at
        timestamp updated_at
    }

    COMPENSATION {
        int id PK
        int employee_id FK
        decimal base_salary
        decimal hourly_rate
        date effective_date
        bool is_current
        timestamp created_at
    }

    EMPLOYEE_SAVINGS {
        int id PK
        int employee_id FK
        decimal contribution_amount
        date contribution_date
        timestamp created_at
    }

    PAYROLL_PERIODS {
        int id PK
        date start_date
        date end_date
        date pay_date
        string status
        bool is_locked
        timestamp locked_at
        string locked_by
        timestamp created_at
        timestamp updated_at
    }

    TIMESHEETS {
        int id PK
        int employee_id FK
        int payroll_period_id FK
        decimal regular_hours
        decimal overtime_hours
        timestamp created_at
        timestamp updated_at
    }

    PAYROLL_ADJUSTMENTS {
        int id PK
        int employee_id FK
        int payroll_period_id FK
        string adjustment_type
        decimal amount
        string description
        string created_by
        timestamp created_at
    }

    PAYROLL_RUNS {
        int id PK
        int employee_id FK
        int payroll_period_id FK
        decimal gross_pay
        decimal regular_pay
        decimal overtime_pay
        decimal bonuses
        decimal tax_deduction
        decimal employee_savings
        decimal company_match
        decimal total_deductions
        decimal net_pay
        string payment_status
        date payment_date
        timestamp created_at
        timestamp updated_at
    }

    AUDIT_LOGS {
        int id PK
        string table_name
        int record_id
        string action
        string user_name
        jsonb changed_data
        timestamp created_at
    }

    APP_USERS {
        int id PK
        string username UK
        string full_name
        string role
        string password_hash
        bool is_active
        timestamp created_at
        timestamp updated_at
    }

    USER_SESSIONS {
        int id PK
        int user_id FK
        string session_token_hash UK
        timestamp expires_at
        timestamp revoked_at
        string ip_address
        string user_agent
        timestamp created_at
    }

    DEPARTMENTS ||--o{ EMPLOYEES : has
    EMPLOYEES ||--o{ COMPENSATION : has_history
    EMPLOYEES ||--o{ EMPLOYEE_SAVINGS : contributes
    EMPLOYEES ||--o{ TIMESHEETS : logs
    PAYROLL_PERIODS ||--o{ TIMESHEETS : contains
    EMPLOYEES ||--o{ PAYROLL_ADJUSTMENTS : receives
    PAYROLL_PERIODS ||--o{ PAYROLL_ADJUSTMENTS : contains
    EMPLOYEES ||--o{ PAYROLL_RUNS : paid_in
    PAYROLL_PERIODS ||--o{ PAYROLL_RUNS : includes
    APP_USERS ||--o{ USER_SESSIONS : owns
```
