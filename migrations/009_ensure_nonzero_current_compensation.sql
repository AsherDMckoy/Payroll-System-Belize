-- Ensure every employee has non-zero current compensation for payroll calculations.

-- 1) Backfill missing current compensation rows.
INSERT INTO compensation (employee_id, base_salary, hourly_rate, effective_date, is_current)
SELECT
    e.id,
    CASE
        WHEN e.employment_type = 'salaried' THEN 1800000.00
        WHEN e.employment_type = 'contractor' THEN 85000.00
        ELSE NULL
    END AS base_salary,
    CASE
        WHEN e.employment_type = 'hourly' THEN 400.00
        ELSE NULL
    END AS hourly_rate,
    COALESCE(e.hire_date, CURRENT_DATE) AS effective_date,
    true
FROM employees e
LEFT JOIN compensation c
    ON c.employee_id = e.id
    AND c.is_current = true
WHERE c.id IS NULL;

-- 2) Fix any zero/null salaried or contractor base salary.
UPDATE compensation c
SET
    base_salary = CASE
        WHEN e.employment_type = 'salaried' THEN 1800000.00
        WHEN e.employment_type = 'contractor' THEN 85000.00
        ELSE c.base_salary
    END,
    hourly_rate = NULL
FROM employees e
WHERE c.employee_id = e.id
  AND c.is_current = true
  AND e.employment_type IN ('salaried', 'contractor')
  AND (c.base_salary IS NULL OR c.base_salary <= 0);

-- 3) Fix any zero/null hourly rate.
UPDATE compensation c
SET
    base_salary = NULL,
    hourly_rate = 400.00
FROM employees e
WHERE c.employee_id = e.id
  AND c.is_current = true
  AND e.employment_type = 'hourly'
  AND (c.hourly_rate IS NULL OR c.hourly_rate <= 0);
