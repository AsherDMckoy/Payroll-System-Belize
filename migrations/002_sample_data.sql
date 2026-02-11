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
(1, 185200.00, NULL, '2023-01-15', true),
(2, 210000.00, NULL, '2024-01-01', true),
(3, 195000.00, NULL, '2023-03-20', true),
(4, 225000.00, NULL, '2024-01-01', true),

-- Hourly (hourly rates)
(5, NULL, 45.00, '2023-07-01', true),
(6, NULL, 42.50, '2023-08-15', true),

-- Contractors (biweekly amounts treated as base_salary)
(7, 8500.00, NULL, '2024-01-01', true),
(8, 7200.00, NULL, '2024-02-01', true),

-- More salaried
(9, 205000.00, NULL, '2022-09-15', true),
(10, 190000.00, NULL, '2023-05-01', true);

-- ======================
-- EMPLOYEE SAVINGS (YTD contributions)
-- ======================
-- Some employees contributing to savings
INSERT INTO employee_savings (employee_id, contribution_amount, contribution_date) VALUES
-- Ada Lovelace - contributing regularly
(1, 500.00, '2024-01-15'),
(1, 500.00, '2024-01-29'),
(1, 500.00, '2024-02-12'),

-- Grace Hopper - maxing out
(2, 1000.00, '2024-01-15'),
(2, 1000.00, '2024-01-29'),
(2, 1000.00, '2024-02-12'),

-- Katherine Johnson - moderate contributions
(3, 300.00, '2024-01-15'),
(3, 300.00, '2024-01-29'),
(3, 300.00, '2024-02-12'),

-- Margaret Hamilton - high contributions
(4, 800.00, '2024-01-15'),
(4, 800.00, '2024-01-29'),
(4, 800.00, '2024-02-12');

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
(1, 2, 'bonus', 5000.00, 'Q1 Performance Bonus', 'manager@company.com'),

-- Unpaid leave deduction for Katherine
(3, 3, 'leave_unpaid', -1500.00, 'Unpaid leave - 2 days', 'hr@company.com'),

-- Correction for Grace
(2, 1, 'correction', 250.00, 'Payroll correction from previous period', 'payroll@company.com');

-- ======================
-- PAYROLL RUNS (Period 1 - Paid)
-- ======================
-- Calculate payroll for Period 1

-- Ada Lovelace (Salaried: $185,200 / 26 = $7,123.08)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    1, 1,
    7123.08, 7123.08, 0, 0,
    1068.46, 500.00, 500.00, 2068.46,
    5054.62, 'paid', '2024-01-15'
);

-- Grace Hopper (Salaried: $210,000 / 26 = $8,076.92)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    2, 1,
    8326.92, 8076.92, 0, 250.00,  -- includes correction
    1249.04, 1000.00, 1000.00, 3249.04,
    5077.88, 'paid', '2024-01-15'
);

-- Katherine Johnson (Salaried: $195,000 / 26 = $7,500)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    3, 1,
    7500.00, 7500.00, 0, 0,
    1125.00, 300.00, 300.00, 1725.00,
    5775.00, 'paid', '2024-01-15'
);

-- Margaret Hamilton (Salaried: $225,000 / 26 = $8,653.85)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    4, 1,
    8653.85, 8653.85, 0, 0,
    1298.08, 800.00, 800.00, 2898.08,
    5755.77, 'paid', '2024-01-15'
);

-- Hedy Lamarr (Hourly: 80 * $45 + 5 * $45 * 1.5 = $3,600 + $337.50 = $3,937.50)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    5, 1,
    3937.50, 3600.00, 337.50, 0,
    590.63, 0, 0, 590.63,
    3346.87, 'paid', '2024-01-15'
);

-- Annie Easley (Hourly: 80 * $42.50 + 2 * $42.50 * 1.5 = $3,400 + $127.50 = $3,527.50)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    6, 1,
    3527.50, 3400.00, 127.50, 0,
    529.13, 0, 0, 529.13,
    2998.37, 'paid', '2024-01-15'
);

-- Dorothy Vaughan (Contractor: biweekly $8,500)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    7, 1,
    8500.00, 8500.00, 0, 0,
    1275.00, 0, 0, 1275.00,
    7225.00, 'paid', '2024-01-15'
);

-- Mary Jackson (Contractor: biweekly $7,200)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    8, 1,
    7200.00, 7200.00, 0, 0,
    1080.00, 0, 0, 1080.00,
    6120.00, 'paid', '2024-01-15'
);

-- Radia Perlman (Salaried: $205,000 / 26 = $7,884.62)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    9, 1,
    7884.62, 7884.62, 0, 0,
    1182.69, 0, 0, 1182.69,
    6701.93, 'paid', '2024-01-15'
);

-- Frances Allen (Salaried: $190,000 / 26 = $7,307.69)
INSERT INTO payroll_runs (
    employee_id, payroll_period_id,
    gross_pay, regular_pay, overtime_pay, bonuses,
    tax_deduction, employee_savings, company_match, total_deductions,
    net_pay, payment_status, payment_date
) VALUES (
    10, 1,
    7307.69, 7307.69, 0, 0,
    1096.15, 0, 0, 1096.15,
    6211.54, 'paid', '2024-01-15'
);

-- ======================
-- SAMPLE AUDIT LOGS
-- ======================
INSERT INTO audit_logs (table_name, record_id, action, user_name, changed_data) VALUES
('payroll_periods', 1, 'UPDATE', 'system', '{"status": {"old": "draft", "new": "paid"}}'::jsonb),
('payroll_runs', 1, 'INSERT', 'payroll@company.com', '{"employee_id": 1, "net_pay": 5054.62}'::jsonb),
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
