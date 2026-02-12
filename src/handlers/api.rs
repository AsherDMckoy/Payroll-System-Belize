// API handlers returning JSON.

use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{Datelike, NaiveDate, Utc, Weekday};
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::{FromRow, PgPool};

use crate::models::api::{
    DashboardOverviewResponse, EmployeeRow, EmployeesResponse, MonthBreakdown, PayPeriodInfo,
    PaydayInfo, PayrollBreakdownResponse, RecentActivityRow,
};

const BIWEEKLY_PERIODS_PER_YEAR: i64 = 26;
const TAX_FREE_ALLOWANCE_ANNUAL: i64 = 29_000;
const TAX_RATE_PERCENT: i64 = 15;
const HOURLY_FALLBACK_HOURS_PER_PERIOD: i64 = 80;

type ApiResult<T> = Result<T, (StatusCode, String)>;

#[derive(FromRow)]
struct EmployeeListDbRow {
    first_name: String,
    last_name: String,
    position: Option<String>,
    department: Option<String>,
    status: String,
    employment_type: String,
    base_salary: Option<Decimal>,
    hourly_rate: Option<Decimal>,
    latest_gross_pay: Option<Decimal>,
    latest_net_pay: Option<Decimal>,
    latest_contributions: Option<Decimal>,
    latest_tax_paid: Option<Decimal>,
}

#[derive(FromRow)]
struct MonthlyTotalsRow {
    payroll_total: Decimal,
    deductions_total: Decimal,
    contributions_total: Decimal,
}

#[derive(FromRow)]
struct RecentActivityDbRow {
    first_name: String,
    last_name: String,
    position: String,
    net_pay: Decimal,
    payment_status: String,
}

#[derive(FromRow)]
struct PayPeriodRow {
    id: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
    pay_date: NaiveDate,
    status: String,
}

#[derive(FromRow)]
struct PayPeriodTotalsRow {
    base_salary: Decimal,
    tax_paid: Decimal,
    company_contributions: Decimal,
    total: Decimal,
}

#[derive(FromRow)]
struct YearToDateCompanyContributionsRow {
    company_contributions_ytd: Decimal,
}

#[derive(FromRow)]
struct MonthlyBreakdownDbRow {
    month: i32,
    total: Decimal,
    base: Decimal,
    tax_paid: Decimal,
    company_contributions: Decimal,
}

/// GET /api/dashboard/overview
pub async fn dashboard_overview(
    State(pool): State<PgPool>,
) -> ApiResult<Json<DashboardOverviewResponse>> {
    let today = Utc::now().date_naive();
    let reference_date = resolve_reference_month(&pool, today).await?;
    let (month_start, next_month_start) = month_bounds(reference_date);

    let monthly_totals = sqlx::query_as::<_, MonthlyTotalsRow>(
        r#"
        SELECT
            COALESCE(SUM(pr.gross_pay), 0) AS payroll_total,
            COALESCE(SUM(pr.tax_deduction), 0) AS deductions_total,
            COALESCE(SUM(pr.employee_savings + pr.company_match), 0) AS contributions_total
        FROM payroll_runs pr
        INNER JOIN payroll_periods pp ON pp.id = pr.payroll_period_id
        WHERE pp.pay_date >= $1
          AND pp.pay_date < $2
        "#,
    )
    .bind(month_start)
    .bind(next_month_start)
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?;

    let active_employees =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM employees WHERE status = 'active'")
            .fetch_one(&pool)
            .await
            .map_err(internal_error)?;

    let hires_since_last_month = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM employees
        WHERE hire_date >= $1
          AND hire_date < $2
        "#,
    )
    .bind(month_start)
    .bind(next_month_start)
    .fetch_one(&pool)
    .await
    .map_err(internal_error)?;

    let recent_activity_rows = sqlx::query_as::<_, RecentActivityDbRow>(
        r#"
        SELECT
            e.first_name,
            e.last_name,
            COALESCE(e.position, 'Unassigned') AS position,
            pr.net_pay,
            pr.payment_status
        FROM payroll_runs pr
        INNER JOIN employees e ON e.id = pr.employee_id
        LEFT JOIN payroll_periods pp ON pp.id = pr.payroll_period_id
        ORDER BY COALESCE(pr.payment_date, pp.pay_date) DESC, pr.created_at DESC
        LIMIT 8
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    let focus_period = sqlx::query_as::<_, PayPeriodRow>(
        r#"
        SELECT
            pp.id,
            pp.start_date,
            pp.end_date,
            pp.pay_date,
            pp.status
        FROM payroll_periods pp
        LEFT JOIN payroll_runs pr ON pr.payroll_period_id = pp.id
        GROUP BY pp.id, pp.start_date, pp.end_date, pp.pay_date, pp.status
        ORDER BY
            CASE WHEN COUNT(pr.id) > 0 THEN 0 ELSE 1 END,
            CASE status
                WHEN 'processing' THEN 0
                WHEN 'approved' THEN 1
                WHEN 'draft' THEN 2
                WHEN 'paid' THEN 3
                ELSE 4
            END,
            end_date DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?;

    let pay_period = if let Some(period) = focus_period {
        let period_totals = sqlx::query_as::<_, PayPeriodTotalsRow>(
            r#"
            SELECT
                COALESCE(SUM(regular_pay), 0) AS base_salary,
                COALESCE(SUM(tax_deduction), 0) AS tax_paid,
                COALESCE(SUM(company_match), 0) AS company_contributions,
                COALESCE(SUM(gross_pay), 0) AS total
            FROM payroll_runs
            WHERE payroll_period_id = $1
            "#,
        )
        .bind(period.id)
        .fetch_one(&pool)
        .await
        .map_err(internal_error)?;

        // Keep period values when present; otherwise show YTD contributions so dashboard
        // still reflects actual company-paid contributions for the active year.
        let display_company_contributions = if period_totals.company_contributions > Decimal::ZERO {
            period_totals.company_contributions
        } else {
            let ytd = sqlx::query_as::<_, YearToDateCompanyContributionsRow>(
                r#"
                SELECT
                    COALESCE(SUM(pr.company_match), 0) AS company_contributions_ytd
                FROM payroll_runs pr
                INNER JOIN payroll_periods pp ON pp.id = pr.payroll_period_id
                WHERE EXTRACT(YEAR FROM pp.pay_date)::INT = $1
                  AND pp.pay_date <= $2
                "#,
            )
            .bind(period.pay_date.year())
            .bind(period.pay_date)
            .fetch_one(&pool)
            .await
            .map_err(internal_error)?;
            ytd.company_contributions_ytd
        };

        PayPeriodInfo {
            working_days: business_days(period.start_date, period.end_date),
            working_hours: business_days(period.start_date, period.end_date) * 8,
            label: format!(
                "Current Pay Period ({})",
                title_case_status(period.status.as_str())
            ),
            start_date: short_date(period.start_date),
            end_date: short_date(period.end_date),
            payday: PaydayInfo {
                date: long_date(period.pay_date),
                day_of_month: period.pay_date.day(),
                total_days_in_period: days_in_month(
                    period.pay_date.year(),
                    period.pay_date.month(),
                ),
                base_salary: format_money_label(period_totals.base_salary),
                tax_paid: format_money_label(period_totals.tax_paid),
                company_contributions: format_money_label(display_company_contributions),
                total: format_money_label(period_totals.total),
            },
        }
    } else {
        PayPeriodInfo {
            working_days: 0,
            working_hours: 0,
            label: "Current Pay Period".to_string(),
            start_date: "—".to_string(),
            end_date: "—".to_string(),
            payday: PaydayInfo {
                date: "—".to_string(),
                day_of_month: 0,
                total_days_in_period: 0,
                base_salary: "$0.00".to_string(),
                tax_paid: "$0.00".to_string(),
                company_contributions: "$0.00".to_string(),
                total: "$0.00".to_string(),
            },
        }
    };

    let month_prefix =
        if reference_date.year() == today.year() && reference_date.month() == today.month() {
            "This Month"
        } else {
            "Most Recent"
        };
    let month_label = format!("{} ({})", month_prefix, reference_date.format("%B %Y"));

    Ok(Json(DashboardOverviewResponse {
        payroll_total: monthly_totals.payroll_total.round_dp(2).to_string(),
        payroll_label: month_label.clone(),
        employee_count: active_employees.max(0) as u32,
        employee_delta: format!("{hires_since_last_month} added since last month"),
        deductions_total: monthly_totals.deductions_total.round_dp(2).to_string(),
        deductions_label: month_label.clone(),
        contributions_total: monthly_totals.contributions_total.round_dp(2).to_string(),
        contributions_label: month_label,
        recent_activity: recent_activity_rows
            .into_iter()
            .map(|row| RecentActivityRow {
                employee_name: format!("{} {}", row.first_name, row.last_name),
                position: row.position,
                salary_paid: format_money_label(row.net_pay),
                status: title_case_status(row.payment_status.as_str()),
            })
            .collect(),
        pay_period,
    }))
}

/// GET /api/employees
pub async fn employees_list(State(pool): State<PgPool>) -> ApiResult<Json<EmployeesResponse>> {
    let rows = sqlx::query_as::<_, EmployeeListDbRow>(
        r#"
        SELECT
            e.first_name,
            e.last_name,
            e.position,
            d.name AS department,
            e.status,
            e.employment_type,
            c.base_salary,
            c.hourly_rate,
            lr.gross_pay AS latest_gross_pay,
            lr.net_pay AS latest_net_pay,
            lr.employee_savings AS latest_contributions,
            lr.tax_deduction AS latest_tax_paid
        FROM employees e
        LEFT JOIN departments d
            ON d.id = e.department_id
        LEFT JOIN compensation c
            ON c.employee_id = e.id
            AND c.is_current = true
        LEFT JOIN LATERAL (
            SELECT
                pr.gross_pay,
                pr.net_pay,
                pr.employee_savings,
                pr.tax_deduction
            FROM payroll_runs pr
            LEFT JOIN payroll_periods pp
                ON pp.id = pr.payroll_period_id
            WHERE pr.employee_id = e.id
            ORDER BY pp.end_date DESC NULLS LAST, pr.created_at DESC
            LIMIT 1
        ) lr ON true
        ORDER BY e.last_name, e.first_name
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    let total_employees = rows.len() as u32;
    let active_employees = rows.iter().filter(|r| r.status == "active").count() as u32;
    let inactive_employees = total_employees.saturating_sub(active_employees);

    let employees = rows
        .into_iter()
        .map(|row| {
            let fallback_gross =
                estimated_gross_pay(&row.employment_type, row.base_salary, row.hourly_rate);
            let fallback_net =
                estimated_net_pay(&row.employment_type, row.base_salary, row.hourly_rate);
            EmployeeRow {
                name: format!("{} {}", row.first_name, row.last_name),
                position: row.position.unwrap_or_else(|| "Unassigned".to_string()),
                department: row.department.unwrap_or_else(|| "Unassigned".to_string()),
                gross_salary: decimal_to_f64(row.latest_gross_pay.unwrap_or(fallback_gross)),
                net_salary: decimal_to_f64(row.latest_net_pay.unwrap_or(fallback_net)),
                contributions: decimal_to_f64(row.latest_contributions.unwrap_or(Decimal::ZERO)),
                tax_paid: decimal_to_f64(row.latest_tax_paid.unwrap_or(Decimal::ZERO)),
            }
        })
        .collect();

    Ok(Json(EmployeesResponse {
        total_employees,
        active_employees,
        inactive_employees,
        employees,
    }))
}

#[derive(Debug, Deserialize)]
pub struct PayrollBreakdownQuery {
    pub year: Option<u32>,
}

/// GET /api/payroll-breakdown?year=2026
pub async fn payroll_breakdown(
    State(pool): State<PgPool>,
    Query(q): Query<PayrollBreakdownQuery>,
) -> ApiResult<Json<PayrollBreakdownResponse>> {
    let fallback_year = Utc::now().year();
    let requested_year = match q.year {
        Some(y) => i32::try_from(y).unwrap_or(fallback_year),
        None => fallback_year,
    };

    let year_i32 = requested_year;
    let rows = fetch_monthly_breakdown_rows(&pool, year_i32).await?;

    let by_month: HashMap<i32, MonthlyBreakdownDbRow> =
        rows.into_iter().map(|row| (row.month, row)).collect();

    let labels = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let mut months = Vec::with_capacity(12);
    for m in 1..=12_i32 {
        let entry = by_month.get(&m);
        months.push(MonthBreakdown {
            month: m as u32,
            label: labels[(m - 1) as usize].to_string(),
            total: entry.map(|e| decimal_to_f64(e.total)).unwrap_or(0.0),
            base: entry.map(|e| decimal_to_f64(e.base)).unwrap_or(0.0),
            tax_paid: entry.map(|e| decimal_to_f64(e.tax_paid)).unwrap_or(0.0),
            company_contributions: entry
                .map(|e| decimal_to_f64(e.company_contributions))
                .unwrap_or(0.0),
        });
    }

    Ok(Json(PayrollBreakdownResponse {
        year: if year_i32 < 0 {
            fallback_year as u32
        } else {
            year_i32 as u32
        },
        months,
    }))
}

async fn fetch_monthly_breakdown_rows(
    pool: &PgPool,
    year: i32,
) -> Result<Vec<MonthlyBreakdownDbRow>, (StatusCode, String)> {
    sqlx::query_as::<_, MonthlyBreakdownDbRow>(
        r#"
        SELECT
            EXTRACT(MONTH FROM pp.pay_date)::INT AS month,
            COALESCE(SUM(pr.regular_pay + pr.tax_deduction + pr.company_match), 0) AS total,
            COALESCE(SUM(pr.regular_pay), 0) AS base,
            COALESCE(SUM(pr.tax_deduction), 0) AS tax_paid,
            COALESCE(SUM(pr.company_match), 0) AS company_contributions
        FROM payroll_periods pp
        INNER JOIN payroll_runs pr ON pr.payroll_period_id = pp.id
        WHERE EXTRACT(YEAR FROM pp.pay_date) = $1
        GROUP BY EXTRACT(MONTH FROM pp.pay_date)
        ORDER BY month
        "#,
    )
    .bind(year)
    .fetch_all(pool)
    .await
    .map_err(internal_error)
}

async fn resolve_reference_month(
    pool: &PgPool,
    today: NaiveDate,
) -> Result<NaiveDate, (StatusCode, String)> {
    let (current_month_start, next_month_start) = month_bounds(today);

    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM payroll_runs pr
        INNER JOIN payroll_periods pp ON pp.id = pr.payroll_period_id
        WHERE pp.pay_date >= $1
          AND pp.pay_date < $2
        "#,
    )
    .bind(current_month_start)
    .bind(next_month_start)
    .fetch_one(pool)
    .await
    .map_err(internal_error)?;

    if count > 0 {
        return Ok(today);
    }

    let latest = sqlx::query_scalar::<_, Option<NaiveDate>>(
        r#"
                SELECT MAX(pp.pay_date)
                FROM payroll_periods pp
                INNER JOIN payroll_runs pr ON pr.payroll_period_id = pp.id
                "#,
    )
    .fetch_one(pool)
    .await
    .map_err(internal_error)?;

    Ok(latest.unwrap_or(today))
}

fn month_bounds(date: NaiveDate) -> (NaiveDate, NaiveDate) {
    let start = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date);
    let (next_year, next_month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    let next_start = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap_or(start);
    (start, next_start)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let start = NaiveDate::from_ymd_opt(year, month, 1);
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let next_start = NaiveDate::from_ymd_opt(next_year, next_month, 1);

    match (start, next_start) {
        (Some(s), Some(n)) => (n - s).num_days().max(0) as u32,
        _ => 30,
    }
}

fn business_days(start: NaiveDate, end: NaiveDate) -> u32 {
    if end < start {
        return 0;
    }

    let mut date = start;
    let mut count = 0;
    while date <= end {
        if !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
            count += 1;
        }
        match date.succ_opt() {
            Some(next) => date = next,
            None => break,
        }
    }
    count
}

fn short_date(date: NaiveDate) -> String {
    date.format("%b %-d").to_string()
}

fn long_date(date: NaiveDate) -> String {
    date.format("%b %-d, %Y").to_string()
}

fn format_money_label(value: Decimal) -> String {
    format!("${:.2}", decimal_to_f64(value.round_dp(2)))
}

fn title_case_status(status: &str) -> String {
    status
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => {
                    format!(
                        "{}{}",
                        first.to_ascii_uppercase(),
                        chars.as_str().to_ascii_lowercase()
                    )
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn estimated_net_pay(
    employment_type: &str,
    base_salary: Option<Decimal>,
    hourly_rate: Option<Decimal>,
) -> Decimal {
    let periods_per_year = Decimal::from(BIWEEKLY_PERIODS_PER_YEAR);

    let (gross_biweekly, annualized_income) = match employment_type {
        "salaried" => {
            let annual = base_salary.unwrap_or(Decimal::ZERO);
            (annual / periods_per_year, annual)
        }
        "hourly" => {
            let hourly = hourly_rate.unwrap_or(Decimal::ZERO);
            let gross = hourly * Decimal::from(HOURLY_FALLBACK_HOURS_PER_PERIOD);
            (gross, gross * periods_per_year)
        }
        "contractor" => {
            let biweekly = base_salary.unwrap_or(Decimal::ZERO);
            (biweekly, biweekly * periods_per_year)
        }
        _ => (Decimal::ZERO, Decimal::ZERO),
    };

    let tax = biweekly_tax(annualized_income);
    (gross_biweekly - tax).max(Decimal::ZERO).round_dp(2)
}

fn estimated_gross_pay(
    employment_type: &str,
    base_salary: Option<Decimal>,
    hourly_rate: Option<Decimal>,
) -> Decimal {
    let periods_per_year = Decimal::from(BIWEEKLY_PERIODS_PER_YEAR);
    match employment_type {
        "salaried" => base_salary.unwrap_or(Decimal::ZERO) / periods_per_year,
        "hourly" => {
            hourly_rate.unwrap_or(Decimal::ZERO) * Decimal::from(HOURLY_FALLBACK_HOURS_PER_PERIOD)
        }
        "contractor" => base_salary.unwrap_or(Decimal::ZERO),
        _ => Decimal::ZERO,
    }
    .round_dp(2)
}

fn biweekly_tax(annualized_income: Decimal) -> Decimal {
    let allowance = Decimal::from(TAX_FREE_ALLOWANCE_ANNUAL);
    let taxable_income = if annualized_income > allowance {
        annualized_income - allowance
    } else {
        Decimal::ZERO
    };
    let rate = Decimal::new(TAX_RATE_PERCENT, 2); // 0.15
    ((taxable_income * rate) / Decimal::from(BIWEEKLY_PERIODS_PER_YEAR)).round_dp(2)
}

fn decimal_to_f64(value: Decimal) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

fn internal_error(err: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {err}"),
    )
}
