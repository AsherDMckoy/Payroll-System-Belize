-- Payroll System Database Schema
-- PostgreSQL Migration Script

-- Enable UUID extension (optional, but useful)
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ======================
-- 1. DEPARTMENTS
-- ======================
CREATE TABLE departments (
    id                  SERIAL PRIMARY KEY,
    name                VARCHAR(100) NOT NULL,
    code                VARCHAR(20) UNIQUE,
    created_at          TIMESTAMP DEFAULT NOW()
);

-- ======================
-- 2. EMPLOYEES
-- ======================
CREATE TABLE employees (
    id                  SERIAL PRIMARY KEY,
    employee_number     VARCHAR(50) UNIQUE NOT NULL,
    first_name          VARCHAR(100) NOT NULL,
    last_name           VARCHAR(100) NOT NULL,
    email               VARCHAR(255) UNIQUE NOT NULL,
    phone               VARCHAR(20),
    department_id       INTEGER REFERENCES departments(id) ON DELETE SET NULL,
    position            VARCHAR(100),
    employment_type     VARCHAR(20) NOT NULL CHECK (employment_type IN ('salaried', 'hourly', 'contractor')),
    status              VARCHAR(20) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'terminated')),
    hire_date           DATE NOT NULL,
    termination_date    DATE,
    bank_account_number VARCHAR(50),
    bank_routing_number VARCHAR(50),
    created_at          TIMESTAMP DEFAULT NOW(),
    updated_at          TIMESTAMP DEFAULT NOW(),
    
    CONSTRAINT valid_employment_dates CHECK (termination_date IS NULL OR termination_date >= hire_date)
);

-- Index for common queries
CREATE INDEX idx_employees_status ON employees(status);
CREATE INDEX idx_employees_department ON employees(department_id);
CREATE INDEX idx_employees_type ON employees(employment_type);

-- ======================
-- 3. COMPENSATION
-- ======================
CREATE TABLE compensation (
    id                  SERIAL PRIMARY KEY,
    employee_id         INTEGER NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    base_salary         DECIMAL(12,2),
    hourly_rate         DECIMAL(8,2),
    effective_date      DATE NOT NULL DEFAULT CURRENT_DATE,
    is_current          BOOLEAN DEFAULT true,
    created_at          TIMESTAMP DEFAULT NOW(),
    
    CONSTRAINT salary_or_hourly CHECK (
        (base_salary IS NOT NULL AND hourly_rate IS NULL) OR
        (base_salary IS NULL AND hourly_rate IS NOT NULL)
    ),
    CONSTRAINT positive_compensation CHECK (
        (base_salary IS NULL OR base_salary > 0) AND
        (hourly_rate IS NULL OR hourly_rate > 0)
    )
);

-- Only one current compensation record per employee
CREATE UNIQUE INDEX idx_compensation_current ON compensation(employee_id) WHERE is_current = true;
CREATE INDEX idx_compensation_employee ON compensation(employee_id);

-- ======================
-- 4. EMPLOYEE SAVINGS
-- ======================
CREATE TABLE employee_savings (
    id                  SERIAL PRIMARY KEY,
    employee_id         INTEGER NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    contribution_amount DECIMAL(10,2) NOT NULL CHECK (contribution_amount >= 0),
    contribution_date   DATE NOT NULL,
    created_at          TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_savings_employee ON employee_savings(employee_id);
CREATE INDEX idx_savings_date ON employee_savings(contribution_date);

-- ======================
-- 5. PAYROLL PERIODS
-- ======================
CREATE TABLE payroll_periods (
    id                  SERIAL PRIMARY KEY,
    start_date          DATE NOT NULL,
    end_date            DATE NOT NULL,
    pay_date            DATE NOT NULL,
    status              VARCHAR(20) DEFAULT 'draft' CHECK (status IN ('draft', 'processing', 'approved', 'paid')),
    created_at          TIMESTAMP DEFAULT NOW(),
    updated_at          TIMESTAMP DEFAULT NOW(),
    
    CONSTRAINT valid_period_dates CHECK (end_date >= start_date AND pay_date >= end_date),
    CONSTRAINT unique_period UNIQUE(start_date, end_date)
);

CREATE INDEX idx_payroll_periods_status ON payroll_periods(status);
CREATE INDEX idx_payroll_periods_dates ON payroll_periods(start_date, end_date);

-- ======================
-- 6. TIMESHEETS
-- ======================
CREATE TABLE timesheets (
    id                  SERIAL PRIMARY KEY,
    employee_id         INTEGER NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    payroll_period_id   INTEGER NOT NULL REFERENCES payroll_periods(id) ON DELETE CASCADE,
    regular_hours       DECIMAL(6,2) DEFAULT 0 CHECK (regular_hours >= 0),
    overtime_hours      DECIMAL(6,2) DEFAULT 0 CHECK (overtime_hours >= 0),
    created_at          TIMESTAMP DEFAULT NOW(),
    updated_at          TIMESTAMP DEFAULT NOW(),
    
    CONSTRAINT unique_timesheet UNIQUE(employee_id, payroll_period_id)
);

CREATE INDEX idx_timesheets_employee ON timesheets(employee_id);
CREATE INDEX idx_timesheets_period ON timesheets(payroll_period_id);

-- ======================
-- 7. PAYROLL ADJUSTMENTS
-- ======================
CREATE TABLE payroll_adjustments (
    id                  SERIAL PRIMARY KEY,
    employee_id         INTEGER NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    payroll_period_id   INTEGER NOT NULL REFERENCES payroll_periods(id) ON DELETE CASCADE,
    adjustment_type     VARCHAR(50) NOT NULL,
    amount              DECIMAL(10,2) NOT NULL,
    description         TEXT,
    created_by          VARCHAR(100),
    created_at          TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_adjustments_employee ON payroll_adjustments(employee_id);
CREATE INDEX idx_adjustments_period ON payroll_adjustments(payroll_period_id);
CREATE INDEX idx_adjustments_type ON payroll_adjustments(adjustment_type);

-- ======================
-- 8. PAYROLL RUNS
-- ======================
CREATE TABLE payroll_runs (
    id                  SERIAL PRIMARY KEY,
    employee_id         INTEGER NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    payroll_period_id   INTEGER NOT NULL REFERENCES payroll_periods(id) ON DELETE CASCADE,
    
    -- Earnings
    gross_pay           DECIMAL(12,2) NOT NULL CHECK (gross_pay >= 0),
    regular_pay         DECIMAL(12,2) DEFAULT 0 CHECK (regular_pay >= 0),
    overtime_pay        DECIMAL(12,2) DEFAULT 0 CHECK (overtime_pay >= 0),
    bonuses             DECIMAL(10,2) DEFAULT 0 CHECK (bonuses >= 0),
    
    -- Deductions
    tax_deduction       DECIMAL(10,2) DEFAULT 0 CHECK (tax_deduction >= 0),
    employee_savings    DECIMAL(10,2) DEFAULT 0 CHECK (employee_savings >= 0),
    company_match       DECIMAL(10,2) DEFAULT 0 CHECK (company_match >= 0),
    total_deductions    DECIMAL(10,2) DEFAULT 0 CHECK (total_deductions >= 0),
    
    -- Net
    net_pay             DECIMAL(12,2) NOT NULL,
    
    payment_status      VARCHAR(20) DEFAULT 'pending' CHECK (payment_status IN ('pending', 'paid', 'failed')),
    payment_date        DATE,
    created_at          TIMESTAMP DEFAULT NOW(),
    updated_at          TIMESTAMP DEFAULT NOW(),
    
    CONSTRAINT unique_payroll_run UNIQUE(employee_id, payroll_period_id),
    CONSTRAINT valid_net_pay CHECK (net_pay = gross_pay - total_deductions)
);

CREATE INDEX idx_payroll_runs_employee ON payroll_runs(employee_id);
CREATE INDEX idx_payroll_runs_period ON payroll_runs(payroll_period_id);
CREATE INDEX idx_payroll_runs_status ON payroll_runs(payment_status);

-- ======================
-- 9. AUDIT LOGS
-- ======================
CREATE TABLE audit_logs (
    id                  SERIAL PRIMARY KEY,
    table_name          VARCHAR(50) NOT NULL,
    record_id           INTEGER,
    action              VARCHAR(20) NOT NULL CHECK (action IN ('INSERT', 'UPDATE', 'DELETE')),
    user_name           VARCHAR(100),
    changed_data        JSONB,
    created_at          TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_audit_table ON audit_logs(table_name);
CREATE INDEX idx_audit_record ON audit_logs(record_id);
CREATE INDEX idx_audit_date ON audit_logs(created_at);

-- ======================
-- TRIGGERS FOR UPDATED_AT
-- ======================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to tables with updated_at
CREATE TRIGGER update_employees_updated_at BEFORE UPDATE ON employees
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_timesheets_updated_at BEFORE UPDATE ON timesheets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_payroll_periods_updated_at BEFORE UPDATE ON payroll_periods
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_payroll_runs_updated_at BEFORE UPDATE ON payroll_runs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ======================
-- VIEWS FOR COMMON QUERIES
-- ======================

-- View: Current employee compensation
CREATE OR REPLACE VIEW v_employee_current_compensation AS
SELECT 
    e.id,
    e.employee_number,
    e.first_name,
    e.last_name,
    e.email,
    e.employment_type,
    e.status,
    e.position,
    d.name as department_name,
    c.base_salary,
    c.hourly_rate,
    c.effective_date
FROM employees e
LEFT JOIN departments d ON e.department_id = d.id
LEFT JOIN compensation c ON e.id = c.employee_id AND c.is_current = true
WHERE e.status = 'active';

-- View: Employee year-to-date savings
CREATE OR REPLACE VIEW v_employee_ytd_savings AS
SELECT 
    e.id as employee_id,
    e.employee_number,
    e.first_name,
    e.last_name,
    COALESCE(SUM(es.contribution_amount), 0) as ytd_contributions,
    LEAST(COALESCE(SUM(es.contribution_amount), 0), 10000) as ytd_company_match
FROM employees e
LEFT JOIN employee_savings es ON e.id = es.employee_id 
    AND EXTRACT(YEAR FROM es.contribution_date) = EXTRACT(YEAR FROM CURRENT_DATE)
GROUP BY e.id, e.employee_number, e.first_name, e.last_name;

-- View: Payroll period summary
CREATE OR REPLACE VIEW v_payroll_period_summary AS
SELECT 
    pp.id as period_id,
    pp.start_date,
    pp.end_date,
    pp.pay_date,
    pp.status,
    COUNT(DISTINCT pr.employee_id) as employee_count,
    COALESCE(SUM(pr.gross_pay), 0) as total_gross_pay,
    COALESCE(SUM(pr.total_deductions), 0) as total_deductions,
    COALESCE(SUM(pr.net_pay), 0) as total_net_pay,
    COALESCE(SUM(pr.tax_deduction), 0) as total_tax,
    COALESCE(SUM(pr.employee_savings + pr.company_match), 0) as total_savings
FROM payroll_periods pp
LEFT JOIN payroll_runs pr ON pp.id = pr.payroll_period_id
GROUP BY pp.id, pp.start_date, pp.end_date, pp.pay_date, pp.status;

-- ======================
-- COMMENTS FOR DOCUMENTATION
-- ======================

COMMENT ON TABLE employees IS 'Core employee information and employment details';
COMMENT ON TABLE compensation IS 'Employee compensation records with history tracking';
COMMENT ON TABLE employee_savings IS 'Employee savings contributions for company matching';
COMMENT ON TABLE payroll_periods IS 'Biweekly payroll period definitions';
COMMENT ON TABLE timesheets IS 'Hours worked tracking for hourly employees';
COMMENT ON TABLE payroll_adjustments IS 'Ad-hoc adjustments to payroll (bonuses, deductions, leave)';
COMMENT ON TABLE payroll_runs IS 'Actual payroll calculations per employee per period';
COMMENT ON TABLE audit_logs IS 'Audit trail for all database changes';

COMMENT ON COLUMN compensation.is_current IS 'Only one record per employee should have is_current = true';
COMMENT ON COLUMN payroll_runs.net_pay IS 'Must equal gross_pay - total_deductions (enforced by constraint)';
