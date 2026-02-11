use axum::{extract::State, http::StatusCode, Json};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};

use crate::models::api::{
    GeneratePayrollRequest, GeneratePayrollResponse, PayrollPeriodItem, PayrollPeriodsResponse,
};

const BIWEEKLY_PERIODS_PER_YEAR: i64 = 26;
const TAX_FREE_ALLOWANCE_ANNUAL: i64 = 29_000;
const TAX_RATE_PERCENT: i64 = 15;
const OVERTIME_MULTIPLIER_X10: i64 = 15; // 1.5x represented as 15 / 10

type ApiResult<T> = Result<T, (StatusCode, String)>;

#[derive(FromRow)]
struct PayrollPeriodRow {
    id: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
    pay_date: NaiveDate,
    status: String,
}

#[derive(FromRow)]
struct EmployeeCompensationRow {
    id: i32,
    employment_type: String,
    base_salary: Option<Decimal>,
    hourly_rate: Option<Decimal>,
}

#[derive(FromRow)]
struct TimesheetHoursRow {
    regular_hours: Decimal,
    overtime_hours: Decimal,
}

#[derive(FromRow)]
struct AdjustmentSumsRow {
    earnings_adjustments: Decimal,
    deduction_adjustments: Decimal,
}

/// GET /api/payroll/periods
pub async fn payroll_periods(
    State(pool): State<PgPool>,
) -> ApiResult<Json<PayrollPeriodsResponse>> {
    let periods = sqlx::query_as::<_, PayrollPeriodRow>(
        r#"
        SELECT id, start_date, end_date, pay_date, status
        FROM payroll_periods
        ORDER BY start_date DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    let items = periods
        .into_iter()
        .map(|p| PayrollPeriodItem {
            id: p.id,
            start_date: p.start_date.format("%Y-%m-%d").to_string(),
            end_date: p.end_date.format("%Y-%m-%d").to_string(),
            pay_date: p.pay_date.format("%Y-%m-%d").to_string(),
            status: p.status,
        })
        .collect();

    Ok(Json(PayrollPeriodsResponse { periods: items }))
}

/// POST /api/payroll/generate
pub async fn generate_payroll(
    State(pool): State<PgPool>,
    Json(req): Json<GeneratePayrollRequest>,
) -> ApiResult<Json<GeneratePayrollResponse>> {
    if req.payroll_period_id <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid payroll_period_id".to_string(),
        ));
    }

    let mut tx = pool.begin().await.map_err(internal_error)?;

    let period = sqlx::query_as::<_, PayrollPeriodRow>(
        r#"
        SELECT id, start_date, end_date, pay_date, status
        FROM payroll_periods
        WHERE id = $1
        FOR UPDATE
        "#,
    )
    .bind(req.payroll_period_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            "Payroll period not found".to_string(),
        )
    })?;

    if period.status == "paid" {
        return Err((
            StatusCode::CONFLICT,
            "Cannot regenerate payroll for a paid period".to_string(),
        ));
    }

    let existing_runs = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM payroll_runs WHERE payroll_period_id = $1",
    )
    .bind(period.id)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal_error)?;

    if existing_runs > 0 && !req.force_recalculate {
        return Err((
            StatusCode::CONFLICT,
            "Payroll runs already exist for this period. Set force_recalculate=true to regenerate."
                .to_string(),
        ));
    }

    if existing_runs > 0 && req.force_recalculate {
        sqlx::query("DELETE FROM payroll_runs WHERE payroll_period_id = $1")
            .bind(period.id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    sqlx::query("UPDATE payroll_periods SET status = 'processing' WHERE id = $1")
        .bind(period.id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    let employees = sqlx::query_as::<_, EmployeeCompensationRow>(
        r#"
        SELECT
            e.id,
            e.employment_type,
            c.base_salary,
            c.hourly_rate
        FROM employees e
        LEFT JOIN compensation c
            ON c.employee_id = e.id
            AND c.is_current = true
        WHERE e.status = 'active'
        ORDER BY e.id
        "#,
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(internal_error)?;

    let periods_per_year = Decimal::from(BIWEEKLY_PERIODS_PER_YEAR);
    let overtime_multiplier = Decimal::new(OVERTIME_MULTIPLIER_X10, 1); // 1.5

    let mut total_gross = Decimal::ZERO;
    let mut total_deductions = Decimal::ZERO;
    let mut total_net = Decimal::ZERO;

    for employee in &employees {
        let timesheet = sqlx::query_as::<_, TimesheetHoursRow>(
            r#"
            SELECT regular_hours, overtime_hours
            FROM timesheets
            WHERE employee_id = $1
              AND payroll_period_id = $2
            "#,
        )
        .bind(employee.id)
        .bind(period.id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(internal_error)?;

        let regular_hours = timesheet
            .as_ref()
            .map(|t| t.regular_hours)
            .unwrap_or(Decimal::ZERO);
        let overtime_hours = timesheet
            .as_ref()
            .map(|t| t.overtime_hours)
            .unwrap_or(Decimal::ZERO);

        let (regular_pay, overtime_pay, annualized_income) = match employee.employment_type.as_str()
        {
            "salaried" => {
                let annual_salary = employee.base_salary.unwrap_or(Decimal::ZERO);
                let regular = annual_salary / periods_per_year;
                (regular, Decimal::ZERO, annual_salary)
            }
            "hourly" => {
                let hourly_rate = employee.hourly_rate.unwrap_or(Decimal::ZERO);
                let regular = regular_hours * hourly_rate;
                let overtime = overtime_hours * hourly_rate * overtime_multiplier;
                let annualized = (regular + overtime) * periods_per_year;
                (regular, overtime, annualized)
            }
            "contractor" => {
                // In current schema/sample data this is stored as biweekly amount.
                let biweekly_amount = employee.base_salary.unwrap_or(Decimal::ZERO);
                let annualized = biweekly_amount * periods_per_year;
                (biweekly_amount, Decimal::ZERO, annualized)
            }
            _ => (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
        };

        let adjustment_sums = sqlx::query_as::<_, AdjustmentSumsRow>(
            r#"
            SELECT
                COALESCE(SUM(CASE WHEN amount > 0 THEN amount ELSE 0 END), 0) AS earnings_adjustments,
                COALESCE(SUM(CASE WHEN amount < 0 THEN -amount ELSE 0 END), 0) AS deduction_adjustments
            FROM payroll_adjustments
            WHERE employee_id = $1
              AND payroll_period_id = $2
            "#,
        )
        .bind(employee.id)
        .bind(period.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal_error)?;

        let savings = sqlx::query_scalar::<_, Decimal>(
            r#"
            SELECT COALESCE(SUM(contribution_amount), 0)
            FROM employee_savings
            WHERE employee_id = $1
              AND contribution_date >= $2
              AND contribution_date <= $3
            "#,
        )
        .bind(employee.id)
        .bind(period.start_date)
        .bind(period.end_date)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal_error)?;

        let gross_pay = money(regular_pay + overtime_pay + adjustment_sums.earnings_adjustments);
        let tax_deduction = biweekly_tax(annualized_income);
        let employee_savings = money(savings);
        let company_match = employee_savings;

        let mut deductions_total =
            money(tax_deduction + employee_savings + adjustment_sums.deduction_adjustments);
        if deductions_total > gross_pay {
            deductions_total = gross_pay;
        }
        let net_pay = money(gross_pay - deductions_total);

        sqlx::query(
            r#"
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
                payment_status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'pending')
            "#,
        )
        .bind(employee.id)
        .bind(period.id)
        .bind(gross_pay)
        .bind(money(regular_pay))
        .bind(money(overtime_pay))
        .bind(money(adjustment_sums.earnings_adjustments))
        .bind(money(tax_deduction))
        .bind(employee_savings)
        .bind(company_match)
        .bind(deductions_total)
        .bind(net_pay)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

        total_gross += gross_pay;
        total_deductions += deductions_total;
        total_net += net_pay;
    }

    sqlx::query("UPDATE payroll_periods SET status = 'approved' WHERE id = $1")
        .bind(period.id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(GeneratePayrollResponse {
        payroll_period_id: period.id,
        pay_date: period.pay_date.format("%Y-%m-%d").to_string(),
        employees_processed: employees.len(),
        total_gross_pay: money(total_gross).to_string(),
        total_deductions: money(total_deductions).to_string(),
        total_net_pay: money(total_net).to_string(),
        status: "approved".to_string(),
    }))
}

fn money(value: Decimal) -> Decimal {
    value.round_dp(2)
}

fn biweekly_tax(annualized_income: Decimal) -> Decimal {
    let allowance = Decimal::from(TAX_FREE_ALLOWANCE_ANNUAL);
    let taxable_income = if annualized_income > allowance {
        annualized_income - allowance
    } else {
        Decimal::ZERO
    };
    let rate = Decimal::new(TAX_RATE_PERCENT, 2); // 0.15
    money((taxable_income * rate) / Decimal::from(BIWEEKLY_PERIODS_PER_YEAR))
}

fn internal_error(err: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {err}"),
    )
}
