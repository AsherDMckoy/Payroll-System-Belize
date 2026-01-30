// API response types for JSON endpoints (hardcoded data for now).

use serde::Serialize;

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
    pub net_salary: f64,
    pub contributions: f64,
    pub deductions: f64,
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
    pub total: u32,
    pub base: u32,
    pub overtime: u32,
    pub incentives: u32,
}
