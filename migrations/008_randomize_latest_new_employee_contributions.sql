-- Randomize the most recent contribution entry per newly added employee.
-- This ensures the Employees page shows varied (non-uniform) contribution values.

WITH target_employees AS (
    SELECT id
    FROM employees
    WHERE employee_number IN (
        'EMP011', 'EMP012', 'EMP013', 'EMP014',
        'EMP015', 'EMP016', 'EMP017', 'EMP018'
    )
),
latest_savings AS (
    SELECT DISTINCT ON (es.employee_id)
        es.id
    FROM employee_savings es
    INNER JOIN target_employees te ON te.id = es.employee_id
    ORDER BY es.employee_id, es.contribution_date DESC, es.id DESC
)
UPDATE employee_savings es
SET contribution_amount = ROUND((1500 + random() * 6500)::numeric, 2)
FROM latest_savings ls
WHERE es.id = ls.id;
