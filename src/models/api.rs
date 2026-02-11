// API response types for JSON endpoints (hardcoded data for now).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// ---- Dashboard overview ----

#[derive(Serialize)]
pub struct DashboardOverviewResponse {
    pub payroll_total: String,
    pub payroll_label: String,
    pub employee_count: u32,
    pub employee_delta: String,
    pub deductions_total: String,
    pub deductions_label: String,
    pub contributions_total: String,
    pub contributions_label: String,
    pub recent_activity: Vec<RecentActivityRow>,
    pub pay_period: PayPeriodInfo,
}

#[derive(Serialize)]
pub struct RecentActivityRow {
    pub employee_name: String,
    pub position: String,
    pub salary_paid: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct PayPeriodInfo {
    pub working_days: u32,
    pub working_hours: u32,
    pub label: String,
    pub start_date: String,
    pub end_date: String,
    pub payday: PaydayInfo,
}

#[derive(Serialize)]
pub struct PaydayInfo {
    pub date: String,
    pub day_of_month: u32,
    pub total_days_in_period: u32,
    pub base_salary: String,
    pub tax_paid: String,
    pub company_contributions: String,
    pub total: String,
}

// ---- Employees ----

#[derive(Serialize)]
pub struct EmployeesResponse {
    pub total_employees: u32,
    pub active_employees: u32,
    pub inactive_employees: u32,
    pub employees: Vec<EmployeeRow>,
}

#[derive(Serialize)]
pub struct EmployeeRow {
    pub name: String,
    pub position: String,
    pub department: String,
    pub gross_salary: f64,
    pub net_salary: f64,
    pub contributions: f64,
    pub tax_paid: f64,
}

// ---- Payroll breakdown (chart) ----

#[derive(Serialize)]
pub struct PayrollBreakdownResponse {
    pub year: u32,
    pub months: Vec<MonthBreakdown>,
}

#[derive(Serialize)]
pub struct MonthBreakdown {
    pub month: u32,
    pub label: String,
    pub total: f64,
    pub base: f64,
    pub tax_paid: f64,
    pub company_contributions: f64,
}

// ---- Payroll generation ----

#[derive(Serialize)]
pub struct PayrollPeriodsResponse {
    pub periods: Vec<PayrollPeriodItem>,
}

#[derive(Serialize)]
pub struct PayrollPeriodItem {
    pub id: i32,
    pub start_date: String,
    pub end_date: String,
    pub pay_date: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct GeneratePayrollRequest {
    pub payroll_period_id: i32,
    #[serde(default)]
    pub force_recalculate: bool,
}

#[derive(Serialize)]
pub struct GeneratePayrollResponse {
    pub payroll_period_id: i32,
    pub pay_date: String,
    pub employees_processed: usize,
    pub total_gross_pay: String,
    pub total_deductions: String,
    pub total_net_pay: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct PayrollPeriodDetailsResponse {
    pub period_id: i32,
    pub start_date: String,
    pub end_date: String,
    pub pay_date: String,
    pub status: String,
    pub total_employees: u32,
    pub gross_pay: f64,
    pub deductions: f64,
    pub net_pay: f64,
    pub employees: Vec<PayrollPeriodEmployeeRow>,
}

#[derive(Serialize)]
pub struct PayrollPeriodEmployeeRow {
    pub employee_name: String,
    pub position: String,
    pub department: String,
    pub net_salary: f64,
    pub payment_status: String,
}

// ---- Report generation ----

#[derive(Debug, Deserialize)]
pub struct GenerateReportRequest {
    pub report_type: String,
    pub format: Option<String>,
    pub payroll_period_id: Option<i32>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub employee_id: Option<i32>,
    pub department: Option<String>,
    pub status: Option<String>,
}

#[derive(Serialize)]
pub struct GenerateReportResponse {
    pub report_type: String,
    pub format: String,
    pub generated_at: String,
    pub row_count: usize,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Serialize)]
pub struct ReportFilterOptionsResponse {
    pub employees: Vec<ReportEmployeeOption>,
    pub departments: Vec<String>,
    pub statuses: Vec<String>,
}

#[derive(Serialize)]
pub struct ReportEmployeeOption {
    pub id: i32,
    pub name: String,
    pub status: String,
}
