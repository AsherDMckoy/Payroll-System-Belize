## Payroll Management System

Internal payroll management app built with Rust (`axum`) + PostgreSQL.

## Local Run

1. Set `DATABASE_URL` in `.env`.
2. Optional: set `RUN_MIGRATIONS_ON_STARTUP=false` if you want to skip startup migrations.
3. Start PostgreSQL.
4. Run:

```bash
cargo run
```

The server runs on `http://localhost:9000`.

Health endpoints:
- `GET /health` (liveness)
- `GET /ready` (readiness + DB check)

## Docker (Self-Hosted)

Start app + database:

```bash
docker compose up --build
```

The app is exposed on `http://localhost:9000` and runs migrations on startup.

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
