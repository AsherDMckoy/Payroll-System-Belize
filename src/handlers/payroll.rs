use axum::{
    extract::{Query, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

use crate::handlers::auth::AuthenticatedUser;
use crate::models::api::{
    ApprovePayrollRequest, ApprovePayrollResponse, ExecutePayrollRequest, ExecutePayrollResponse,
    GeneratePayrollRequest, GeneratePayrollResponse, PayrollAuditEvent, PayrollAuditTrailResponse,
    PayrollPeriodDetailsResponse, PayrollPeriodEmployeeRow, PayrollPeriodItem,
    PayrollPeriodsResponse,
};

const BIWEEKLY_PERIODS_PER_YEAR: i64 = 26;
const TAX_FREE_ALLOWANCE_ANNUAL: i64 = 29_000;
const TAX_RATE_PERCENT: i64 = 15;
const COMPANY_MATCH_CAP: i64 = 10_000;
const OVERTIME_MULTIPLIER_X10: i64 = 15; // 1.5x represented as 15 / 10
const HOURLY_FALLBACK_HOURS_PER_PERIOD: i64 = 80;
const MIN_NET_PAY_CENTS: i64 = 1; // keep a positive non-zero net pay when gross pay > 0

type ApiResult<T> = Result<T, (StatusCode, String)>;

#[derive(FromRow)]
struct PayrollPeriodRow {
    id: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
    pay_date: NaiveDate,
    status: String,
    is_locked: bool,
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

#[derive(FromRow)]
struct CompanyMatchTotalsRow {
    matched_to_date: Decimal,
}

#[derive(FromRow)]
struct PayrollRunsSummaryRow {
    run_count: i64,
    total_net_pay: Decimal,
}

#[derive(FromRow)]
struct PayrollAuditDbRow {
    id: i32,
    created_at: chrono::NaiveDateTime,
    action: String,
    event_name: Option<String>,
    user_name: Option<String>,
    record_id: Option<i32>,
    changed_data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PayrollPeriodDetailsQuery {
    pub period_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PayrollAuditQuery {
    pub period_id: Option<i32>,
    pub limit: Option<u16>,
}

#[derive(FromRow)]
struct PayrollPeriodTotalsRow {
    gross_pay: Decimal,
    total_deductions: Decimal,
    net_pay: Decimal,
}

#[derive(FromRow)]
struct PayrollPeriodEmployeeDbRow {
    first_name: String,
    last_name: String,
    position: Option<String>,
    department: Option<String>,
    payment_status: Option<String>,
    net_pay: Option<Decimal>,
    employment_type: String,
    base_salary: Option<Decimal>,
    hourly_rate: Option<Decimal>,
}

/// GET /api/payroll/period-details?period_id=3
pub async fn payroll_period_details(
    State(pool): State<PgPool>,
    Query(q): Query<PayrollPeriodDetailsQuery>,
) -> ApiResult<Json<PayrollPeriodDetailsResponse>> {
    let period = if let Some(period_id) = q.period_id {
        sqlx::query_as::<_, PayrollPeriodRow>(
            r#"
            SELECT id, start_date, end_date, pay_date, status, is_locked
            FROM payroll_periods
            WHERE id = $1
            "#,
        )
        .bind(period_id)
        .fetch_optional(&pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                "Payroll period not found".to_string(),
            )
        })?
    } else {
        sqlx::query_as::<_, PayrollPeriodRow>(
            r#"
            SELECT id, start_date, end_date, pay_date, status, is_locked
            FROM payroll_periods
            ORDER BY end_date DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                "No payroll periods found".to_string(),
            )
        })?
    };

    let totals = sqlx::query_as::<_, PayrollPeriodTotalsRow>(
        r#"
        SELECT
            COALESCE(SUM(gross_pay), 0) AS gross_pay,
            COALESCE(SUM(total_deductions), 0) AS total_deductions,
            COALESCE(SUM(net_pay), 0) AS net_pay
        FROM payroll_runs
        WHERE payroll_period_id = $1
        "#,
    )
    .bind(period.id)
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?;

    let run_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM payroll_runs WHERE payroll_period_id = $1",
    )
    .bind(period.id)
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?;

    let employee_rows = sqlx::query_as::<_, PayrollPeriodEmployeeDbRow>(
        r#"
        SELECT
            e.first_name,
            e.last_name,
            e.position,
            d.name AS department,
            pr.payment_status,
            pr.net_pay,
            e.employment_type,
            c.base_salary,
            c.hourly_rate
        FROM employees e
        LEFT JOIN departments d ON d.id = e.department_id
        LEFT JOIN compensation c
            ON c.employee_id = e.id
            AND c.is_current = true
        LEFT JOIN payroll_runs pr
            ON pr.employee_id = e.id
            AND pr.payroll_period_id = $1
        WHERE e.status = 'active'
        ORDER BY e.last_name, e.first_name
        "#,
    )
    .bind(period.id)
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    let employees = employee_rows
        .into_iter()
        .map(|row| PayrollPeriodEmployeeRow {
            employee_name: format!("{} {}", row.first_name, row.last_name),
            position: row.position.unwrap_or_else(|| "Unassigned".to_string()),
            department: row.department.unwrap_or_else(|| "Unassigned".to_string()),
            net_salary: decimal_to_f64(row.net_pay.unwrap_or_else(|| {
                estimated_net_pay(
                    row.employment_type.as_str(),
                    row.base_salary,
                    row.hourly_rate,
                )
            })),
            payment_status: row
                .payment_status
                .as_deref()
                .map(title_case_status)
                .unwrap_or_else(|| "Pending".to_string()),
        })
        .collect::<Vec<_>>();

    Ok(Json(PayrollPeriodDetailsResponse {
        period_id: period.id,
        start_date: period.start_date.format("%Y-%m-%d").to_string(),
        end_date: period.end_date.format("%Y-%m-%d").to_string(),
        pay_date: period.pay_date.format("%Y-%m-%d").to_string(),
        status: title_case_status(period.status.as_str()),
        is_locked: period.is_locked,
        can_generate: !period.is_locked && period.status != "paid",
        can_approve: !period.is_locked && period.status == "processing" && run_count > 0,
        can_execute: period.is_locked && period.status == "approved" && run_count > 0,
        total_employees: employees.len() as u32,
        gross_pay: decimal_to_f64(totals.gross_pay),
        deductions: decimal_to_f64(totals.total_deductions),
        net_pay: decimal_to_f64(totals.net_pay),
        employees,
    }))
}

/// GET /api/payroll/periods
pub async fn payroll_periods(
    State(pool): State<PgPool>,
) -> ApiResult<Json<PayrollPeriodsResponse>> {
    let current_year = Utc::now().year();
    let mut periods = sqlx::query_as::<_, PayrollPeriodRow>(
        r#"
        SELECT id, start_date, end_date, pay_date, status, is_locked
        FROM payroll_periods
        WHERE EXTRACT(YEAR FROM start_date)::INT = $1
        ORDER BY start_date DESC
        "#,
    )
    .bind(current_year)
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    if periods.is_empty() {
        periods = sqlx::query_as::<_, PayrollPeriodRow>(
            r#"
        SELECT id, start_date, end_date, pay_date, status, is_locked
        FROM payroll_periods
        ORDER BY start_date DESC
        "#,
        )
        .fetch_all(&pool)
        .await
        .map_err(internal_error)?;
    }

    let items = periods
        .into_iter()
        .map(|p| PayrollPeriodItem {
            id: p.id,
            start_date: p.start_date.format("%Y-%m-%d").to_string(),
            end_date: p.end_date.format("%Y-%m-%d").to_string(),
            pay_date: p.pay_date.format("%Y-%m-%d").to_string(),
            status: p.status,
            is_locked: p.is_locked,
        })
        .collect();

    Ok(Json(PayrollPeriodsResponse { periods: items }))
}

/// POST /api/payroll/generate
pub async fn generate_payroll(
    State(pool): State<PgPool>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<GeneratePayrollRequest>,
) -> ApiResult<Json<GeneratePayrollResponse>> {
    if !user.can_manage_payroll() {
        return Err((
            StatusCode::FORBIDDEN,
            "You are not allowed to generate payroll".to_string(),
        ));
    }

    if req.payroll_period_id <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid payroll_period_id".to_string(),
        ));
    }

    let requested_by = user.username.as_str();

    let mut tx = pool.begin().await.map_err(internal_error)?;

    let period = sqlx::query_as::<_, PayrollPeriodRow>(
        r#"
        SELECT id, start_date, end_date, pay_date, status, is_locked
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

    if period.is_locked {
        return Err((
            StatusCode::CONFLICT,
            "Payroll period is locked. Unlocking is required before regenerating.".to_string(),
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

    insert_audit_log(
        &mut tx,
        "payroll_periods",
        Some(period.id),
        "UPDATE",
        requested_by,
        json!({
            "event": "payroll_processing_started",
            "payroll_period_id": period.id,
            "force_recalculate": req.force_recalculate,
        }),
    )
    .await?;

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

        let (regular_pay, overtime_pay) = match employee.employment_type.as_str() {
            "salaried" => {
                let annual_salary = employee.base_salary.unwrap_or(Decimal::ZERO);
                let regular = annual_salary / periods_per_year;
                (regular, Decimal::ZERO)
            }
            "hourly" => {
                let hourly_rate = employee.hourly_rate.unwrap_or(Decimal::ZERO);
                let regular = regular_hours * hourly_rate;
                let overtime = overtime_hours * hourly_rate * overtime_multiplier;
                (regular, overtime)
            }
            "contractor" => {
                // In current schema/sample data this is stored as biweekly amount.
                let biweekly_amount = employee.base_salary.unwrap_or(Decimal::ZERO);
                (biweekly_amount, Decimal::ZERO)
            }
            _ => (Decimal::ZERO, Decimal::ZERO),
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
        let tax_deduction = biweekly_tax(gross_pay);
        let minimum_net = if gross_pay > Decimal::ZERO {
            Decimal::new(MIN_NET_PAY_CENTS, 2)
        } else {
            Decimal::ZERO
        };
        let max_deductions = (gross_pay - minimum_net).max(Decimal::ZERO);

        let fixed_deductions = money(tax_deduction + adjustment_sums.deduction_adjustments);
        let savings_capacity = (max_deductions - fixed_deductions).max(Decimal::ZERO);
        let employee_savings = money(money(savings).min(savings_capacity));

        let ytd_company_match = sqlx::query_as::<_, CompanyMatchTotalsRow>(
            r#"
            SELECT
                COALESCE(SUM(pr.company_match), 0) AS matched_to_date
            FROM payroll_runs pr
            INNER JOIN payroll_periods pp ON pp.id = pr.payroll_period_id
            WHERE pr.employee_id = $1
              AND pr.payroll_period_id <> $2
              AND EXTRACT(YEAR FROM pp.pay_date)::INT = $3
              AND pp.pay_date < $4
            "#,
        )
        .bind(employee.id)
        .bind(period.id)
        .bind(period.pay_date.year())
        .bind(period.pay_date)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal_error)?;

        let company_match =
            capped_company_match(employee_savings, ytd_company_match.matched_to_date);

        let mut deductions_total = money(fixed_deductions + employee_savings);
        if deductions_total > max_deductions {
            deductions_total = max_deductions;
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

    insert_audit_log(
        &mut tx,
        "payroll_periods",
        Some(period.id),
        "UPDATE",
        requested_by,
        json!({
            "event": "payroll_generated",
            "payroll_period_id": period.id,
            "employees_processed": employees.len(),
            "total_gross_pay": money(total_gross).to_string(),
            "total_deductions": money(total_deductions).to_string(),
            "total_net_pay": money(total_net).to_string(),
            "status": "processing",
        }),
    )
    .await?;

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(GeneratePayrollResponse {
        payroll_period_id: period.id,
        pay_date: period.pay_date.format("%Y-%m-%d").to_string(),
        employees_processed: employees.len(),
        total_gross_pay: money(total_gross).to_string(),
        total_deductions: money(total_deductions).to_string(),
        total_net_pay: money(total_net).to_string(),
        status: "processing".to_string(),
        is_locked: false,
    }))
}

/// POST /api/payroll/approve
pub async fn approve_payroll(
    State(pool): State<PgPool>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<ApprovePayrollRequest>,
) -> ApiResult<Json<ApprovePayrollResponse>> {
    if !user.can_manage_payroll() {
        return Err((
            StatusCode::FORBIDDEN,
            "You are not allowed to approve payroll".to_string(),
        ));
    }

    if req.payroll_period_id <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid payroll_period_id".to_string(),
        ));
    }

    let requested_by = user.username.as_str();

    let mut tx = pool.begin().await.map_err(internal_error)?;

    let period = sqlx::query_as::<_, PayrollPeriodRow>(
        r#"
        SELECT id, start_date, end_date, pay_date, status, is_locked
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
            "Payroll period has already been paid".to_string(),
        ));
    }

    if period.status == "draft" {
        return Err((
            StatusCode::CONFLICT,
            "Generate payroll before approval.".to_string(),
        ));
    }

    if period.is_locked {
        return Err((
            StatusCode::CONFLICT,
            "Payroll period is already locked/approved.".to_string(),
        ));
    }

    if period.status != "processing" && period.status != "approved" {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "Payroll period in '{}' state cannot be approved.",
                period.status
            ),
        ));
    }

    let run_summary = sqlx::query_as::<_, PayrollRunsSummaryRow>(
        r#"
        SELECT
            COUNT(*) AS run_count,
            COALESCE(SUM(net_pay), 0) AS total_net_pay
        FROM payroll_runs
        WHERE payroll_period_id = $1
        "#,
    )
    .bind(period.id)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal_error)?;

    if run_summary.run_count <= 0 {
        return Err((
            StatusCode::CONFLICT,
            "No payroll runs found for this period. Generate payroll first.".to_string(),
        ));
    }

    sqlx::query(
        r#"
        UPDATE payroll_periods
        SET status = 'approved',
            is_locked = true,
            locked_at = NOW(),
            locked_by = $2
        WHERE id = $1
        "#,
    )
    .bind(period.id)
    .bind(requested_by)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    insert_audit_log(
        &mut tx,
        "payroll_periods",
        Some(period.id),
        "UPDATE",
        requested_by,
        json!({
            "event": "payroll_approved",
            "payroll_period_id": period.id,
            "employees_ready": run_summary.run_count,
            "total_net_pay": money(run_summary.total_net_pay).to_string(),
            "status": "approved",
            "is_locked": true,
        }),
    )
    .await?;

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(ApprovePayrollResponse {
        payroll_period_id: period.id,
        pay_date: period.pay_date.format("%Y-%m-%d").to_string(),
        employees_ready: run_summary.run_count.max(0) as usize,
        total_net_pay: money(run_summary.total_net_pay).to_string(),
        status: "approved".to_string(),
        is_locked: true,
    }))
}

/// POST /api/payroll/execute
pub async fn execute_payroll(
    State(pool): State<PgPool>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<ExecutePayrollRequest>,
) -> ApiResult<Json<ExecutePayrollResponse>> {
    if !user.can_manage_payroll() {
        return Err((
            StatusCode::FORBIDDEN,
            "You are not allowed to execute payroll".to_string(),
        ));
    }

    if req.payroll_period_id <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid payroll_period_id".to_string(),
        ));
    }

    let requested_by = user.username.as_str();

    let mut tx = pool.begin().await.map_err(internal_error)?;

    let period = sqlx::query_as::<_, PayrollPeriodRow>(
        r#"
        SELECT id, start_date, end_date, pay_date, status, is_locked
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
            "Payroll period has already been paid".to_string(),
        ));
    }

    if period.status != "approved" {
        return Err((
            StatusCode::CONFLICT,
            "Only approved payroll periods can be executed".to_string(),
        ));
    }

    if !period.is_locked {
        return Err((
            StatusCode::CONFLICT,
            "Payroll period must be locked before execution. Approve payroll first.".to_string(),
        ));
    }

    let run_summary = sqlx::query_as::<_, PayrollRunsSummaryRow>(
        r#"
        SELECT
            COUNT(*) AS run_count,
            COALESCE(SUM(net_pay), 0) AS total_net_pay
        FROM payroll_runs
        WHERE payroll_period_id = $1
        "#,
    )
    .bind(period.id)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal_error)?;

    if run_summary.run_count <= 0 {
        return Err((
            StatusCode::CONFLICT,
            "No payroll runs found for this period. Generate payroll first.".to_string(),
        ));
    }

    sqlx::query(
        r#"
        UPDATE payroll_runs
        SET payment_status = 'paid',
            payment_date = $2
        WHERE payroll_period_id = $1
          AND payment_status <> 'paid'
        "#,
    )
    .bind(period.id)
    .bind(period.pay_date)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        r#"
        UPDATE payroll_periods
        SET status = 'paid',
            is_locked = true
        WHERE id = $1
        "#,
    )
    .bind(period.id)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    insert_audit_log(
        &mut tx,
        "payroll_periods",
        Some(period.id),
        "UPDATE",
        requested_by,
        json!({
            "event": "payroll_paid",
            "payroll_period_id": period.id,
            "employees_paid": run_summary.run_count,
            "total_net_pay": money(run_summary.total_net_pay).to_string(),
            "payment_date": period.pay_date,
            "status": "paid",
        }),
    )
    .await?;

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(ExecutePayrollResponse {
        payroll_period_id: period.id,
        pay_date: period.pay_date.format("%Y-%m-%d").to_string(),
        employees_paid: run_summary.run_count.max(0) as usize,
        total_net_pay: money(run_summary.total_net_pay).to_string(),
        status: "paid".to_string(),
        is_locked: true,
    }))
}

/// GET /api/payroll/audit?period_id=3&limit=50
pub async fn payroll_audit_trail(
    State(pool): State<PgPool>,
    Query(q): Query<PayrollAuditQuery>,
) -> ApiResult<Json<PayrollAuditTrailResponse>> {
    let limit = q.limit.unwrap_or(50).clamp(1, 200) as i64;

    let rows = sqlx::query_as::<_, PayrollAuditDbRow>(
        r#"
        SELECT
            id,
            created_at,
            action,
            changed_data ->> 'event' AS event_name,
            user_name,
            record_id,
            changed_data
        FROM audit_logs
        WHERE table_name = 'payroll_periods'
          AND ($1::INT IS NULL OR record_id = $1)
        ORDER BY created_at DESC, id DESC
        LIMIT $2
        "#,
    )
    .bind(q.period_id)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    let events = rows
        .into_iter()
        .map(|row| PayrollAuditEvent {
            id: row.id,
            created_at: row.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            action: row.action,
            event: row
                .event_name
                .unwrap_or_else(|| "payroll_event".to_string()),
            user_name: row.user_name.unwrap_or_else(|| "system".to_string()),
            payroll_period_id: row.record_id,
            details: row.changed_data.unwrap_or_else(|| json!({})),
        })
        .collect::<Vec<_>>();

    Ok(Json(PayrollAuditTrailResponse { events }))
}

fn money(value: Decimal) -> Decimal {
    value.round_dp(2)
}

fn biweekly_tax(gross_biweekly: Decimal) -> Decimal {
    let annualized_income = gross_biweekly * Decimal::from(BIWEEKLY_PERIODS_PER_YEAR);
    let allowance = Decimal::from(TAX_FREE_ALLOWANCE_ANNUAL);
    let taxable_income = if annualized_income > allowance {
        annualized_income - allowance
    } else {
        Decimal::ZERO
    };
    let rate = Decimal::new(TAX_RATE_PERCENT, 2); // 0.15
    money((taxable_income * rate) / Decimal::from(BIWEEKLY_PERIODS_PER_YEAR))
}

fn capped_company_match(employee_savings: Decimal, matched_to_date: Decimal) -> Decimal {
    let remaining = (Decimal::from(COMPANY_MATCH_CAP) - matched_to_date).max(Decimal::ZERO);
    money(employee_savings.max(Decimal::ZERO).min(remaining))
}

fn estimated_net_pay(
    employment_type: &str,
    base_salary: Option<Decimal>,
    hourly_rate: Option<Decimal>,
) -> Decimal {
    let periods_per_year = Decimal::from(BIWEEKLY_PERIODS_PER_YEAR);
    let gross_biweekly = match employment_type {
        "salaried" => {
            let annual = base_salary.unwrap_or(Decimal::ZERO);
            annual / periods_per_year
        }
        "hourly" => {
            let hourly = hourly_rate.unwrap_or(Decimal::ZERO);
            hourly * Decimal::from(HOURLY_FALLBACK_HOURS_PER_PERIOD)
        }
        "contractor" => base_salary.unwrap_or(Decimal::ZERO),
        _ => Decimal::ZERO,
    };

    (gross_biweekly - biweekly_tax(gross_biweekly))
        .max(Decimal::ZERO)
        .round_dp(2)
}

fn decimal_to_f64(value: Decimal) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

fn title_case_status(status: &str) -> String {
    let mut chars = status.chars();
    match chars.next() {
        Some(first) => {
            format!(
                "{}{}",
                first.to_ascii_uppercase(),
                chars.as_str().to_ascii_lowercase()
            )
        }
        None => "Unknown".to_string(),
    }
}

async fn insert_audit_log(
    tx: &mut Transaction<'_, Postgres>,
    table_name: &str,
    record_id: Option<i32>,
    action: &str,
    user_name: &str,
    changed_data: serde_json::Value,
) -> ApiResult<()> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs (table_name, record_id, action, user_name, changed_data)
        VALUES ($1, $2, $3, $4, $5::jsonb)
        "#,
    )
    .bind(table_name)
    .bind(record_id)
    .bind(action)
    .bind(user_name)
    .bind(changed_data)
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;
    Ok(())
}

fn internal_error(err: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {err}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biweekly_tax_follows_flat_rule_after_allowance() {
        let gross_biweekly = Decimal::from(100_000) / Decimal::from(BIWEEKLY_PERIODS_PER_YEAR);
        let tax = biweekly_tax(gross_biweekly);

        assert_eq!(tax, Decimal::new(40962, 2));
    }

    #[test]
    fn company_match_is_capped_by_remaining_annual_limit() {
        let this_period_savings = Decimal::from(8_000);
        let already_matched = Decimal::from(7_500);
        let company_match = capped_company_match(this_period_savings, already_matched);

        assert_eq!(company_match, Decimal::from(2_500));
    }

    #[test]
    fn company_match_is_zero_when_cap_is_exhausted() {
        let this_period_savings = Decimal::from(2_000);
        let already_matched = Decimal::from(10_000);
        let company_match = capped_company_match(this_period_savings, already_matched);

        assert_eq!(company_match, Decimal::ZERO);
    }
}
