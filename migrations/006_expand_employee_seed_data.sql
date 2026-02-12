-- Expand internal employee dataset so the Employees page can validate >10 rows.

INSERT INTO employees (
    employee_number,
    first_name,
    last_name,
    email,
    phone,
    department_id,
    position,
    employment_type,
    status,
    hire_date,
    bank_account_number,
    bank_routing_number
)
VALUES
    (
        'EMP011', 'Evelyn', 'Boyd', 'evelyn.boyd@company.com', '555-0111',
        (SELECT id FROM departments WHERE code = 'ENG' LIMIT 1),
        'Backend Engineer', 'salaried', 'active', '2024-03-01', '1234567900', '021000021'
    ),
    (
        'EMP012', 'Joan', 'Clarke', 'joan.clarke@company.com', '555-0112',
        (SELECT id FROM departments WHERE code = 'QUANT' LIMIT 1),
        'Data Scientist', 'salaried', 'active', '2024-05-13', '1234567901', '021000021'
    ),
    (
        'EMP013', 'Lynn', 'Conway', 'lynn.conway@company.com', '555-0113',
        (SELECT id FROM departments WHERE code = 'INFRA' LIMIT 1),
        'Platform Engineer', 'salaried', 'active', '2024-04-22', '1234567902', '021000021'
    ),
    (
        'EMP014', 'Sophie', 'Wilson', 'sophie.wilson@company.com', '555-0114',
        (SELECT id FROM departments WHERE code = 'ENG' LIMIT 1),
        'Principal Architect', 'salaried', 'active', '2024-06-10', '1234567903', '021000021'
    ),
    (
        'EMP015', 'Karen', 'Spark', 'karen.spark@company.com', '555-0115',
        (SELECT id FROM departments WHERE code = 'OPS' LIMIT 1),
        'Payroll Specialist', 'hourly', 'active', '2024-07-08', '1234567904', '021000021'
    ),
    (
        'EMP016', 'Elena', 'Mills', 'elena.mills@company.com', '555-0116',
        (SELECT id FROM departments WHERE code = 'DES' LIMIT 1),
        'UI Designer', 'hourly', 'active', '2024-08-19', '1234567905', '021000021'
    ),
    (
        'EMP017', 'Naomi', 'Price', 'naomi.price@company.com', '555-0117',
        (SELECT id FROM departments WHERE code = 'PROD' LIMIT 1),
        'Product Advisor', 'contractor', 'active', '2024-09-02', '1234567906', '021000021'
    ),
    (
        'EMP018', 'Mira', 'Solis', 'mira.solis@company.com', '555-0118',
        (SELECT id FROM departments WHERE code = 'PROD' LIMIT 1),
        'Operations Consultant', 'contractor', 'active', '2024-09-16', '1234567907', '021000021'
    )
ON CONFLICT (employee_number) DO NOTHING;

INSERT INTO compensation (employee_id, base_salary, hourly_rate, effective_date, is_current)
SELECT
    e.id,
    CASE
        WHEN e.employee_number = 'EMP011' THEN 1750000.00
        WHEN e.employee_number = 'EMP012' THEN 1680000.00
        WHEN e.employee_number = 'EMP013' THEN 1820000.00
        WHEN e.employee_number = 'EMP014' THEN 2100000.00
        WHEN e.employee_number = 'EMP017' THEN 92000.00
        WHEN e.employee_number = 'EMP018' THEN 88000.00
        ELSE NULL
    END AS base_salary,
    CASE
        WHEN e.employee_number = 'EMP015' THEN 460.00
        WHEN e.employee_number = 'EMP016' THEN 440.00
        ELSE NULL
    END AS hourly_rate,
    COALESCE(e.hire_date, CURRENT_DATE),
    true
FROM employees e
LEFT JOIN compensation c
    ON c.employee_id = e.id
    AND c.is_current = true
WHERE e.employee_number IN (
    'EMP011', 'EMP012', 'EMP013', 'EMP014',
    'EMP015', 'EMP016', 'EMP017', 'EMP018'
)
  AND c.id IS NULL;

-- Give new hourly employees default time in the most recent editable period.
WITH selected_period AS (
    SELECT id
    FROM payroll_periods
    WHERE status IN ('draft', 'processing', 'approved')
    ORDER BY end_date DESC
    LIMIT 1
)
INSERT INTO timesheets (employee_id, payroll_period_id, regular_hours, overtime_hours)
SELECT e.id, sp.id, 80.00, 2.00
FROM employees e
CROSS JOIN selected_period sp
WHERE e.employee_number IN ('EMP015', 'EMP016')
  AND NOT EXISTS (
      SELECT 1
      FROM timesheets t
      WHERE t.employee_id = e.id
        AND t.payroll_period_id = sp.id
  );

-- Seed savings entries so contribution math includes the added employees immediately.
INSERT INTO employee_savings (employee_id, contribution_amount, contribution_date)
SELECT
    e.id,
    CASE
        WHEN e.employee_number IN ('EMP011', 'EMP012', 'EMP013', 'EMP014') THEN 6500.00
        WHEN e.employee_number IN ('EMP015', 'EMP016') THEN 2500.00
        ELSE 3000.00
    END,
    CURRENT_DATE
FROM employees e
WHERE e.employee_number IN (
    'EMP011', 'EMP012', 'EMP013', 'EMP014',
    'EMP015', 'EMP016', 'EMP017', 'EMP018'
)
  AND NOT EXISTS (
      SELECT 1
      FROM employee_savings es
      WHERE es.employee_id = e.id
        AND es.contribution_date = CURRENT_DATE
  );
