// Database models for Payroll System
// Using sqlx with PostgreSQL

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ======================
// DEPARTMENT
// ======================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Department {
    pub id: i32,
    pub name: String,
    pub code: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateDepartment {
    pub name: String,
    pub code: Option<String>,
}

// ======================
// EMPLOYEE
// ======================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Employee {
    pub id: i32,
    pub employee_number: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub department_id: Option<i32>,
    pub position: Option<String>,
    pub employment_type: String, // 'salaried', 'hourly', 'contractor'
    pub status: String,          // 'active', 'inactive', 'terminated'
    pub hire_date: NaiveDate,
    pub termination_date: Option<NaiveDate>,
    pub bank_account_number: Option<String>,
    pub bank_routing_number: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployee {
    pub employee_number: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub department_id: Option<i32>,
    pub position: Option<String>,
    pub employment_type: String,
    pub hire_date: NaiveDate,
    pub bank_account_number: Option<String>,
    pub bank_routing_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmployee {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department_id: Option<i32>,
    pub position: Option<String>,
    pub status: Option<String>,
    pub termination_date: Option<NaiveDate>,
    pub bank_account_number: Option<String>,
    pub bank_routing_number: Option<String>,
}

// ======================
// COMPENSATION
// ======================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Compensation {
    pub id: i32,
    pub employee_id: i32,
    pub base_salary: Option<Decimal>,
    pub hourly_rate: Option<Decimal>,
    pub effective_date: NaiveDate,
    pub is_current: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateCompensation {
    pub employee_id: i32,
    pub base_salary: Option<Decimal>,
    pub hourly_rate: Option<Decimal>,
    pub effective_date: NaiveDate,
}

// ======================
// EMPLOYEE SAVINGS
// ======================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmployeeSavings {
    pub id: i32,
    pub employee_id: i32,
    pub contribution_amount: Decimal,
    pub contribution_date: NaiveDate,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployeeSavings {
    pub employee_id: i32,
    pub contribution_amount: Decimal,
    pub contribution_date: NaiveDate,
}

// ======================
// PAYROLL PERIOD
// ======================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PayrollPeriod {
    pub id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub pay_date: NaiveDate,
    pub status: String, // 'draft', 'processing', 'approved', 'paid'
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayrollPeriod {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub pay_date: NaiveDate,
}

// ======================
// TIMESHEET
// ======================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Timesheet {
    pub id: i32,
    pub employee_id: i32,
    pub payroll_period_id: i32,
    pub regular_hours: Decimal,
    pub overtime_hours: Decimal,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateTimesheet {
    pub employee_id: i32,
    pub payroll_period_id: i32,
    pub regular_hours: Decimal,
    pub overtime_hours: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTimesheet {
    pub regular_hours: Option<Decimal>,
    pub overtime_hours: Option<Decimal>,
}

// ======================
// PAYROLL ADJUSTMENT
// ======================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PayrollAdjustment {
    pub id: i32,
    pub employee_id: i32,
    pub payroll_period_id: i32,
    pub adjustment_type: String,
    pub amount: Decimal,
    pub description: Option<String>,
    pub created_by: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayrollAdjustment {
    pub employee_id: i32,
    pub payroll_period_id: i32,
    pub adjustment_type: String,
    pub amount: Decimal,
    pub description: Option<String>,
    pub created_by: Option<String>,
}

// ======================
// PAYROLL RUN
// ======================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PayrollRun {
    pub id: i32,
    pub employee_id: i32,
    pub payroll_period_id: i32,
    pub gross_pay: Decimal,
    pub regular_pay: Decimal,
    pub overtime_pay: Decimal,
    pub bonuses: Decimal,
    pub tax_deduction: Decimal,
    pub employee_savings: Decimal,
    pub company_match: Decimal,
    pub total_deductions: Decimal,
    pub net_pay: Decimal,
    pub payment_status: String, // 'pending', 'paid', 'failed'
    pub payment_date: Option<NaiveDate>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreatePayrollRun {
    pub employee_id: i32,
    pub payroll_period_id: i32,
    pub gross_pay: Decimal,
    pub regular_pay: Decimal,
    pub overtime_pay: Decimal,
    pub bonuses: Decimal,
    pub tax_deduction: Decimal,
    pub employee_savings: Decimal,
    pub company_match: Decimal,
    pub total_deductions: Decimal,
    pub net_pay: Decimal,
}

// ======================
// AUDIT LOG
// ======================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: i32,
    pub table_name: String,
    pub record_id: Option<i32>,
    pub action: String,
    pub user_name: Option<String>,
    pub changed_data: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
}

// ======================
// VIEW MODELS
// ======================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmployeeWithCompensation {
    pub id: i32,
    pub employee_number: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub employment_type: String,
    pub status: String,
    pub position: Option<String>,
    pub department_name: Option<String>,
    pub base_salary: Option<Decimal>,
    pub hourly_rate: Option<Decimal>,
    pub effective_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmployeeYtdSavings {
    pub employee_id: i32,
    pub employee_number: String,
    pub first_name: String,
    pub last_name: String,
    pub ytd_contributions: Decimal,
    pub ytd_company_match: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PayrollPeriodSummary {
    pub period_id: i32,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub pay_date: NaiveDate,
    pub status: String,
    pub employee_count: Option<i64>,
    pub total_gross_pay: Option<Decimal>,
    pub total_deductions: Option<Decimal>,
    pub total_net_pay: Option<Decimal>,
    pub total_tax: Option<Decimal>,
    pub total_savings: Option<Decimal>,
}

// ======================
// HELPER STRUCTS
// ======================

#[derive(Debug, Serialize)]
pub struct EmployeeListItem {
    pub id: i32,
    pub employee_number: String,
    pub name: String,
    pub position: Option<String>,
    pub department: Option<String>,
    pub employment_type: String,
    pub status: String,
    pub net_salary: Option<Decimal>,
    pub contributions: Option<Decimal>,
    pub deductions: Option<Decimal>,
}

impl From<EmployeeWithCompensation> for EmployeeListItem {
    fn from(emp: EmployeeWithCompensation) -> Self {
        let name = format!("{} {}", emp.first_name, emp.last_name);

        // Calculate biweekly amounts
        let (net_salary, contributions, deductions) = match emp.employment_type.as_str() {
            "salaried" => {
                if let Some(annual) = emp.base_salary {
                    let biweekly = annual / Decimal::from(26);
                    // Simplified calculation - you'll calculate this properly in payroll
                    (Some(biweekly), Some(Decimal::ZERO), Some(Decimal::ZERO))
                } else {
                    (None, None, None)
                }
            }
            "contractor" => {
                // base_salary is biweekly for contractors
                (emp.base_salary, Some(Decimal::ZERO), Some(Decimal::ZERO))
            }
            _ => (None, None, None),
        };

        EmployeeListItem {
            id: emp.id,
            employee_number: emp.employee_number,
            name,
            position: emp.position,
            department: emp.department_name,
            employment_type: emp.employment_type,
            status: emp.status,
            net_salary,
            contributions,
            deductions,
        }
    }
}
