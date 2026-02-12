-- Add payroll period locking metadata for approval workflow.

ALTER TABLE payroll_periods
ADD COLUMN IF NOT EXISTS is_locked BOOLEAN NOT NULL DEFAULT false,
ADD COLUMN IF NOT EXISTS locked_at TIMESTAMP,
ADD COLUMN IF NOT EXISTS locked_by VARCHAR(100);

-- Backfill lock state for periods already approved/paid.
UPDATE payroll_periods
SET
    is_locked = true,
    locked_at = COALESCE(locked_at, NOW()),
    locked_by = COALESCE(locked_by, 'system_migration')
WHERE status IN ('approved', 'paid');

-- Keep lock timestamp consistent with lock flag.
ALTER TABLE payroll_periods
DROP CONSTRAINT IF EXISTS payroll_periods_lock_consistency;

ALTER TABLE payroll_periods
ADD CONSTRAINT payroll_periods_lock_consistency
CHECK (
    (is_locked = false AND locked_at IS NULL)
    OR (is_locked = true AND locked_at IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_payroll_periods_locked ON payroll_periods(is_locked);
