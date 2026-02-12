use axum::{
    extract::State,
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderValue, StatusCode,
    },
    response::{IntoResponse, Response},
    Json,
};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};

use crate::models::api::{
    GenerateReportRequest, GenerateReportResponse, ReportEmployeeOption,
    ReportFilterOptionsResponse,
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
    bank_account_number: Option<String>,
    employment_type: String,
    pay_date: NaiveDate,
    payment_status: String,
    regular_pay: Decimal,
    overtime_pay: Decimal,
    bonuses: Decimal,
    gross_pay: Decimal,
    tax_deduction: Decimal,
    employee_savings: Decimal,
    company_match: Decimal,
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

#[derive(FromRow)]
struct PayrollApprovalHistoryRow {
    audit_id: i32,
    created_at: NaiveDateTime,
    payroll_period_id: Option<i32>,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
    pay_date: Option<NaiveDate>,
    event_name: Option<String>,
    period_status: Option<String>,
    user_name: Option<String>,
    action: String,
    employees_impacted: Option<String>,
    total_net_pay: Option<String>,
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
    let report_type = req.report_type.clone();
    let format = req
        .format
        .as_deref()
        .unwrap_or("json")
        .trim()
        .to_ascii_lowercase();

    if format != "json" && format != "csv" && format != "pdf" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Unsupported format. Use 'json', 'csv', or 'pdf'.".to_string(),
        ));
    }

    let report = match report_type.as_str() {
        "payroll_period_summary" => payroll_period_summary_report(&pool, &req).await?,
        "employee_payroll" => employee_payroll_report(&pool, &req).await?,
        "employee_listing" => employee_listing_report(&pool, &req).await?,
        "employee_profile" => employee_profile_report(&pool, &req).await?,
        "deductions_summary" => deductions_summary_report(&pool, &req).await?,
        "tax_register" => tax_register_report(&pool, &req).await?,
        "payroll_approval_history" => payroll_approval_history_report(&pool, &req).await?,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Unsupported report_type. Use payroll_period_summary, employee_payroll, employee_listing, employee_profile, deductions_summary, tax_register, or payroll_approval_history.".to_string(),
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

    if format == "pdf" {
        let pdf = to_pdf(&report_type, &report.headers, &report.rows);
        let filename = format!(
            "{}_{}.pdf",
            report.file_stem,
            Utc::now().format("%Y%m%d_%H%M%S")
        );
        let mut response = pdf.into_response();
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/pdf"));
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
        report_type,
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
            SELECT pp.id
            FROM payroll_periods pp
            WHERE pp.status IN ('processing', 'approved', 'paid')
              AND EXISTS (
                  SELECT 1
                  FROM payroll_runs pr
                  WHERE pr.payroll_period_id = pp.id
                    AND pr.payment_status IN ('pending', 'paid')
              )
            ORDER BY pp.end_date DESC
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
            e.bank_account_number,
            e.employment_type,
            pp.pay_date,
            pr.payment_status,
            pr.regular_pay,
            pr.overtime_pay,
            pr.bonuses,
            pr.gross_pay,
            pr.tax_deduction,
            pr.employee_savings,
            pr.company_match,
            pr.total_deductions,
            pr.net_pay
        FROM payroll_runs pr
        INNER JOIN employees e ON e.id = pr.employee_id
        INNER JOIN payroll_periods pp ON pp.id = pr.payroll_period_id
        WHERE pr.payroll_period_id = $1
          AND pr.payment_status IN ('pending', 'paid')
          AND ($2::INT IS NULL OR e.id = $2)
        ORDER BY e.last_name, e.first_name
        "#,
    )
    .bind(period_id)
    .bind(req.employee_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let headers = vec![
        "pay_date".to_string(),
        "employee_number".to_string(),
        "employee_name".to_string(),
        "bank_account_number".to_string(),
        "employment_type".to_string(),
        "payment_status".to_string(),
        "regular_pay".to_string(),
        "overtime_pay".to_string(),
        "bonuses".to_string(),
        "gross_pay".to_string(),
        "tax_deduction".to_string(),
        "employee_savings_contribution".to_string(),
        "company_match_contribution".to_string(),
        "total_deductions".to_string(),
        "net_pay".to_string(),
    ];

    let rows = rows
        .into_iter()
        .map(|r| {
            vec![
                r.pay_date.to_string(),
                r.employee_number,
                format!("{} {}", r.first_name, r.last_name),
                r.bank_account_number.unwrap_or_else(|| "".to_string()),
                r.employment_type,
                r.payment_status,
                money(r.regular_pay),
                money(r.overtime_pay),
                money(r.bonuses),
                money(r.gross_pay),
                money(r.tax_deduction),
                money(r.employee_savings),
                money(r.company_match),
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
    let employee_id = if let Some(id) = req.employee_id {
        id
    } else {
        sqlx::query_scalar::<_, i32>(
            r#"
            SELECT id
            FROM employees
            WHERE status = 'active'
            ORDER BY last_name, first_name
            LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "No employees found".to_string()))?
    };

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

async fn tax_register_report(pool: &PgPool, req: &GenerateReportRequest) -> ApiResult<ReportTable> {
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

async fn payroll_approval_history_report(
    pool: &PgPool,
    req: &GenerateReportRequest,
) -> ApiResult<ReportTable> {
    let rows = sqlx::query_as::<_, PayrollApprovalHistoryRow>(
        r#"
        SELECT
            al.id AS audit_id,
            al.created_at,
            al.record_id AS payroll_period_id,
            pp.start_date,
            pp.end_date,
            pp.pay_date,
            al.changed_data ->> 'event' AS event_name,
            al.changed_data ->> 'status' AS period_status,
            al.user_name,
            al.action,
            COALESCE(
                al.changed_data ->> 'employees_processed',
                al.changed_data ->> 'employees_ready',
                al.changed_data ->> 'employees_paid'
            ) AS employees_impacted,
            al.changed_data ->> 'total_net_pay' AS total_net_pay
        FROM audit_logs al
        LEFT JOIN payroll_periods pp ON pp.id = al.record_id
        WHERE al.table_name = 'payroll_periods'
          AND (al.changed_data ->> 'event') IN ('payroll_generated', 'payroll_approved', 'payroll_paid')
          AND ($1::INT IS NULL OR al.record_id = $1)
          AND ($2::DATE IS NULL OR pp.start_date >= $2)
          AND ($3::DATE IS NULL OR pp.end_date <= $3)
        ORDER BY al.created_at DESC, al.id DESC
        "#,
    )
    .bind(req.payroll_period_id)
    .bind(req.start_date)
    .bind(req.end_date)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let headers = vec![
        "audit_id".to_string(),
        "created_at".to_string(),
        "event".to_string(),
        "action".to_string(),
        "user_name".to_string(),
        "payroll_period_id".to_string(),
        "start_date".to_string(),
        "end_date".to_string(),
        "pay_date".to_string(),
        "status".to_string(),
        "employees_impacted".to_string(),
        "total_net_pay".to_string(),
    ];

    let rows = rows
        .into_iter()
        .map(|r| {
            vec![
                r.audit_id.to_string(),
                r.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                humanize_event_name(r.event_name.as_deref()),
                r.action,
                r.user_name.unwrap_or_else(|| "system".to_string()),
                r.payroll_period_id
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                r.start_date.map(|d| d.to_string()).unwrap_or_default(),
                r.end_date.map(|d| d.to_string()).unwrap_or_default(),
                r.pay_date.map(|d| d.to_string()).unwrap_or_default(),
                r.period_status.unwrap_or_default(),
                r.employees_impacted.unwrap_or_default(),
                r.total_net_pay.unwrap_or_default(),
            ]
        })
        .collect();

    Ok(ReportTable {
        file_stem: "payroll_approval_history".to_string(),
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

fn humanize_event_name(value: Option<&str>) -> String {
    match value {
        Some(event) if !event.trim().is_empty() => event
            .split('_')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        _ => "Payroll Event".to_string(),
    }
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

fn to_pdf(report_type: &str, headers: &[String], rows: &[Vec<String>]) -> Vec<u8> {
    let mut lines = Vec::new();
    lines.push(format!("Payroll Report: {report_type}"));
    lines.push(format!(
        "Generated At: {}",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));
    lines.push(String::new());

    if headers.is_empty() {
        lines.push("No columns available.".to_string());
    } else {
        lines.push(headers.join(" | "));
        lines.push("-".repeat(120));
        if rows.is_empty() {
            lines.push("(No rows returned for this report)".to_string());
        } else {
            for row in rows {
                lines.push(row.join(" | "));
            }
        }
    }

    let mut wrapped_lines = Vec::new();
    for line in lines {
        wrapped_lines.extend(wrap_line(&line, 110));
    }

    let lines_per_page = 48usize;
    let mut pages = Vec::new();
    for chunk in wrapped_lines.chunks(lines_per_page) {
        pages.push(chunk.to_vec());
    }
    if pages.is_empty() {
        pages.push(vec!["Payroll report is empty.".to_string()]);
    }

    build_pdf(&pages)
}

fn wrap_line(line: &str, max_chars: usize) -> Vec<String> {
    if line.chars().count() <= max_chars {
        return vec![line.to_string()];
    }

    let mut out = Vec::new();
    let mut current = String::new();
    for token in line.split_whitespace() {
        let token_len = token.chars().count();
        if current.is_empty() {
            if token_len > max_chars {
                out.push(token.chars().take(max_chars).collect());
            } else {
                current.push_str(token);
            }
            continue;
        }

        let next_len = current.chars().count() + 1 + token_len;
        if next_len > max_chars {
            out.push(current);
            if token_len > max_chars {
                out.push(token.chars().take(max_chars).collect());
                current = String::new();
            } else {
                current = token.to_string();
            }
        } else {
            current.push(' ');
            current.push_str(token);
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

fn build_pdf(pages: &[Vec<String>]) -> Vec<u8> {
    let page_count = pages.len();
    let font_obj_id = 3 + (page_count * 2);
    let max_obj_id = font_obj_id;
    let mut objects: Vec<Option<Vec<u8>>> = vec![None; max_obj_id + 1];

    let kids = (0..page_count)
        .map(|index| format!("{} 0 R", 3 + (index * 2)))
        .collect::<Vec<_>>()
        .join(" ");

    objects[1] = Some(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    objects[2] =
        Some(format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids, page_count).into_bytes());
    objects[font_obj_id] = Some(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec());

    for (index, lines) in pages.iter().enumerate() {
        let page_obj_id = 3 + (index * 2);
        let content_obj_id = page_obj_id + 1;
        let content_stream = build_pdf_stream(lines);
        let content_obj = format!(
            "<< /Length {} >>\nstream\n{}endstream",
            content_stream.as_bytes().len(),
            content_stream
        );
        objects[content_obj_id] = Some(content_obj.into_bytes());
        objects[page_obj_id] = Some(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 {} 0 R >> >> /Contents {} 0 R >>",
                font_obj_id, content_obj_id
            )
            .into_bytes(),
        );
    }

    let mut pdf = Vec::new();
    pdf.extend_from_slice(b"%PDF-1.4\n");

    let mut offsets = vec![0usize; max_obj_id + 1];
    for (id, obj) in objects.iter().enumerate().skip(1) {
        if let Some(payload) = obj {
            offsets[id] = pdf.len();
            pdf.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
            pdf.extend_from_slice(payload);
            pdf.extend_from_slice(b"\nendobj\n");
        }
    }

    let xref_pos = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n", max_obj_id + 1).as_bytes());
    pdf.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            max_obj_id + 1,
            xref_pos
        )
        .as_bytes(),
    );

    pdf
}

fn build_pdf_stream(lines: &[String]) -> String {
    let mut out = String::new();
    out.push_str("BT\n/F1 10 Tf\n50 770 Td\n");
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            out.push_str("0 -14 Td\n");
        }
        out.push('(');
        out.push_str(&escape_pdf_text(line));
        out.push_str(") Tj\n");
    }
    out.push_str("ET\n");
    out
}

fn escape_pdf_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '(' => escaped.push_str("\\("),
            ')' => escaped.push_str("\\)"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn internal_error(err: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {err}"),
    )
}
