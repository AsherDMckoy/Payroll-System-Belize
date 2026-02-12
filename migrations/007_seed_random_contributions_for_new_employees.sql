-- Seed randomized contribution amounts for newly-added employees (EMP011-EMP018).
-- This makes contribution figures non-zero before payroll runs are generated.

WITH target_employees AS (
    SELECT id, employee_number
    FROM employees
    WHERE employee_number IN (
        'EMP011', 'EMP012', 'EMP013', 'EMP014',
        'EMP015', 'EMP016', 'EMP017', 'EMP018'
    )
),
randomized AS (
    SELECT
        te.id AS employee_id,
        ROUND((1000 + random() * 7000)::numeric, 2) AS contribution_amount,
        CURRENT_DATE - ((ROW_NUMBER() OVER (ORDER BY te.employee_number)) % 14)::int AS contribution_date
    FROM target_employees te
)
INSERT INTO employee_savings (employee_id, contribution_amount, contribution_date)
SELECT
    r.employee_id,
    r.contribution_amount,
    r.contribution_date
FROM randomized r
WHERE NOT EXISTS (
    SELECT 1
    FROM employee_savings es
    WHERE es.employee_id = r.employee_id
      AND es.contribution_date = r.contribution_date
);
