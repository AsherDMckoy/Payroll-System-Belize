// API handlers returning JSON (hardcoded data for now).

use axum::extract::Query;
use axum::Json;
use serde::Deserialize;

use crate::models::api::{
    DashboardOverviewResponse, EmployeeRow, EmployeesResponse, MonthBreakdown,
    PayPeriodInfo, PayrollBreakdownResponse, RecentActivityRow,
};

/// GET /api/dashboard/overview
pub async fn dashboard_overview() -> Json<DashboardOverviewResponse> {
    Json(DashboardOverviewResponse {
        payroll_total: "350000".to_string(),
        payroll_label: "This Month (June 2025)".to_string(),
        employee_count: 25,
        employee_delta: "0 added since last month".to_string(),
        deductions_total: "150000".to_string(),
        deductions_label: "This Month (June 2025)".to_string(),
        contributions_total: "50000".to_string(),
        contributions_label: "This Month (June 2025)".to_string(),
        recent_activity: vec![
            RecentActivityRow {
                employee_name: "Hedy Lamarr".to_string(),
                position: "Networks Operator".to_string(),
                salary_paid: "$350.00".to_string(),
                status: "Paid".to_string(),
            },
            RecentActivityRow {
                employee_name: "Ada Lovelace".to_string(),
                position: "Digital Forensics".to_string(),
                salary_paid: "$350.00".to_string(),
                status: "Paid".to_string(),
            },
            RecentActivityRow {
                employee_name: "Hypatia".to_string(),
                position: "Mathematician".to_string(),
                salary_paid: "$350.00".to_string(),
                status: "Paid".to_string(),
            },
            RecentActivityRow {
                employee_name: "Hannah Fry".to_string(),
                position: "Design Expert".to_string(),
                salary_paid: "$350.00".to_string(),
                status: "Paid".to_string(),
            },
        ],
        pay_period: PayPeriodInfo {
            working_days: 10,
            working_hours: 80,
            label: "Months Dates".to_string(),
        },
    })
}

/// GET /api/employees
pub async fn employees_list() -> Json<EmployeesResponse> {
    Json(EmployeesResponse {
        total_employees: 100,
        active_employees: 98,
        inactive_employees: 2,
        employees: vec![
            EmployeeRow {
                name: "Ada Lovelace".to_string(),
                position: "Programmer".to_string(),
                department: "Reverse Engineering".to_string(),
                net_salary: 185_200.0,
                contributions: 6_500.0,
                deductions: 14_800.0,
            },
            EmployeeRow {
                name: "Hypatia".to_string(),
                position: "Quantitative Analyst".to_string(),
                department: "Quantitative Programming".to_string(),
                net_salary: 200_000.0,
                contributions: 10_000.0,
                deductions: 15_000.0,
            },
            EmployeeRow {
                name: "Hedy Lamarr".to_string(),
                position: "Networks Operator".to_string(),
                department: "Infrastructure".to_string(),
                net_salary: 95_000.0,
                contributions: 3_200.0,
                deductions: 8_100.0,
            },
            EmployeeRow {
                name: "Hannah Fry".to_string(),
                position: "Design Expert".to_string(),
                department: "Product".to_string(),
                net_salary: 120_000.0,
                contributions: 4_500.0,
                deductions: 9_200.0,
            },
        ],
    })
}

#[derive(Debug, Deserialize)]
pub struct PayrollBreakdownQuery {
    pub year: Option<u32>,
}

/// GET /api/payroll-breakdown?year=2026
pub async fn payroll_breakdown(
    Query(q): Query<PayrollBreakdownQuery>,
) -> Json<PayrollBreakdownResponse> {
    let year = q.year.unwrap_or(2026);

    // Hardcoded monthly breakdown (same pattern for any year for now)
    let months = vec![
        MonthBreakdown {
            month: 1,
            label: "Jan".to_string(),
            total: 64_110,
            base: 27_555,
            overtime: 26_555,
            incentives: 10_000,
        },
        MonthBreakdown {
            month: 2,
            label: "Feb".to_string(),
            total: 104_110,
            base: 27_555,
            overtime: 26_555,
            incentives: 50_000,
        },
        MonthBreakdown {
            month: 3,
            label: "Mar".to_string(),
            total: 47_500,
            base: 15_200,
            overtime: 12_300,
            incentives: 20_000,
        },
        MonthBreakdown {
            month: 4,
            label: "Apr".to_string(),
            total: 104_110,
            base: 27_555,
            overtime: 26_555,
            incentives: 50_000,
        },
        MonthBreakdown {
            month: 5,
            label: "May".to_string(),
            total: 104_110,
            base: 27_555,
            overtime: 26_555,
            incentives: 50_000,
        },
        MonthBreakdown {
            month: 6,
            label: "Jun".to_string(),
            total: 104_110,
            base: 27_555,
            overtime: 26_555,
            incentives: 50_000,
        },
        MonthBreakdown {
            month: 7,
            label: "Jul".to_string(),
            total: 104_110,
            base: 27_555,
            overtime: 26_555,
            incentives: 50_000,
        },
        MonthBreakdown {
            month: 8,
            label: "Aug".to_string(),
            total: 104_110,
            base: 27_555,
            overtime: 26_555,
            incentives: 50_000,
        },
        MonthBreakdown {
            month: 9,
            label: "Sep".to_string(),
            total: 104_110,
            base: 27_555,
            overtime: 26_555,
            incentives: 50_000,
        },
        MonthBreakdown {
            month: 10,
            label: "Oct".to_string(),
            total: 104_110,
            base: 27_555,
            overtime: 26_555,
            incentives: 50_000,
        },
        MonthBreakdown {
            month: 11,
            label: "Nov".to_string(),
            total: 104_110,
            base: 27_555,
            overtime: 26_555,
            incentives: 50_000,
        },
        MonthBreakdown {
            month: 12,
            label: "Dec".to_string(),
            total: 104_110,
            base: 27_555,
            overtime: 26_555,
            incentives: 50_000,
        },
    ];

    Json(PayrollBreakdownResponse { year, months })
}
