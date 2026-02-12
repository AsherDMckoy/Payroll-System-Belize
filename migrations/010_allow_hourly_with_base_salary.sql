-- Allow hourly employees to carry both hourly_rate and annualized base_salary.

ALTER TABLE compensation
DROP CONSTRAINT IF EXISTS salary_or_hourly;

ALTER TABLE compensation
ADD CONSTRAINT salary_or_hourly
CHECK (base_salary IS NOT NULL OR hourly_rate IS NOT NULL);

-- Backfill annualized base salary for hourly employees where missing/zero.
UPDATE compensation c
SET base_salary = ROUND((c.hourly_rate * 80 * 26)::numeric, 2)
FROM employees e
WHERE c.employee_id = e.id
  AND e.employment_type = 'hourly'
  AND c.hourly_rate IS NOT NULL
  AND c.hourly_rate > 0
  AND (c.base_salary IS NULL OR c.base_salary <= 0);
