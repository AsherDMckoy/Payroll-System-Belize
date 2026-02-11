use axum::{
    extract::State,
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
    Json,
};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};

use crate::models::api::{
    GenerateReportRequest, GenerateReportResponse, ReportEmployeeOption, ReportFilterOptionsResponse,
};

type ApiResult<T> = Result<T, (StatusCode, String)>;

#[derive(FromRow)]
struct PayrollSummaryRow {
    period_id: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
    pay_date: NaiveDate,
    status: String,
    employee_count: Option<i64>,
    total_gross_pay: Option<Decimal>,
    total_deductions: Option<Decimal>,
    total_net_pay: Option<Decimal>,
}

#[derive(FromRow)]
struct EmployeePayrollRow {
    employee_number: String,
    first_name: String,
    last_name: String,
    employment_type: String,
    payment_status: String,
    gross_pay: Decimal,
    tax_deduction: Decimal,
    employee_savings: Decimal,
    total_deductions: Decimal,
    net_pay: Decimal,
}

#[derive(FromRow)]
struct DeductionSummaryRow {
    period_id: i32,
    start_date: NaiveDate,
    end_date: NaiveDate,
    tax_deduction_total: Decimal,
    employee_savings_total: Decimal,
    company_match_total: Decimal,
    total_deductions: Decimal,
}

#[derive(FromRow)]
struct ReportOptionEmployeeRow {
    id: i32,
    first_name: String,
    last_name: String,
    status: String,
}

#[derive(FromRow)]
struct EmployeeListingRow {
    employee_number: String,
    first_name: String,
    last_name: String,
    department: String,
    position: String,
    employment_type: String,
    status: String,
    hire_date: NaiveDate,
    base_salary: Option<Decimal>,
    hourly_rate: Option<Decimal>,
}

#[derive(FromRow)]
struct EmployeeProfileRow {
    employee_number: String,
    first_name: String,
    last_name: String,
    email: String,
    phone: Option<String>,
    department: String,
    position: String,
    employment_type: String,
    status: String,
    hire_date: NaiveDate,
    base_salary: Option<Decimal>,
    hourly_rate: Option<Decimal>,
}

#[derive(FromRow)]
struct TaxRegisterRow {
    employee_number: String,
    first_name: String,
    last_name: String,
    pay_date: NaiveDate,
    gross_pay: Decimal,
    tax_deduction: Decimal,
}

/// GET /api/reports/options
pub async fn report_filter_options(
    State(pool): State<PgPool>,
) -> ApiResult<Json<ReportFilterOptionsResponse>> {
    let employees = sqlx::query_as::<_, ReportOptionEmployeeRow>(
        r#"
        SELECT id, first_name, last_name, status
        FROM employees
        ORDER BY last_name, first_name
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    let departments = sqlx::query_scalar::<_, String>(
        r#"
        SELECT name
        FROM departments
        ORDER BY name
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    let statuses = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT status
        FROM employees
        ORDER BY status
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(ReportFilterOptionsResponse {
        employees: employees
            .into_iter()
            .map(|e| ReportEmployeeOption {
                id: e.id,
                name: format!("{} {}", e.first_name, e.last_name),
                status: e.status,
            })
            .collect(),
        departments,
        statuses,
    }))
}

/// POST /api/reports/generate
pub async fn generate_report(
    State(pool): State<PgPool>,
    Json(req): Json<GenerateReportRequest>,
) -> ApiResult<Response> {
    let format = req
        .format
        .as_deref()
        .unwrap_or("json")
        .trim()
        .to_ascii_lowercase();

    if format != "json" && format != "csv" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Unsupported format. Use 'json' or 'csv'.".to_string(),
        ));
    }

    let report = match req.report_type.as_str() {
        "payroll_period_summary" => payroll_period_summary_report(&pool, &req).await?,
        "employee_payroll" => employee_payroll_report(&pool, &req).await?,
        "employee_listing" => employee_listing_report(&pool, &req).await?,
        "employee_profile" => employee_profile_report(&pool, &req).await?,
        "deductions_summary" => deductions_summary_report(&pool, &req).await?,
        "tax_register" => tax_register_report(&pool, &req).await?,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Unsupported report_type. Use payroll_period_summary, employee_payroll, employee_listing, employee_profile, deductions_summary, or tax_register.".to_string(),
            ))
        }
    };

    if format == "csv" {
        let csv = to_csv(&report.headers, &report.rows);
        let filename = format!(
            "{}_{}.csv",
            report.file_stem,
            Utc::now().format("%Y%m%d_%H%M%S")
        );
        let mut response = csv.into_response();
        response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("text/csv; charset=utf-8"),
        );
        response.headers_mut().insert(
            CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")).map_err(
                |_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Invalid filename".to_string(),
                    )
                },
            )?,
        );
        return Ok(response);
    }

    Ok(Json(GenerateReportResponse {
        report_type: req.report_type,
        format,
        generated_at: Utc::now().to_rfc3339(),
        row_count: report.rows.len(),
        headers: report.headers,
        rows: report.rows,
    })
    .into_response())
}

struct ReportTable {
    file_stem: String,
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

async fn payroll_period_summary_report(
    pool: &PgPool,
    req: &GenerateReportRequest,
) -> ApiResult<ReportTable> {
    let rows = sqlx::query_as::<_, PayrollSummaryRow>(
        r#"
        SELECT
            period_id,
            start_date,
            end_date,
            pay_date,
            status,
            employee_count,
            total_gross_pay,
            total_deductions,
            total_net_pay
        FROM v_payroll_period_summary
        WHERE ($1::INT IS NULL OR period_id = $1)
          AND ($2::DATE IS NULL OR start_date >= $2)
          AND ($3::DATE IS NULL OR end_date <= $3)
        ORDER BY start_date DESC
        "#,
    )
    .bind(req.payroll_period_id)
    .bind(req.start_date)
    .bind(req.end_date)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let headers = vec![
        "period_id".to_string(),
        "start_date".to_string(),
        "end_date".to_string(),
        "pay_date".to_string(),
        "status".to_string(),
        "employee_count".to_string(),
        "total_gross_pay".to_string(),
        "total_deductions".to_string(),
        "total_net_pay".to_string(),
    ];

    let rows = rows
        .into_iter()
        .map(|r| {
            vec![
                r.period_id.to_string(),
                r.start_date.to_string(),
                r.end_date.to_string(),
                r.pay_date.to_string(),
                r.status,
                r.employee_count.unwrap_or(0).to_string(),
                money_opt(r.total_gross_pay),
                money_opt(r.total_deductions),
                money_opt(r.total_net_pay),
            ]
        })
        .collect();

    Ok(ReportTable {
        file_stem: "payroll_period_summary".to_string(),
        headers,
        rows,
    })
}

async fn employee_payroll_report(
    pool: &PgPool,
    req: &GenerateReportRequest,
) -> ApiResult<ReportTable> {
    let period_id = if let Some(id) = req.payroll_period_id {
        id
    } else {
        sqlx::query_scalar::<_, i32>(
            r#"
            SELECT id
            FROM payroll_periods
            ORDER BY end_date DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                "No payroll periods found".to_string(),
            )
        })?
    };

    let rows = sqlx::query_as::<_, EmployeePayrollRow>(
        r#"
        SELECT
            e.employee_number,
            e.first_name,
            e.last_name,
            e.employment_type,
            pr.payment_status,
            pr.gross_pay,
            pr.tax_deduction,
            pr.employee_savings,
            pr.total_deductions,
            pr.net_pay
        FROM payroll_runs pr
        INNER JOIN employees e ON e.id = pr.employee_id
        WHERE pr.payroll_period_id = $1
        ORDER BY e.last_name, e.first_name
        "#,
    )
    .bind(period_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let headers = vec![
        "employee_number".to_string(),
        "employee_name".to_string(),
        "employment_type".to_string(),
        "payment_status".to_string(),
        "gross_pay".to_string(),
        "tax_deduction".to_string(),
        "employee_savings".to_string(),
        "total_deductions".to_string(),
        "net_pay".to_string(),
    ];

    let rows = rows
        .into_iter()
        .map(|r| {
            vec![
                r.employee_number,
                format!("{} {}", r.first_name, r.last_name),
                r.employment_type,
                r.payment_status,
                money(r.gross_pay),
                money(r.tax_deduction),
                money(r.employee_savings),
                money(r.total_deductions),
                money(r.net_pay),
            ]
        })
        .collect();

    Ok(ReportTable {
        file_stem: format!("employee_payroll_period_{period_id}"),
        headers,
        rows,
    })
}

async fn deductions_summary_report(
    pool: &PgPool,
    req: &GenerateReportRequest,
) -> ApiResult<ReportTable> {
    let rows = sqlx::query_as::<_, DeductionSummaryRow>(
        r#"
        SELECT
            pp.id AS period_id,
            pp.start_date,
            pp.end_date,
            COALESCE(SUM(pr.tax_deduction), 0) AS tax_deduction_total,
            COALESCE(SUM(pr.employee_savings), 0) AS employee_savings_total,
            COALESCE(SUM(pr.company_match), 0) AS company_match_total,
            COALESCE(SUM(pr.total_deductions), 0) AS total_deductions
        FROM payroll_periods pp
        LEFT JOIN payroll_runs pr ON pr.payroll_period_id = pp.id
        WHERE ($1::INT IS NULL OR pp.id = $1)
          AND ($2::DATE IS NULL OR pp.start_date >= $2)
          AND ($3::DATE IS NULL OR pp.end_date <= $3)
        GROUP BY pp.id, pp.start_date, pp.end_date
        ORDER BY pp.start_date DESC
        "#,
    )
    .bind(req.payroll_period_id)
    .bind(req.start_date)
    .bind(req.end_date)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let headers = vec![
        "period_id".to_string(),
        "start_date".to_string(),
        "end_date".to_string(),
        "tax_deduction_total".to_string(),
        "employee_savings_total".to_string(),
        "company_match_total".to_string(),
        "total_deductions".to_string(),
    ];

    let rows = rows
        .into_iter()
        .map(|r| {
            vec![
                r.period_id.to_string(),
                r.start_date.to_string(),
                r.end_date.to_string(),
                money(r.tax_deduction_total),
                money(r.employee_savings_total),
                money(r.company_match_total),
                money(r.total_deductions),
            ]
        })
        .collect();

    Ok(ReportTable {
        file_stem: "deductions_summary".to_string(),
        headers,
        rows,
    })
}

async fn employee_listing_report(
    pool: &PgPool,
    req: &GenerateReportRequest,
) -> ApiResult<ReportTable> {
    let status = normalized_filter(req.status.as_deref());
    let department = normalized_filter(req.department.as_deref());

    let rows = sqlx::query_as::<_, EmployeeListingRow>(
        r#"
        SELECT
            e.employee_number,
            e.first_name,
            e.last_name,
            COALESCE(d.name, 'Unassigned') AS department,
            COALESCE(e.position, 'Unassigned') AS position,
            e.employment_type,
            e.status,
            e.hire_date,
            c.base_salary,
            c.hourly_rate
        FROM employees e
        LEFT JOIN departments d ON d.id = e.department_id
        LEFT JOIN compensation c
            ON c.employee_id = e.id
            AND c.is_current = true
        WHERE ($1::TEXT IS NULL OR e.status = $1)
          AND ($2::TEXT IS NULL OR d.name = $2)
          AND ($3::INT IS NULL OR e.id = $3)
        ORDER BY e.last_name, e.first_name
        "#,
    )
    .bind(status)
    .bind(department)
    .bind(req.employee_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let headers = vec![
        "employee_number".to_string(),
        "employee_name".to_string(),
        "department".to_string(),
        "position".to_string(),
        "employment_type".to_string(),
        "status".to_string(),
        "hire_date".to_string(),
        "base_salary".to_string(),
        "hourly_rate".to_string(),
    ];

    let rows = rows
        .into_iter()
        .map(|r| {
            vec![
                r.employee_number,
                format!("{} {}", r.first_name, r.last_name),
                r.department,
                r.position,
                r.employment_type,
                r.status,
                r.hire_date.to_string(),
                money_opt(r.base_salary),
                money_opt(r.hourly_rate),
            ]
        })
        .collect();

    Ok(ReportTable {
        file_stem: "employee_listing".to_string(),
        headers,
        rows,
    })
}

async fn employee_profile_report(
    pool: &PgPool,
    req: &GenerateReportRequest,
) -> ApiResult<ReportTable> {
    let employee_id = req.employee_id.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            "employee_profile requires employee_id".to_string(),
        )
    })?;

    let row = sqlx::query_as::<_, EmployeeProfileRow>(
        r#"
        SELECT
            e.employee_number,
            e.first_name,
            e.last_name,
            e.email,
            e.phone,
            COALESCE(d.name, 'Unassigned') AS department,
            COALESCE(e.position, 'Unassigned') AS position,
            e.employment_type,
            e.status,
            e.hire_date,
            c.base_salary,
            c.hourly_rate
        FROM employees e
        LEFT JOIN departments d ON d.id = e.department_id
        LEFT JOIN compensation c
            ON c.employee_id = e.id
            AND c.is_current = true
        WHERE e.id = $1
        "#,
    )
    .bind(employee_id)
    .fetch_optional(pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| (StatusCode::NOT_FOUND, "Employee not found".to_string()))?;

    let headers = vec![
        "employee_number".to_string(),
        "employee_name".to_string(),
        "email".to_string(),
        "phone".to_string(),
        "department".to_string(),
        "position".to_string(),
        "employment_type".to_string(),
        "status".to_string(),
        "hire_date".to_string(),
        "base_salary".to_string(),
        "hourly_rate".to_string(),
    ];

    let rows = vec![vec![
        row.employee_number,
        format!("{} {}", row.first_name, row.last_name),
        row.email,
        row.phone.unwrap_or_else(|| "".to_string()),
        row.department,
        row.position,
        row.employment_type,
        row.status,
        row.hire_date.to_string(),
        money_opt(row.base_salary),
        money_opt(row.hourly_rate),
    ]];

    Ok(ReportTable {
        file_stem: format!("employee_profile_{employee_id}"),
        headers,
        rows,
    })
}

async fn tax_register_report(
    pool: &PgPool,
    req: &GenerateReportRequest,
) -> ApiResult<ReportTable> {
    let rows = sqlx::query_as::<_, TaxRegisterRow>(
        r#"
        SELECT
            e.employee_number,
            e.first_name,
            e.last_name,
            pp.pay_date,
            pr.gross_pay,
            pr.tax_deduction
        FROM payroll_runs pr
        INNER JOIN employees e ON e.id = pr.employee_id
        INNER JOIN payroll_periods pp ON pp.id = pr.payroll_period_id
        WHERE ($1::INT IS NULL OR pp.id = $1)
          AND ($2::DATE IS NULL OR pp.start_date >= $2)
          AND ($3::DATE IS NULL OR pp.end_date <= $3)
          AND ($4::INT IS NULL OR e.id = $4)
        ORDER BY pp.pay_date DESC, e.last_name, e.first_name
        "#,
    )
    .bind(req.payroll_period_id)
    .bind(req.start_date)
    .bind(req.end_date)
    .bind(req.employee_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let headers = vec![
        "employee_number".to_string(),
        "employee_name".to_string(),
        "pay_date".to_string(),
        "gross_pay".to_string(),
        "tax_deduction".to_string(),
    ];

    let rows = rows
        .into_iter()
        .map(|r| {
            vec![
                r.employee_number,
                format!("{} {}", r.first_name, r.last_name),
                r.pay_date.to_string(),
                money(r.gross_pay),
                money(r.tax_deduction),
            ]
        })
        .collect();

    Ok(ReportTable {
        file_stem: "tax_register".to_string(),
        headers,
        rows,
    })
}

fn money(value: Decimal) -> String {
    value.round_dp(2).to_string()
}

fn money_opt(value: Option<Decimal>) -> String {
    value.map(money).unwrap_or_else(|| "0.00".to_string())
}

fn normalized_filter(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty() && *v != "all")
        .map(ToOwned::to_owned)
}

fn to_csv(headers: &[String], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    out.push_str(
        &headers
            .iter()
            .map(|h| csv_escape(h))
            .collect::<Vec<_>>()
            .join(","),
    );
    out.push('\n');

    for row in rows {
        out.push_str(
            &row.iter()
                .map(|cell| csv_escape(cell))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }

    out
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn internal_error(err: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {err}"),
    )
}
