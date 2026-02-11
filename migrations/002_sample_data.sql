-- Sample Data for Payroll System
-- Run this after 001_initial_schema.sql

-- ======================
-- DEPARTMENTS
-- ======================
INSERT INTO departments (name, code) VALUES
('Engineering', 'ENG'),
('Product', 'PROD'),
('Infrastructure', 'INFRA'),
('Quantitative Programming', 'QUANT'),
('Design', 'DES'),
('Operations', 'OPS');

-- ======================
-- EMPLOYEES
-- ======================
INSERT INTO employees (
    employee_number, first_name, last_name, email, phone,
    department_id, position, employment_type, status, hire_date,
    bank_account_number, bank_routing_number
) VALUES
-- Salaried employees
('EMP001', 'Ada', 'Lovelace', 'ada.lovelace@company.com', '555-0101',
 1, 'Senior Software Engineer', 'salaried', 'active', '2023-01-15',
 '1234567890', '021000021'),

('EMP002', 'Grace', 'Hopper', 'grace.hopper@company.com', '555-0102',
 1, 'Principal Engineer', 'salaried', 'active', '2022-06-01',
 '1234567891', '021000021'),

('EMP003', 'Katherine', 'Johnson', 'katherine.johnson@company.com', '555-0103',
 4, 'Quantitative Analyst', 'salaried', 'active', '2023-03-20',
 '1234567892', '021000021'),

('EMP004', 'Margaret', 'Hamilton', 'margaret.hamilton@company.com', '555-0104',
 1, 'Engineering Manager', 'salaried', 'active', '2021-11-10',
 '1234567893', '021000021'),

-- Hourly employees
('EMP005', 'Hedy', 'Lamarr', 'hedy.lamarr@company.com', '555-0105',
 3, 'Networks Operator', 'hourly', 'active', '2023-07-01',
 '1234567894', '021000021'),

('EMP006', 'Annie', 'Easley', 'annie.easley@company.com', '555-0106',
 3, 'Systems Administrator', 'hourly', 'active', '2023-08-15',
 '1234567895', '021000021'),

-- Contractors
('EMP007', 'Dorothy', 'Vaughan', 'dorothy.vaughan@company.com', '555-0107',
 2, 'Product Consultant', 'contractor', 'active', '2024-01-01',
 '1234567896', '021000021'),

('EMP008', 'Mary', 'Jackson', 'mary.jackson@company.com', '555-0108',
 5, 'UX Designer', 'contractor', 'active', '2024-02-01',
 '1234567897', '021000021'),

('EMP009', 'Radia', 'Perlman', 'radia.perlman@company.com', '555-0109',
 3, 'Network Architect', 'salaried', 'active', '2022-09-15',
 '1234567898', '021000021'),

('EMP010', 'Frances', 'Allen', 'frances.allen@company.com', '555-0110',
 1, 'Compiler Engineer', 'salaried', 'active', '2023-05-01',
 '1234567899', '021000021');

-- ======================
-- COMPENSATION
-- ======================
INSERT INTO compensation (employee_id, base_salary, hourly_rate, effective_date, is_current) VALUES
-- Salaried (annual salaries)
(1, 1852000.00, NULL, '2023-01-15', true),
(2, 2100000.00, NULL, '2024-01-01', true),
(3, 1950000.00, NULL, '2023-03-20', true),
(4, 2250000.00, NULL, '2024-01-01', true),

-- Hourly (hourly rates)
(5, NULL, 450.00, '2023-07-01', true),
(6, NULL, 425.00, '2023-08-15', true),

-- Contractors (biweekly amounts treated as base_salary)
(7, 85000.00, NULL, '2024-01-01', true),
(8, 72000.00, NULL, '2024-02-01', true),

-- More salaried
(9, 2050000.00, NULL, '2022-09-15', true),
(10, 1900000.00, NULL, '2023-05-01', true);

-- ======================
-- EMPLOYEE SAVINGS (YTD contributions)
-- ======================
-- Some employees contributing to savings
INSERT INTO employee_savings (employee_id, contribution_amount, contribution_date) VALUES
-- Ada Lovelace - contributing regularly
(1, 5000.00, '2024-01-15'),
(1, 5000.00, '2024-01-29'),
(1, 5000.00, '2024-02-12'),

-- Grace Hopper - maxing out
(2, 10000.00, '2024-01-15'),
(2, 10000.00, '2024-01-29'),
(2, 10000.00, '2024-02-12'),

-- Katherine Johnson - moderate contributions
(3, 3000.00, '2024-01-15'),
(3, 3000.00, '2024-01-29'),
(3, 3000.00, '2024-02-12'),

-- Margaret Hamilton - high contributions
(4, 8000.00, '2024-01-15'),
(4, 8000.00, '2024-01-29'),
(4, 8000.00, '2024-02-12');

-- ======================
-- PAYROLL PERIODS
-- ======================
INSERT INTO payroll_periods (start_date, end_date, pay_date, status) VALUES
-- Previous periods (paid)
('2024-01-01', '2024-01-14', '2024-01-15', 'paid'),
('2024-01-15', '2024-01-28', '2024-01-29', 'paid'),

-- Current period (approved, ready for payment)
('2024-01-29', '2024-02-11', '2024-02-12', 'approved'),

-- Upcoming period (draft)
('2024-02-12', '2024-02-25', '2024-02-26', 'draft');

-- ======================
-- TIMESHEETS (for hourly employees)
-- ======================
INSERT INTO timesheets (employee_id, payroll_period_id, regular_hours, overtime_hours) VALUES
-- Period 1 (2024-01-01 to 2024-01-14)
(5, 1, 80.0, 5.0),  -- Hedy Lamarr
(6, 1, 80.0, 2.0),  -- Annie Easley

-- Period 2 (2024-01-15 to 2024-01-28)
(5, 2, 80.0, 8.0),
(6, 2, 80.0, 4.0),

-- Period 3 (2024-01-29 to 2024-02-11) - current
(5, 3, 80.0, 3.0),
(6, 3, 80.0, 0.0);

-- ======================
-- PAYROLL ADJUSTMENTS
-- ======================
INSERT INTO payroll_adjustments (
    employee_id, payroll_period_id, adjustment_type, amount, description, created_by
) VALUES
-- Bonus for Ada
(1, 2, 'bonus', 500000.00, 'Q1 Performance Bonus', 'manager@company.com'),

-- Unpaid leave deduction for Katherine
(3, 3, 'leave_unpaid', -15000.00, 'Unpaid leave - 2 days', 'hr@company.com'),

-- Correction for Grace
(2, 1, 'correction', 2500.00, 'Payroll correction from previous period', 'payroll@company.com');

-- ======================
-- PAYROLL RUNS (Period 1 - Paid)
-- ======================
-- Calculate payroll for Period 1

-- Ada Lovelace (Salaried biweekly: $71,230.80)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    1, 1,
    71230.80, 71230.80, 0, 0,
    10684.60, 5000.00, 5000.00, 20684.60,
    50546.20, 'paid', '2024-01-15'
);

-- Grace Hopper (Salaried biweekly: $80,769.20)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    2, 1,
    105769.20, 80769.20, 0, 25000.00,  -- includes correction
    12490.40, 10000.00, 10000.00, 32490.40,
    73278.80, 'paid', '2024-01-15'
);

-- Katherine Johnson (Salaried biweekly: $75,000.00)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    3, 1,
    75000.00, 75000.00, 0, 0,
    11250.00, 3000.00, 3000.00, 17250.00,
    57750.00, 'paid', '2024-01-15'
);

-- Margaret Hamilton (Salaried biweekly: $86,538.50)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    4, 1,
    86538.50, 86538.50, 0, 0,
    12980.80, 8000.00, 8000.00, 28980.80,
    57557.70, 'paid', '2024-01-15'
);

-- Hedy Lamarr (Hourly: 80 * $450 + 5 * $450 * 1.5 = $36,000 + $3,375 = $39,375)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    5, 1,
    39375.00, 36000.00, 3375.00, 0,
    5906.30, 0, 0, 5906.30,
    33468.70, 'paid', '2024-01-15'
);

-- Annie Easley (Hourly: 80 * $425 + 2 * $425 * 1.5 = $34,000 + $1,275 = $35,275)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    6, 1,
    35275.00, 34000.00, 1275.00, 0,
    5291.30, 0, 0, 5291.30,
    29983.70, 'paid', '2024-01-15'
);

-- Dorothy Vaughan (Contractor: biweekly $85,000)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    7, 1,
    85000.00, 85000.00, 0, 0,
    12750.00, 0, 0, 12750.00,
    72250.00, 'paid', '2024-01-15'
);

-- Mary Jackson (Contractor: biweekly $72,000)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    8, 1,
    72000.00, 72000.00, 0, 0,
    10800.00, 0, 0, 10800.00,
    61200.00, 'paid', '2024-01-15'
);

-- Radia Perlman (Salaried biweekly: $78,846.20)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    9, 1,
    78846.20, 78846.20, 0, 0,
    11826.90, 0, 0, 11826.90,
    67019.30, 'paid', '2024-01-15'
);

-- Frances Allen (Salaried biweekly: $73,076.90)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    10, 1,
    73076.90, 73076.90, 0, 0,
    10961.50, 0, 0, 10961.50,
    62115.40, 'paid', '2024-01-15'
);

-- ======================
-- SAMPLE AUDIT LOGS
-- ======================
INSERT INTO audit_logs (table_name, record_id, action, user_name, changed_data) VALUES
('payroll_periods', 1, 'UPDATE', 'system', '{"status": {"old": "draft", "new": "paid"}}'::jsonb),
('payroll_runs', 1, 'INSERT', 'payroll@company.com', '{"employee_id": 1, "net_pay": 50546.20}'::jsonb),
('employees', 1, 'UPDATE', 'hr@company.com', '{"phone": {"old": null, "new": "555-0101"}}'::jsonb);

-- ======================
-- VERIFICATION QUERIES
-- ======================

-- Verify all employees have compensation
SELECT 
    e.employee_number,
    e.first_name,
    e.last_name,
    e.employment_type,
    c.base_salary,
    c.hourly_rate
FROM employees e
LEFT JOIN compensation c ON e.id = c.employee_id AND c.is_current = true
ORDER BY e.id;

-- Verify payroll period totals
SELECT * FROM v_payroll_period_summary ORDER BY start_date;

-- Verify YTD savings
SELECT * FROM v_employee_ytd_savings ORDER BY employee_id;

-- Count records
SELECT 
    'Departments' as table_name, COUNT(*) as count FROM departments
UNION ALL
SELECT 'Employees', COUNT(*) FROM employees
UNION ALL
SELECT 'Compensation', COUNT(*) FROM compensation
UNION ALL
SELECT 'Employee Savings', COUNT(*) FROM employee_savings
UNION ALL
SELECT 'Payroll Periods', COUNT(*) FROM payroll_periods
UNION ALL
SELECT 'Timesheets', COUNT(*) FROM timesheets
UNION ALL
SELECT 'Payroll Adjustments', COUNT(*) FROM payroll_adjustments
UNION ALL
SELECT 'Payroll Runs', COUNT(*) FROM payroll_runs
UNION ALL
SELECT 'Audit Logs', COUNT(*) FROM audit_logs;
