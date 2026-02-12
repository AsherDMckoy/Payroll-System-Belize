-- Ensure compensation is never zero and seed 2026/current-month payroll data.

-- 1) Guarantee each active employee has a current compensation row.
INSERT INTO compensation (employee_id, base_salary, hourly_rate, effective_date, is_current)
SELECT
    e.id,
    CASE
        WHEN e.employment_type = 'salaried' THEN 1800000.00
        WHEN e.employment_type = 'contractor' THEN 80000.00
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
WHERE e.status = 'active'
  AND c.id IS NULL;

-- 2) Normalize any zero/null compensation to safe defaults.
UPDATE compensation c
SET
    base_salary = CASE
        WHEN e.employment_type = 'salaried' THEN 1800000.00
        WHEN e.employment_type = 'contractor' THEN 80000.00
        ELSE c.base_salary
    END,
    hourly_rate = NULL
FROM employees e
WHERE c.employee_id = e.id
  AND c.is_current = true
  AND e.employment_type IN ('salaried', 'contractor')
  AND (c.base_salary IS NULL OR c.base_salary <= 0);

UPDATE compensation c
SET
    base_salary = NULL,
    hourly_rate = 400.00
FROM employees e
WHERE c.employee_id = e.id
  AND c.is_current = true
  AND e.employment_type = 'hourly'
  AND (c.hourly_rate IS NULL OR c.hourly_rate <= 0);

-- 3) Seed current-year payroll periods and hardcoded previous months.
DO $$
DECLARE
    target_year INT := GREATEST(2026, EXTRACT(YEAR FROM CURRENT_DATE)::INT);
    target_month INT := EXTRACT(MONTH FROM CURRENT_DATE)::INT;
    month_start DATE := make_date(target_year, target_month, 1);
    month_last DATE := (month_start + INTERVAL '1 month - 1 day')::DATE;
    current_start DATE;
    current_end DATE;
    current_pay DATE;
    source_paid_period_id INT;
    current_period_id INT;
BEGIN
    -- Hardcoded previous month periods for the target year (Jan 1-14 and Jan 15-28).
    INSERT INTO payroll_periods (start_date, end_date, pay_date, status)
    VALUES (make_date(target_year, 1, 1), make_date(target_year, 1, 14), make_date(target_year, 1, 15), 'paid')
    ON CONFLICT (start_date, end_date) DO UPDATE
    SET pay_date = EXCLUDED.pay_date,
        status = 'paid';

    INSERT INTO payroll_periods (start_date, end_date, pay_date, status)
    VALUES (make_date(target_year, 1, 15), make_date(target_year, 1, 28), make_date(target_year, 1, 29), 'paid')
    ON CONFLICT (start_date, end_date) DO UPDATE
    SET pay_date = EXCLUDED.pay_date,
        status = 'paid';

    -- Use latest paid period with runs as template for hardcoded January payroll data.
    SELECT pp.id
    INTO source_paid_period_id
    FROM payroll_periods pp
    WHERE pp.status = 'paid'
      AND EXISTS (
          SELECT 1
          FROM payroll_runs pr
          WHERE pr.payroll_period_id = pp.id
      )
    ORDER BY pp.pay_date DESC
    LIMIT 1;

    IF source_paid_period_id IS NOT NULL THEN
        INSERT INTO payroll_runs (
            employee_id,
            payroll_period_id,
            gross_pay,
            regular_pay,
            overtime_pay,
            bonuses,
            tax_deduction,
            employee_savings,
            company_match,
            total_deductions,
            net_pay,
            payment_status,
            payment_date
        )
        SELECT
            src.employee_id,
            target_pp.id,
            src.gross_pay,
            src.regular_pay,
            src.overtime_pay,
            src.bonuses,
            src.tax_deduction,
            src.employee_savings,
            src.company_match,
            src.total_deductions,
            src.net_pay,
            'paid',
            target_pp.pay_date
        FROM payroll_runs src
        INNER JOIN payroll_periods target_pp
            ON target_pp.start_date IN (make_date(target_year, 1, 1), make_date(target_year, 1, 15))
        LEFT JOIN payroll_runs existing
            ON existing.employee_id = src.employee_id
            AND existing.payroll_period_id = target_pp.id
        WHERE src.payroll_period_id = source_paid_period_id
          AND existing.id IS NULL;
    END IF;

    -- Current month draft period is generate-ready.
    IF EXTRACT(DAY FROM CURRENT_DATE)::INT <= 14 THEN
        current_start := month_start;
    ELSE
        current_start := (month_start + INTERVAL '14 days')::DATE;
    END IF;

    current_end := LEAST(month_last, (current_start + INTERVAL '13 days')::DATE);
    current_pay := (current_end + INTERVAL '1 day')::DATE;

    INSERT INTO payroll_periods (start_date, end_date, pay_date, status)
    VALUES (current_start, current_end, current_pay, 'draft')
    ON CONFLICT (start_date, end_date) DO UPDATE
    SET pay_date = EXCLUDED.pay_date,
        status = CASE
            WHEN payroll_periods.status = 'paid' THEN payroll_periods.status
            ELSE 'draft'
        END;

    SELECT id
    INTO current_period_id
    FROM payroll_periods
    WHERE start_date = current_start
      AND end_date = current_end
    LIMIT 1;

    IF current_period_id IS NOT NULL THEN
        -- Seed current period timesheets for active hourly employees.
        INSERT INTO timesheets (employee_id, payroll_period_id, regular_hours, overtime_hours)
        SELECT e.id, current_period_id, 80.00, 0.00
        FROM employees e
        WHERE e.status = 'active'
          AND e.employment_type = 'hourly'
          AND NOT EXISTS (
              SELECT 1
              FROM timesheets t
              WHERE t.employee_id = e.id
                AND t.payroll_period_id = current_period_id
          );

        -- Seed one contribution entry in the current period from each employee's latest amount.
        INSERT INTO employee_savings (employee_id, contribution_amount, contribution_date)
        SELECT latest.employee_id, latest.contribution_amount, current_start
        FROM (
            SELECT DISTINCT ON (employee_id)
                employee_id,
                contribution_amount
            FROM employee_savings
            ORDER BY employee_id, contribution_date DESC, id DESC
        ) latest
        WHERE NOT EXISTS (
            SELECT 1
            FROM employee_savings es
            WHERE es.employee_id = latest.employee_id
              AND es.contribution_date = current_start
        );
    END IF;
END $$;
