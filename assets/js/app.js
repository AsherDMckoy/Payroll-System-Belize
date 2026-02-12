const header = document.getElementById("header-bar");
const headerHeight = header.offsetHeight;

document.documentElement.style.setProperty(
  "--header-height",
  headerHeight + "px",
);

const logoutBtn = document.getElementById("logout-btn");
if (logoutBtn) {
  logoutBtn.addEventListener("click", async () => {
    logoutBtn.disabled = true;
    try {
      await fetch("/api/auth/logout", { method: "POST" });
    } catch (err) {
      console.error("logout:", err);
    } finally {
      window.location.href = "/login";
    }
  });
}

// =============================
// API-driven data (JSON from Rust backend)
// =============================

function formatCompactCurrency(numStr) {
  const n = parseFloat(numStr);
  if (!Number.isFinite(n)) return "—";
  if (n >= 1000000) return "$" + (n / 1000000).toFixed(1) + "M";
  if (n >= 1000) return "$" + (n / 1000).toFixed(0) + "K";
  return "$" + Math.round(n);
}

function formatCurrency(num) {
  const n = parseFloat(num);
  if (!Number.isFinite(n)) return "—";
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: 0,
  }).format(n);
}

function formatBreakdownNumber(num) {
  const n = Number(num);
  if (!Number.isFinite(n)) return "0.00";
  return new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(n);
}

const CURRENT_YEAR = new Date().getFullYear();
let activePayrollBreakdownYear = CURRENT_YEAR;

function normalizeBreakdownYear(value) {
  const parsed = Number.parseInt(String(value ?? ""), 10);
  if (!Number.isFinite(parsed)) return CURRENT_YEAR;
  return Math.min(parsed, CURRENT_YEAR);
}

function updateBreakdownYearControls(component, year) {
  const yearsWrap = component.querySelector(".breakdown_years");
  if (!yearsWrap) return;

  const yearValue = normalizeBreakdownYear(year);
  const yearBtn = yearsWrap.querySelector("[data-breakdown-year]");
  const nextBtn = yearsWrap.querySelector("[data-year-nav=\"next\"]");

  if (yearBtn) {
    yearBtn.textContent = String(yearValue);
    yearBtn.dataset.year = String(yearValue);
  }

  if (nextBtn) {
    nextBtn.disabled = yearValue >= CURRENT_YEAR;
  }
}

function applyDashboardOverviewData(data, grid, tbody) {
  if (!grid || !tbody) return;

  grid.querySelector(".payroll_total [data-dashboard-value]").textContent =
    formatCompactCurrency(data.payroll_total);
  grid.querySelector(".payroll_total [data-dashboard-sublabel]").textContent =
    data.payroll_label || "—";

  grid.querySelector(".employee_total [data-dashboard-value]").textContent =
    String(data.employee_count ?? "—");
  grid.querySelector(".employee_total [data-dashboard-sublabel]").textContent =
    data.employee_delta || "—";

  grid.querySelector(".deduction_total [data-dashboard-value]").textContent =
    formatCompactCurrency(data.deductions_total);
  grid.querySelector(".deduction_total [data-dashboard-sublabel]").textContent =
    data.deductions_label || "—";

  grid.querySelector(".contribution_total [data-dashboard-value]").textContent =
    formatCurrency(data.contributions_total);
  grid.querySelector(".contribution_total [data-dashboard-sublabel]").textContent =
    data.contributions_label || "—";

  const pp = data.pay_period || {};
  const daysEl = grid.querySelector("[data-pay-period-days-value]");
  const hoursEl = grid.querySelector("[data-pay-period-hours-value]");
  if (daysEl) daysEl.textContent = pp.working_days ?? "—";
  if (hoursEl) hoursEl.textContent = pp.working_hours ?? "—";

  const startEl = grid.querySelector("[data-pay-period-start]");
  const endEl = grid.querySelector("[data-pay-period-end]");
  if (startEl) startEl.textContent = pp.start_date ?? "—";
  if (endEl) endEl.textContent = pp.end_date ?? "—";

  if (pp.payday) {
    const bubble = grid.querySelector("[data-payday-bubble]");
    const paydayDateEl = grid.querySelector("[data-payday-date]");

    if (bubble && paydayDateEl) {
      paydayDateEl.textContent = pp.payday.date || "—";

      const dayOfMonth = pp.payday.day_of_month;
      const totalDays = pp.payday.total_days_in_period;
      if (dayOfMonth && totalDays) {
        const percentage = (dayOfMonth / totalDays) * 100;
        bubble.style.left = `${percentage}%`;
        bubble.hidden = false;
      } else {
        bubble.hidden = true;
      }
    }

    const breakdownSection = grid.querySelector("[data-payday-breakdown]");
    if (breakdownSection) {
      const baseEl = breakdownSection.querySelector("[data-payday-base]");
      const taxEl = breakdownSection.querySelector("[data-payday-tax]");
      const contributionsEl = breakdownSection.querySelector("[data-payday-contributions]");
      const totalEl = breakdownSection.querySelector("[data-payday-total]");

      if (baseEl) baseEl.textContent = pp.payday.base_salary || "—";
      if (taxEl) taxEl.textContent = pp.payday.tax_paid || "—";
      if (contributionsEl) {
        contributionsEl.textContent = pp.payday.company_contributions || "—";
      }
      if (totalEl) totalEl.textContent = pp.payday.total || "—";

      breakdownSection.hidden = false;
    }
  }

  tbody.innerHTML = "";
  (data.recent_activity || []).forEach((row) => {
    const tr = document.createElement("tr");
    tr.innerHTML = `
      <td>${escapeHtml(row.employee_name)}</td>
      <td>${escapeHtml(row.position)}</td>
      <td>${escapeHtml(row.salary_paid)}</td>
      <td>${escapeHtml(row.status)}</td>
    `;
    tbody.appendChild(tr);
  });
  if (tbody.children.length === 0) {
    const tr = document.createElement("tr");
    tr.innerHTML = "<td colspan=\"4\">No recent activity</td>";
    tbody.appendChild(tr);
  }
}

function applyPayrollPageSummaryCards(data) {
  const payrollValue = document.querySelector("[data-payroll-page-payroll-value]");
  const payrollLabel = document.querySelector("[data-payroll-page-payroll-label]");
  const contributionsValue = document.querySelector("[data-payroll-page-contributions-value]");
  const contributionsLabel = document.querySelector("[data-payroll-page-contributions-label]");
  const deductionsValue = document.querySelector("[data-payroll-page-deductions-value]");
  const deductionsLabel = document.querySelector("[data-payroll-page-deductions-label]");
  const employeesValue = document.querySelector("[data-payroll-page-employees-value]");
  const employeesLabel = document.querySelector("[data-payroll-page-employees-label]");

  if (payrollValue) payrollValue.textContent = formatCompactCurrency(data.payroll_total);
  if (payrollLabel) payrollLabel.textContent = data.payroll_label || "—";
  if (contributionsValue) contributionsValue.textContent = formatCurrency(data.contributions_total);
  if (contributionsLabel) contributionsLabel.textContent = data.contributions_label || "—";
  if (deductionsValue) deductionsValue.textContent = formatCompactCurrency(data.deductions_total);
  if (deductionsLabel) deductionsLabel.textContent = data.deductions_label || "—";
  if (employeesValue) employeesValue.textContent = String(data.employee_count ?? "—");
  if (employeesLabel) employeesLabel.textContent = data.employee_delta || "—";
}

async function loadDashboardOverview() {
  const grid = document.getElementById("dashboard-overview-grid");
  const tbody = document.getElementById("recent-activity-tbody");

  try {
    const res = await fetch("/api/dashboard/overview");
    if (!res.ok) throw new Error(res.statusText);
    const data = await res.json();
    applyDashboardOverviewData(data, grid, tbody);
    return data;
  } catch (err) {
    console.error("loadDashboardOverview:", err);
    if (tbody) {
      tbody.innerHTML = "<tr><td colspan=\"4\">Failed to load</td></tr>";
    }
    return null;
  }
}

function escapeHtml(s) {
  if (s == null) return "";
  const div = document.createElement("div");
  div.textContent = s;
  return div.innerHTML;
}

async function loadPayrollBreakdown(year) {
  const components = document.querySelectorAll('.payroll_breakdown[data-chart="payroll-breakdown"]');
  if (!components.length) return;

  try {
    const targetYear = normalizeBreakdownYear(
      year ?? activePayrollBreakdownYear,
    );
    const query = `?year=${encodeURIComponent(targetYear)}`;
    const res = await fetch(`/api/payroll-breakdown${query}`);
    if (!res.ok) throw new Error(res.statusText);
    const data = await res.json();
    const resolvedYear = normalizeBreakdownYear(data.year ?? targetYear);
    activePayrollBreakdownYear = resolvedYear;

    const months = data.months || [];
    const monthLabels = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

    components.forEach((component) => {
      const chart = component.querySelector(".breakdowns_chart");
      if (!chart) return;

      const breakdowns = Array.from(chart.querySelectorAll(".breakdown"));

      for (let i = 0; i < 12; i++) {
        const m = months[i];
        const b = breakdowns[i];
        if (!b) continue;

        const total = m ? m.total : 0;
        const base = m ? m.base : 0;
        const taxPaid = m ? m.tax_paid : 0;
        const companyContributions = m ? m.company_contributions : 0;

        const totalEl = b.querySelector(".breakdown_total");
        const bar = b.querySelector(".breakdown_bar");
        const monthEl = b.querySelector(".breakdown_month");
        if (totalEl) totalEl.textContent = String(total);
        if (bar) {
          bar.dataset.base = String(base);
          bar.dataset.tax = String(taxPaid);
          bar.dataset.contributions = String(companyContributions);
        }
        if (monthEl) monthEl.textContent = m ? m.label : monthLabels[i] || "";

        const tooltipList = b.querySelectorAll(".tooltip_list_item .tooltip_breakdown_value");
        if (tooltipList.length >= 3) {
          tooltipList[0].textContent = formatBreakdownNumber(base);
          tooltipList[1].textContent = formatBreakdownNumber(taxPaid);
          tooltipList[2].textContent = formatBreakdownNumber(companyContributions);
        }
      }

      updateBreakdownYearControls(component, resolvedYear);
    });

    initPayrollBreakdownCharts();
    return data;
  } catch (err) {
    console.error("loadPayrollBreakdown:", err);
    return null;
  }
}

async function loadEmployees() {
  const summary = document.getElementById("employees-summary");
  const tbody = document.getElementById("employees-tbody");
  if (!tbody) return;

  try {
    const res = await fetch("/api/employees");
    if (!res.ok) throw new Error(res.statusText);
    const data = await res.json();

    if (summary) {
      const totalEl = summary.querySelector("[data-employees-total]");
      const activeEl = summary.querySelector("[data-employees-active]");
      const inactiveEl = summary.querySelector("[data-employees-inactive]");
      if (totalEl) totalEl.textContent = String(data.total_employees ?? "—");
      if (activeEl) activeEl.textContent = String(data.active_employees ?? "—");
      if (inactiveEl) inactiveEl.textContent = String(data.inactive_employees ?? "—");
    }

    tbody.innerHTML = "";
    (data.employees || []).forEach((emp) => {
      const tr = document.createElement("tr");
      tr.dataset.grossSalary = String(emp.gross_salary ?? "");
      tr.innerHTML = `
        <td>${escapeHtml(emp.name)}</td>
        <td>${escapeHtml(emp.position)}</td>
        <td>${escapeHtml(emp.department)}</td>
        <td class="is-numeric">${formatCurrency(emp.net_salary)}</td>
        <td class="is-numeric">${formatCurrency(emp.contributions)}</td>
        <td class="is-numeric">${formatCurrency(emp.tax_paid)}</td>
      `;
      tbody.appendChild(tr);
    });
    if (tbody.children.length === 0) {
      const tr = document.createElement("tr");
      tr.innerHTML = "<td colspan=\"6\">No employees</td>";
      tbody.appendChild(tr);
    }

    wireEmployeesTable();
  } catch (err) {
    console.error("loadEmployees:", err);
    tbody.innerHTML = "<tr><td colspan=\"6\">Failed to load</td></tr>";
  }
}

function initOverviewPage() {
  loadDashboardOverview();
  loadPayrollBreakdown(activePayrollBreakdownYear);
}

function initEmployeesPage() {
  loadEmployees();
}

// Reports page init
function initReportsPage() {
  initReports();
}

let payrollActionsWired = false;

function formatPeriodLabel(period) {
  const status = period.status ? ` (${period.status})` : "";
  const lock = period.is_locked ? " [Locked]" : "";
  return `${period.start_date} to ${period.end_date}${status}${lock}`;
}

async function loadPayrollPeriods() {
  const select = document.getElementById("pay-period");
  if (!select) return [];

  try {
    const previousSelection = select.value;
    const res = await fetch("/api/payroll/periods");
    if (!res.ok) throw new Error(res.statusText);
    const data = await res.json();
    const periods = Array.isArray(data.periods) ? data.periods : [];

    select.innerHTML = "";
    periods.forEach((period) => {
      const option = document.createElement("option");
      option.value = String(period.id);
      option.textContent = formatPeriodLabel(period);
      select.appendChild(option);
    });

    if (!periods.length) {
      const option = document.createElement("option");
      option.value = "";
      option.textContent = "No payroll periods available";
      select.appendChild(option);
      return [];
    }

    const hasPrevious = periods.some((p) => String(p.id) === previousSelection);
    if (hasPrevious) {
      select.value = previousSelection;
    }
    return periods;
  } catch (err) {
    console.error("loadPayrollPeriods:", err);
    return [];
  }
}

async function generatePayrollForSelectedPeriod() {
  const select = document.getElementById("pay-period");
  if (!select || !select.value) {
    alert("Select a payroll period before generating payroll.");
    return;
  }

  const payrollPeriodId = Number(select.value);
  if (!Number.isFinite(payrollPeriodId)) {
    alert("Selected payroll period is invalid.");
    return;
  }

  const res = await fetch("/api/payroll/generate", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      payroll_period_id: payrollPeriodId,
      force_recalculate: false,
      requested_by: "internal_ui",
    }),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || "Failed to generate payroll.");
  }

  return res.json();
}

async function approvePayrollForSelectedPeriod() {
  const select = document.getElementById("pay-period");
  if (!select || !select.value) {
    alert("Select a payroll period before approval.");
    return;
  }

  const payrollPeriodId = Number(select.value);
  if (!Number.isFinite(payrollPeriodId)) {
    alert("Selected payroll period is invalid.");
    return;
  }

  const res = await fetch("/api/payroll/approve", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      payroll_period_id: payrollPeriodId,
      requested_by: "internal_ui",
    }),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || "Failed to approve payroll.");
  }

  return res.json();
}

async function executePayrollForSelectedPeriod() {
  const select = document.getElementById("pay-period");
  if (!select || !select.value) {
    alert("Select a payroll period before executing payroll.");
    return;
  }

  const payrollPeriodId = Number(select.value);
  if (!Number.isFinite(payrollPeriodId)) {
    alert("Selected payroll period is invalid.");
    return;
  }

  const res = await fetch("/api/payroll/execute", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      payroll_period_id: payrollPeriodId,
      requested_by: "internal_ui",
    }),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || "Failed to execute payroll.");
  }

  return res.json();
}

function updatePayrollActionState(canGenerate, canApprove, canExecute) {
  const newPayrollBtn = document.querySelector(".new_payroll_btn");
  const approveBtn = document.querySelector(".approve_payroll_btn");
  const markPaidBtn = document.querySelector(".mark_paid_btn");

  if (newPayrollBtn) newPayrollBtn.disabled = !canGenerate;
  if (approveBtn) approveBtn.disabled = !canApprove;
  if (markPaidBtn) markPaidBtn.disabled = !canExecute;
}

function formatAuditEventName(name) {
  const raw = String(name || "").trim();
  if (!raw) return "Payroll Event";
  return raw
    .replaceAll("_", " ")
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

function renderPayrollAuditEvents(events) {
  const list = document.getElementById("payroll-audit-list");
  if (!list) return;

  const safeEvents = Array.isArray(events) ? events : [];
  if (!safeEvents.length) {
    list.innerHTML = "<div class=\"payroll_audit_empty\">No audit events for this period yet.</div>";
    return;
  }

  list.innerHTML = "";
  safeEvents.forEach((event) => {
    const when = escapeHtml(event.created_at || "Unknown time");
    const eventName = escapeHtml(formatAuditEventName(event.event));
    const statusText = event?.details?.status ? ` (${escapeHtml(event.details.status)})` : "";
    const byUser = escapeHtml(event.user_name || "system");
    const action = escapeHtml(event.action || "UPDATE");

    const row = document.createElement("div");
    row.className = "payroll_audit_item";
    row.innerHTML = `
      <div class="payroll_audit_when">${when}</div>
      <div class="payroll_audit_event">${eventName}${statusText}</div>
      <div class="payroll_audit_meta">${action} by ${byUser}</div>
    `;
    list.appendChild(row);
  });
}

async function loadPayrollAuditTrail(periodId) {
  const list = document.getElementById("payroll-audit-list");
  if (!list) return;

  const query = new URLSearchParams();
  if (Number.isFinite(periodId) && periodId > 0) {
    query.set("period_id", String(periodId));
  }
  query.set("limit", "25");

  try {
    const res = await fetch(`/api/payroll/audit?${query.toString()}`);
    if (!res.ok) throw new Error(res.statusText);
    const data = await res.json();
    renderPayrollAuditEvents(data.events);
  } catch (err) {
    console.error("loadPayrollAuditTrail:", err);
    list.innerHTML = "<div class=\"payroll_audit_empty\">Unable to load audit trail right now.</div>";
  }
}

function wirePayrollActions() {
  if (payrollActionsWired) return;
  payrollActionsWired = true;

  const newPayrollBtn = document.querySelector(".new_payroll_btn");
  const approveBtn = document.querySelector(".approve_payroll_btn");
  const markPaidBtn = document.querySelector(".mark_paid_btn");
  const reportsBtn = document.querySelector(".generate_report_btn");
  const payPeriodSelect = document.getElementById("pay-period");
  if (!newPayrollBtn || !payPeriodSelect) return;

  payPeriodSelect.addEventListener("change", () => {
    updatePayrollPeriodDetails();
  });

  newPayrollBtn.addEventListener("click", async () => {
    const originalText = newPayrollBtn.textContent;
    newPayrollBtn.disabled = true;
    newPayrollBtn.textContent = "Generating...";

    try {
      const result = await generatePayrollForSelectedPeriod();
      if (result) {
        alert(
          `Payroll generated.\nEmployees: ${result.employees_processed}\nNet Pay: ${result.total_net_pay}`,
        );
      }
      const overviewData = await loadDashboardOverview();
      if (overviewData) applyPayrollPageSummaryCards(overviewData);
      await loadPayrollPeriods();
      loadEmployees();
    } catch (err) {
      console.error("generatePayrollForSelectedPeriod:", err);
      alert(`Payroll generation failed: ${err.message || err}`);
    } finally {
      newPayrollBtn.textContent = originalText;
      await updatePayrollPeriodDetails();
    }
  });

  if (approveBtn) {
    approveBtn.addEventListener("click", async () => {
      const originalText = approveBtn.textContent;
      approveBtn.disabled = true;
      approveBtn.textContent = "Approving...";

      try {
        const result = await approvePayrollForSelectedPeriod();
        if (result) {
          alert(
            `Payroll approved and locked.\nEmployees Ready: ${result.employees_ready}\nTotal Net Pay: ${result.total_net_pay}`,
          );
        }
        await loadPayrollPeriods();
      } catch (err) {
        console.error("approvePayrollForSelectedPeriod:", err);
        alert(`Payroll approval failed: ${err.message || err}`);
      } finally {
        approveBtn.textContent = originalText;
        await updatePayrollPeriodDetails();
      }
    });
  }

  if (markPaidBtn) {
    markPaidBtn.addEventListener("click", async () => {
      const originalText = markPaidBtn.textContent;
      markPaidBtn.disabled = true;
      markPaidBtn.textContent = "Paying...";

      try {
        const result = await executePayrollForSelectedPeriod();
        if (result) {
          alert(
            `Payroll executed.\nEmployees Paid: ${result.employees_paid}\nTotal Net Pay: ${result.total_net_pay}`,
          );
        }
        const overviewData = await loadDashboardOverview();
        if (overviewData) applyPayrollPageSummaryCards(overviewData);
        await loadPayrollPeriods();
        loadEmployees();
      } catch (err) {
        console.error("executePayrollForSelectedPeriod:", err);
        alert(`Payroll execution failed: ${err.message || err}`);
      } finally {
        markPaidBtn.textContent = originalText;
        await updatePayrollPeriodDetails();
      }
    });
  }

  if (reportsBtn) {
    reportsBtn.addEventListener("click", () => {
      window.location.href = "/reports";
    });
  }
}

// Payroll process page init – populate chart so it works when landing directly on /payroll
async function initPayrollPage() {
  loadPayrollBreakdown(activePayrollBreakdownYear);
  wirePayrollActions();
  const overviewData = await loadDashboardOverview();
  if (overviewData) applyPayrollPageSummaryCards(overviewData);
  await loadPayrollPeriods();
  await updatePayrollPeriodDetails();
}

function fillPayrollEmployeesTable(employees) {
  const tbody = document.getElementById("payroll-employees-tbody");
  if (!tbody) return;

  tbody.innerHTML = "";
  (employees || []).forEach((row) => {
    const tr = document.createElement("tr");
    tr.innerHTML = `
      <td>${escapeHtml(row.employee_name)}</td>
      <td>${escapeHtml(row.position)}</td>
      <td>${escapeHtml(row.department)}</td>
      <td class="is-numeric">${formatCurrency(row.net_salary)}</td>
      <td>${escapeHtml(row.payment_status)}</td>
      <td>payslip</td>
    `;
    tbody.appendChild(tr);
  });

  if (tbody.children.length === 0) {
    const tr = document.createElement("tr");
    tr.innerHTML = "<td colspan=\"6\">No payroll data for this period</td>";
    tbody.appendChild(tr);
  }
}

async function updatePayrollPeriodDetails() {
  const select = document.getElementById("pay-period");
  if (!select) return null;

  const periodId = Number(select.value);
  const usePeriodId = Number.isFinite(periodId) && periodId > 0;
  const query = usePeriodId ? `?period_id=${encodeURIComponent(periodId)}` : "";

  try {
    const res = await fetch(`/api/payroll/period-details${query}`);
    if (!res.ok) throw new Error(res.statusText);
    const data = await res.json();

    updatePayrollPeriodDetailsFromData({
      payrollPeriod: `${data.start_date} to ${data.end_date}`,
      totalEmployees: data.total_employees,
      payDay: data.pay_date,
      status: data.status,
      isLocked: Boolean(data.is_locked),
      canGenerate: Boolean(data.can_generate),
      canApprove: Boolean(data.can_approve),
      canExecute: Boolean(data.can_execute),
      grossPay: data.gross_pay,
      deductions: data.deductions,
      netPay: data.net_pay,
    });
    fillPayrollEmployeesTable(data.employees || []);

    if (usePeriodId && data.period_id && String(data.period_id) !== select.value) {
      select.value = String(data.period_id);
    }

    const resolvedPeriodId = Number.isFinite(periodId) && periodId > 0
      ? periodId
      : Number(data.period_id);
    await loadPayrollAuditTrail(resolvedPeriodId);

    return data;
  } catch (err) {
    console.error("updatePayrollPeriodDetails:", err);
    updatePayrollActionState(false, false, false);
    fillPayrollEmployeesTable([]);
    renderPayrollAuditEvents([]);
    return null;
  }
}

function updatePayrollPeriodDetailsFromData(data) {
  const {
    payrollPeriod,
    totalEmployees,
    payDay,
    status,
    isLocked,
    canGenerate,
    canApprove,
    canExecute,
    grossPay,
    deductions,
    netPay,
  } = data;

  // Update info grid
  const periodEl = document.querySelector('[data-payroll-period]');
  const employeesEl = document.querySelector('[data-total-employees]');
  const payDayEl = document.querySelector('[data-pay-day]');
  const statusEl = document.querySelector('[data-payroll-status]');
  const lockedEl = document.querySelector('[data-payroll-locked]');

  if (periodEl) periodEl.textContent = payrollPeriod || "—";
  if (employeesEl) employeesEl.textContent = String(totalEmployees ?? "—");
  if (payDayEl) payDayEl.textContent = payDay || "—";
  if (lockedEl) lockedEl.textContent = isLocked ? "Yes" : "No";

  if (statusEl && status) {
    const indicator = document.createElement("span");
    indicator.className = "status_indicator";
    const normalized = String(status).toLowerCase();
    if (normalized === "paid") indicator.classList.add("status_paid");
    else if (normalized === "pending" || normalized === "processing") indicator.classList.add("status_pending");
    else indicator.classList.add("status_scheduled");

    statusEl.innerHTML = "";
    statusEl.appendChild(indicator);
    statusEl.appendChild(document.createTextNode(` ${status}`));
  }

  updatePayrollActionState(Boolean(canGenerate), Boolean(canApprove), Boolean(canExecute));

  // Update financial values
  const grossPayEl = document.querySelector('[data-gross-pay]');
  const deductionsEl = document.querySelector('[data-deductions]');
  const totalNetPayEl = document.querySelector('[data-total-net-pay]');

  const gross = Number(grossPay);
  const ded = Number(deductions);
  const net = Number.isFinite(Number(netPay)) ? Number(netPay) : gross - ded;

  if (grossPayEl) grossPayEl.textContent = formatMoney(gross);
  if (deductionsEl) deductionsEl.textContent = formatMoney(ded);

  // Calculate and update circle graph
  const netPercentage = gross > 0 ? (net / gross) * 100 : 0;

  if (totalNetPayEl) totalNetPayEl.textContent = formatMoney(net);

  const circleProgress = document.querySelector('[data-circle-progress]');
  const circlePercentage = document.querySelector('[data-circle-percentage]');

  if (circleProgress && circlePercentage) {
    const radius = 40;
    const circumference = 2 * Math.PI * radius;
    const progressLength = (netPercentage / 100) * circumference;

    requestAnimationFrame(() => {
      circleProgress.style.strokeDasharray = `${progressLength} ${circumference}`;
      circlePercentage.textContent = `${Math.round(Math.max(0, netPercentage))}%`;
    });
  }
}

// Expose for router
window.initOverviewPage = initOverviewPage;
window.initEmployeesPage = initEmployeesPage;
window.initReportsPage = initReportsPage;
window.initPayrollPage = initPayrollPage;
window.loadPayrollBreakdown = loadPayrollBreakdown;
window.updatePayrollPeriodDetails = updatePayrollPeriodDetails;
window.updatePayrollPeriodDetailsFromData = updatePayrollPeriodDetailsFromData;

// =============================
// Reports UI
// =============================

function initReports() {
  const root = document.getElementById("reports-root");
  if (!root) return;
  if (root.dataset.initialized === "1") return;
  root.dataset.initialized = "1";

  const tabs = Array.from(root.querySelectorAll(".tab"));
  const cardsWrap = root.querySelector("#reports-cards");
  const listTitle = root.querySelector("#reports-list-title");
  const listSubtitle = root.querySelector("#reports-list-subtitle");
  const search = root.querySelector("#reports-search");
  const dynamicFilters = root.querySelector("#reports-dynamic-filters");
  const rangeRow = root.querySelector("#reports-range-row");
  const selectedHint = root.querySelector("#reports-selected-hint");
  const contextTitle = root.querySelector(".report-context-title");
  const contextText = root.querySelector(".report-context-text");
  const range = root.querySelector("#reports-range");
  const customRange = root.querySelector("#reports-custom-range");
  const resetBtn = root.querySelector("#reports-reset-btn");
  const previewBtn = root.querySelector("#reports-preview-btn");
  const generateBtn = root.querySelector("#reports-generate-btn");
  const helpBtn = root.querySelector("#reports-help-btn");

  const modal = root.querySelector("#reports-modal");
  const modalSummary = root.querySelector("#reports-modal-summary");
  const modalGenerate = root.querySelector("#reports-modal-generate");
  const helpModal = root.querySelector("#reports-help");

  const categories = {
    employee: {
      title: "Employee",
      subtitle: "Employee listing, profile, and payroll detail exports.",
    },
    payroll: {
      title: "Payroll",
      subtitle: "Pay period, approval workflow, and deductions reports for operations and finance.",
    },
    compliance: {
      title: "Compliance",
      subtitle: "Tax-facing reports for reconciliations and filing support.",
    },
  };

  // Report definitions mapped directly to backend report_type values.
  const reports = [
    {
      id: "employee_listing",
      reportType: "employee_listing",
      category: "employee",
      title: "Employee Listing",
      desc: "Directory of employees with department, status, and compensation.",
      tags: ["HR", "Directory"],
      usesDateRange: false,
      filters: [
        { type: "select", name: "status", label: "Employee status", options: ["All"] },
        { type: "select", name: "department", label: "Department", options: ["All"] },
      ],
    },
    {
      id: "employee_profile",
      reportType: "employee_profile",
      category: "employee",
      title: "Employee Profile",
      desc: "Detailed profile report for one selected employee.",
      tags: ["HR"],
      usesDateRange: false,
      filters: [
        { type: "employee", name: "employee_id", label: "Employee", required: true },
      ],
    },
    {
      id: "employee_payroll",
      reportType: "employee_payroll",
      category: "payroll",
      title: "Employee Payroll Detail",
      desc: "Who is pending/paid in a pay period with full payroll financials.",
      tags: ["Payroll", "Pay Run"],
      usesDateRange: false,
      filters: [
        { type: "payroll_period", name: "payroll_period_id", label: "Pay period" },
        { type: "employee", name: "employee_id", label: "Employee (optional)" },
      ],
    },
    {
      id: "payroll_period_summary",
      reportType: "payroll_period_summary",
      category: "payroll",
      title: "Payroll Period Summary",
      desc: "Gross, deductions, and net totals by payroll period.",
      tags: ["Finance", "Summary"],
      usesDateRange: true,
      filters: [
        { type: "payroll_period", name: "payroll_period_id", label: "Pay period (optional)" },
      ],
    },
    {
      id: "deductions_summary",
      reportType: "deductions_summary",
      category: "payroll",
      title: "Deductions Summary",
      desc: "Tax, employee savings, company match, and total deductions.",
      tags: ["Tax", "Deductions"],
      usesDateRange: true,
      filters: [
        { type: "payroll_period", name: "payroll_period_id", label: "Pay period (optional)" },
      ],
    },
    {
      id: "payroll_approval_history",
      reportType: "payroll_approval_history",
      category: "payroll",
      title: "Payroll Approval History",
      desc: "Audit trail of payroll generation, approval/locking, and paid actions.",
      tags: ["Audit", "Approval"],
      usesDateRange: true,
      filters: [
        { type: "payroll_period", name: "payroll_period_id", label: "Pay period (optional)" },
      ],
    },
    {
      id: "tax_register",
      reportType: "tax_register",
      category: "compliance",
      title: "Tax Register",
      desc: "Employee-level tax deductions for reporting and filing support.",
      tags: ["Compliance", "Tax"],
      usesDateRange: true,
      filters: [
        { type: "employee", name: "employee_id", label: "Employee (optional)" },
        { type: "payroll_period", name: "payroll_period_id", label: "Pay period (optional)" },
      ],
    },
  ];

  let activeCategory = "employee";
  let selectedReportId = null;
  const reportOptions = {
    employees: [],
    departments: [],
    statuses: [],
    periods: [],
  };

  function setModalOpen(which, open) {
    const el = which === "help" ? helpModal : modal;
    if (!el) return;
    el.hidden = !open;
    el.setAttribute("aria-hidden", String(!open));
    document.body.style.overflow = open ? "hidden" : "";
  }

  async function loadReportOptions() {
    try {
      const [filtersRes, periodsRes] = await Promise.all([
        fetch("/api/reports/options"),
        fetch("/api/payroll/periods"),
      ]);
      if (!filtersRes.ok) throw new Error(filtersRes.statusText);
      if (!periodsRes.ok) throw new Error(periodsRes.statusText);

      const filterData = await filtersRes.json();
      const periodData = await periodsRes.json();

      reportOptions.employees = Array.isArray(filterData.employees) ? filterData.employees : [];
      reportOptions.departments = Array.isArray(filterData.departments) ? filterData.departments : [];
      reportOptions.statuses = Array.isArray(filterData.statuses) ? filterData.statuses : [];
      reportOptions.periods = Array.isArray(periodData.periods) ? periodData.periods : [];

      if (selectedReportId) {
        const report = reports.find((r) => r.id === selectedReportId);
        renderDynamicFilters(report || null);
      }
    } catch (err) {
      console.error("loadReportOptions:", err);
    }
  }

  function renderDynamicFilters(report) {
    dynamicFilters.innerHTML = "";
    if (!report) return;

    for (const f of report.filters) {
      const row = document.createElement("div");
      row.className = "form-row";

      const label = document.createElement("label");
      label.className = "label";
      const id = `reports-filter-${f.name}`;
      label.htmlFor = id;
      label.textContent = f.label;
      row.appendChild(label);

      if (f.type === "select") {
        const select = document.createElement("select");
        select.className = "select";
        select.id = id;
        select.name = f.name;

        let options = Array.isArray(f.options) ? [...f.options] : [];
        if (f.name === "department" && reportOptions.departments.length) {
          options = ["All", ...reportOptions.departments];
        }
        if (f.name === "status" && reportOptions.statuses.length) {
          options = ["All", ...reportOptions.statuses];
        }

        for (const opt of options) {
          const normalizedOpt = String(opt);
          const o = document.createElement("option");
          if (f.name === "department" || f.name === "status") {
            o.value = normalizedOpt.toLowerCase() === "all" ? "all" : normalizedOpt;
          } else {
            o.value = normalizedOpt.toLowerCase().replace(/\s+/g, "_");
          }
          o.textContent = normalizedOpt;
          select.appendChild(o);
        }
        row.appendChild(select);
      } else if (f.type === "employee") {
        const select = document.createElement("select");
        select.className = "select";
        select.id = id;
        select.name = f.name;
        if (f.required) select.required = true;

        const placeholder = document.createElement("option");
        placeholder.value = "";
        placeholder.textContent = reportOptions.employees.length
          ? (f.required ? "Select employee..." : "All employees")
          : "No employees available";
        if (f.required) placeholder.disabled = true;
        select.appendChild(placeholder);

        reportOptions.employees.forEach((emp) => {
          const o = document.createElement("option");
          o.value = String(emp.id);
          o.textContent = `${emp.name} (${emp.status})`;
          select.appendChild(o);
        });
        if (f.required && reportOptions.employees.length) {
          select.value = String(reportOptions.employees[0].id);
        }
        row.appendChild(select);
      } else if (f.type === "payroll_period") {
        const select = document.createElement("select");
        select.className = "select";
        select.id = id;
        select.name = f.name;

        const placeholder = document.createElement("option");
        placeholder.value = "";
        placeholder.textContent = "Current pay period";
        select.appendChild(placeholder);

        reportOptions.periods.forEach((period) => {
          const o = document.createElement("option");
          o.value = String(period.id);
          o.textContent = `${period.start_date} to ${period.end_date} (${period.status})`;
          select.appendChild(o);
        });
        row.appendChild(select);
      }

      dynamicFilters.appendChild(row);
    }
  }

  function renderCards() {
    const q = (search.value || "").trim().toLowerCase();
    const filtered = reports
      .filter((r) => r.category === activeCategory)
      .filter((r) => !q || r.title.toLowerCase().includes(q) || r.desc.toLowerCase().includes(q));

    cardsWrap.innerHTML = "";
    if (!filtered.length) {
      cardsWrap.innerHTML = `<div class="report-desc">No reports match your search.</div>`;
      return;
    }

    filtered.forEach((r) => {
      const card = document.createElement("div");
      card.className = "report-card" + (r.id === selectedReportId ? " is-selected" : "");
      card.tabIndex = 0;
      card.setAttribute("role", "button");
      card.setAttribute("aria-label", `Select report ${r.title}`);
      card.dataset.reportId = r.id;
      card.innerHTML = `
        <div class="report-title">${r.title}</div>
        <div class="report-desc">${r.desc}</div>
        <div class="tags">${r.tags.map((t) => `<span class="tag">${t}</span>`).join("")}</div>
      `;
      cardsWrap.appendChild(card);
    });
  }

  function updateReportContext(report) {
    if (!contextTitle || !contextText) return;

    if (!report) {
      contextTitle.textContent = "No report selected";
      contextText.textContent = "Choose a report card to load the correct filters.";
      return;
    }

    contextTitle.textContent = report.title;
    contextText.textContent = report.usesDateRange
      ? "This report supports date-based filtering."
      : "This report does not use date range filters.";
  }

  function setCategory(cat) {
    activeCategory = cat;
    selectedReportId = null;
    generateBtn.disabled = true;
    previewBtn.disabled = true;
    selectedHint.textContent = "Select a report to configure options.";
    renderDynamicFilters(null);
    updateReportContext(null);
    if (rangeRow) rangeRow.hidden = false;
    search.value = "";
    listTitle.textContent = categories[cat].title;
    listSubtitle.textContent = categories[cat].subtitle;

    tabs.forEach((t) => {
      const active = t.dataset.category === cat;
      t.classList.toggle("is-active", active);
      t.setAttribute("aria-selected", String(active));
    });
    renderCards();

    const first = reports.find((r) => r.category === cat);
    if (first) setReport(first.id);
  }

  function setReport(id) {
    selectedReportId = id;
    const report = reports.find((r) => r.id === id);
    selectedHint.textContent = report ? `Selected: ${report.title}` : "Select a report to configure options.";
    generateBtn.disabled = !report;
    previewBtn.disabled = !report;
    if (rangeRow && report) {
      rangeRow.hidden = !report.usesDateRange;
      if (!report.usesDateRange) customRange.hidden = true;
    }
    updateReportContext(report || null);
    renderDynamicFilters(report);
    renderCards();
  }

  // Date range UI
  range.addEventListener("change", () => {
    customRange.hidden = range.value !== "custom";
  });

  // Tabs
  tabs.forEach((t) => t.addEventListener("click", () => setCategory(t.dataset.category)));

  // Search
  search.addEventListener("input", renderCards);

  // Card selection
  cardsWrap.addEventListener("click", (e) => {
    const card = e.target.closest(".report-card");
    if (!card) return;
    setReport(card.dataset.reportId);
  });
  cardsWrap.addEventListener("keydown", (e) => {
    if (e.key !== "Enter" && e.key !== " ") return;
    const card = e.target.closest(".report-card");
    if (!card) return;
    e.preventDefault();
    setReport(card.dataset.reportId);
  });

  // Reset
  resetBtn.addEventListener("click", () => {
    root.querySelector("#reports-filters-form").reset();
    customRange.hidden = true;
    // Keep selected report, just re-render filters to defaults
    if (selectedReportId) setReport(selectedReportId);
  });

  // Preview (UI-only)
  previewBtn.addEventListener("click", () => {
    if (!selectedReportId) return;
    const report = reports.find((r) => r.id === selectedReportId);
    modalSummary.innerHTML = buildSummary(report);
    root.querySelector("#reports-modal-title").textContent = `Preview: ${report.title}`;
    modalGenerate.textContent = "Generate";
    setModalOpen("main", true);
  });

  // Generate
  generateBtn.addEventListener("click", () => {
    if (!selectedReportId) return;
    const report = reports.find((r) => r.id === selectedReportId);
    modalSummary.innerHTML = buildSummary(report);
    root.querySelector("#reports-modal-title").textContent = `Generate: ${report.title}`;
    modalGenerate.textContent = "Generate";
    setModalOpen("main", true);
  });

  modal.addEventListener("click", (e) => {
    if (e.target.closest("[data-modal-close]")) setModalOpen("main", false);
  });
  helpModal.addEventListener("click", (e) => {
    if (e.target.closest("[data-help-close]")) setModalOpen("help", false);
  });

  function extractFilename(contentDisposition, fallback) {
    if (!contentDisposition) return fallback;
    const match = contentDisposition.match(/filename=\"?([^"]+)\"?/i);
    return match ? match[1] : fallback;
  }

  function localDateIso(date) {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    return `${year}-${month}-${day}`;
  }

  function applyPresetDateRange(rangeValue, payload) {
    const today = new Date();
    const start = new Date(today.getFullYear(), today.getMonth(), 1);
    const end = new Date(today.getFullYear(), today.getMonth() + 1, 0);

    if (rangeValue === "last_month") {
      start.setMonth(start.getMonth() - 1);
      end.setMonth(end.getMonth() - 1);
    } else if (rangeValue === "this_year") {
      start.setMonth(0, 1);
      end.setMonth(11, 31);
    } else if (rangeValue === "last_year") {
      start.setFullYear(start.getFullYear() - 1, 0, 1);
      end.setFullYear(end.getFullYear() - 1, 11, 31);
    } else if (rangeValue !== "this_month") {
      return;
    }

    payload.start_date = localDateIso(start);
    payload.end_date = localDateIso(end);
  }

  function downloadBlob(blob, filename) {
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
  }

  modalGenerate.addEventListener("click", async () => {
    if (!selectedReportId) return;

    const report = reports.find((r) => r.id === selectedReportId);
    if (!report) return;

    const reportType = report.reportType;
    if (!reportType) {
      alert("This report is not yet wired to backend generation.");
      return;
    }

    const form = root.querySelector("#reports-filters-form");
    const formData = new FormData(form);
    const preferredFormat = String(formData.get("format") || "csv").toLowerCase();
    const backendFormat = preferredFormat === "xlsx" ? "csv" : preferredFormat;

    if (preferredFormat !== backendFormat) {
      alert(`XLSX export is not available yet. Generating ${backendFormat.toUpperCase()} instead.`);
    }

    const payload = {
      report_type: reportType,
      format: backendFormat,
    };

    const employeeIdRaw = String(formData.get("employee_id") || "").trim();
    if (employeeIdRaw) {
      const employeeId = Number(employeeIdRaw);
      if (Number.isFinite(employeeId)) payload.employee_id = employeeId;
    }
    if (report.reportType === "employee_profile" && payload.employee_id == null) {
      const fallbackEmployee = reportOptions.employees[0];
      if (fallbackEmployee) {
        payload.employee_id = Number(fallbackEmployee.id);
      } else {
        alert("No employee is available to generate an employee profile report.");
        return;
      }
    }

    const department = String(formData.get("department") || "").trim();
    if (department && department !== "all") {
      payload.department = department;
    }

    const status = String(formData.get("status") || "").trim();
    if (status && status !== "all") {
      payload.status = status;
    }

    const payrollPeriodRaw = String(formData.get("payroll_period_id") || "").trim();
    if (payrollPeriodRaw) {
      const periodId = Number(payrollPeriodRaw);
      if (Number.isFinite(periodId)) payload.payroll_period_id = periodId;
    }

    if (report.usesDateRange) {
      const rangeValue = String(formData.get("range") || "").trim().toLowerCase();
      if (rangeValue === "custom") {
        const start = formData.get("start");
        const end = formData.get("end");
        if (start) payload.start_date = start;
        if (end) payload.end_date = end;
      } else {
        applyPresetDateRange(rangeValue, payload);
      }
    }

    modalGenerate.disabled = true;
    const originalText = modalGenerate.textContent;
    modalGenerate.textContent = "Generating...";

    try {
      const res = await fetch("/api/reports/generate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });

      if (!res.ok) {
        const text = await res.text();
        throw new Error(text || "Failed to generate report.");
      }

      if (backendFormat === "csv" || backendFormat === "pdf") {
        const blob = await res.blob();
        const filename = extractFilename(
          res.headers.get("content-disposition"),
          `${reportType}.${backendFormat}`,
        );
        downloadBlob(blob, filename);
      } else {
        const json = await res.json();
        const blob = new Blob([JSON.stringify(json, null, 2)], {
          type: "application/json",
        });
        downloadBlob(blob, `${reportType}.json`);
      }

      setModalOpen("main", false);
    } catch (err) {
      console.error("generateReport:", err);
      alert(`Report generation failed: ${err.message || err}`);
    } finally {
      modalGenerate.disabled = false;
      modalGenerate.textContent = originalText;
    }
  });

  helpBtn.addEventListener("click", () => setModalOpen("help", true));

  function buildSummary(report) {
    const form = root.querySelector("#reports-filters-form");
    const fd = new FormData(form);
    const labelMap = {
      range: "Date range",
      start: "Start date",
      end: "End date",
      format: "Export format",
      employee_id: "Employee",
      department: "Department",
      status: "Status",
      payroll_period_id: "Pay period",
    };

    const periodNameById = new Map(
      reportOptions.periods.map((p) => [String(p.id), `${p.start_date} to ${p.end_date} (${p.status})`]),
    );
    const employeeNameById = new Map(
      reportOptions.employees.map((e) => [String(e.id), `${e.name} (${e.status})`]),
    );

    const entries = [];
    for (const [key, value] of fd.entries()) {
      if (!report.usesDateRange && (key === "range" || key === "start" || key === "end")) {
        continue;
      }
      const textValue = String(value || "").trim();
      if (!textValue) continue;

      let displayValue = textValue;
      if (key === "employee_id") {
        displayValue = employeeNameById.get(textValue) || textValue;
      } else if (key === "payroll_period_id") {
        displayValue = periodNameById.get(textValue) || "Current pay period";
      } else if (key === "format") {
        displayValue = textValue.toUpperCase();
      } else if (textValue === "all") {
        displayValue = "All";
      } else if (key === "range") {
        displayValue = textValue.replace(/_/g, " ");
      }

      entries.push({
        label: labelMap[key] || key,
        value: displayValue,
      });
    }

    if (!entries.length) {
      entries.push({ label: "Filters", value: "Default report settings" });
    }

    const rows = entries
      .map(({ label, value }) => `<div style="display:flex;justify-content:space-between;gap:12px;"><span style="color:rgba(255,255,255,0.72)">${label}</span><span>${value}</span></div>`)
      .join("");

    return `
      <div style="display:grid;gap:10px;">
        <div><strong>Category:</strong> ${categories[report.category].title}</div>
        <div><strong>Report:</strong> ${report.title}</div>
        <div class="divider"></div>
        <div style="display:grid;gap:8px;">${rows}</div>
      </div>
    `;
  }

  // Initialize
  customRange.hidden = range.value !== "custom";
  setCategory(activeCategory);
  loadReportOptions();
}

// =========================
// Employees table -> details card
// =========================

function parseMoney(raw) {
  const cleaned = (raw || "").toString().replace(/[^0-9.-]/g, "");
  const num = Number(cleaned);
  return Number.isFinite(num) ? num : null;
}

function formatMoney(num) {
  if (!Number.isFinite(num)) return "—";
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    maximumFractionDigits: 0,
  }).format(num);
}

function initialsFromName(name) {
  const parts = (name || "").trim().split(/\s+/).filter(Boolean);
  const first = parts[0]?.[0] || "";
  const last = parts.length > 1 ? parts[parts.length - 1][0] : "";
  return (first + last).toUpperCase() || "??";
}

function setMetricByLabel(cardEl, labelText, valueText, valueClass = "") {
  const metrics = cardEl.querySelectorAll(".metric");
  for (const metric of metrics) {
    const label = metric.querySelector(".label");
    const value = metric.querySelector(".value");
    if (!label || !value) continue;

    if (label.textContent.trim().toLowerCase() === labelText.trim().toLowerCase()) {
      value.textContent = valueText;
      value.className = "value" + (valueClass ? ` ${valueClass}` : "");
      return true;
    }
  }
  return false;
}

function wireEmployeesTable() {
  const table = document.querySelector(".employees_table");
  const card = document.querySelector(".employee_card");
  if (!table || !card) return;

  table.addEventListener("click", (e) => {
    const row = e.target.closest("tr");
    if (!row) return;
    if (row.querySelector("th")) return; // header row

    const cells = row.querySelectorAll("td");
    // Expect: Name, Position, Department, Net, Contributions, Deductions
    if (cells.length < 6) return;

    const name = cells[0].textContent.trim();
    const position = cells[1].textContent.trim();
    const department = cells[2].textContent.trim();

    const netNum = parseMoney(cells[3].textContent);
    const contribNum = parseMoney(cells[4].textContent);
    const taxNum = parseMoney(cells[5].textContent);
    const grossNumFromRow = Number(row.dataset.grossSalary);
    const grossNum = Number.isFinite(grossNumFromRow)
      ? grossNumFromRow
      : (Number.isFinite(netNum) && Number.isFinite(taxNum) ? netNum + Math.abs(taxNum) : null);

    // Selected row highlight
    table.querySelectorAll("tr.is-selected").forEach((r) => r.classList.remove("is-selected"));
    row.classList.add("is-selected");

    // Populate top of card
    const avatar = card.querySelector(".employee_avatar span");
    const nameEl = card.querySelector(".employee_name");
    const roleEl = card.querySelector(".employee_role");

    if (avatar) avatar.textContent = initialsFromName(name);
    if (nameEl) nameEl.textContent = name;
    if (roleEl) roleEl.textContent = `${position} • ${department}`;

    // Populate metrics (labels must match your card labels exactly)
    setMetricByLabel(card, "Gross Salary", grossNum !== null ? formatMoney(grossNum) : "—");
    setMetricByLabel(card, "Net Salary", netNum !== null ? formatMoney(netNum) : "—");
    setMetricByLabel(card, "Contributions", contribNum !== null ? formatMoney(contribNum) : "—");
    if (taxNum !== null) {
      setMetricByLabel(card, "Tax Paid", `-${formatMoney(Math.abs(taxNum))}`, "negative");
    } else {
      setMetricByLabel(card, "Tax Paid", "—", "negative");
    }
  });

  // Auto-select first row
  const firstDataRow = table.querySelectorAll("tr")[1];
  if (firstDataRow) {
    firstDataRow.classList.add("is-selected");
    firstDataRow.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  }
}

// If your SPA swaps sections without reload, also call wireEmployeesTable()
// right after you render/show the employees page in your router.


// =========================
// Payroll Chart Initialize
// =========================

function initPayrollBreakdownCharts(root = document) {
  const charts = Array.from(
    root.querySelectorAll('.payroll_breakdown[data-chart="payroll-breakdown"]')
  );

  charts.forEach((component) => {
    const chart = component.querySelector(".breakdowns_chart");
    if (!chart) return;

    const breakdowns = Array.from(chart.querySelectorAll(".breakdown"));
    if (!breakdowns.length) return;

    // Totals for THIS chart instance
    const totals = breakdowns.map((b) => {
      const totalEl = b.querySelector(".breakdown_total");
      const total = totalEl ? Number(totalEl.textContent.trim()) : NaN;
      return Number.isFinite(total) ? total : 0;
    });

    const maxTotal = Math.max(...totals, 1);
    const maxBarHeight = 210;

    breakdowns.forEach((breakdown) => {
      const bar = breakdown.querySelector(".breakdown_bar");
      const totalEl = breakdown.querySelector(".breakdown_total");
      if (!bar || !totalEl) return;

      const total = Number(totalEl.textContent.trim()) || 0;
      const base = Number(bar.dataset.base || 0);
      const taxPaid = Number(bar.dataset.tax || 0);
      const companyContributions = Number(bar.dataset.contributions || 0);

      // Fallback to computed total if total is missing/0
      const computedTotal = base + taxPaid + companyContributions;
      const safeTotal = total > 0 ? total : computedTotal;

      // Scale bar height
      const scaledHeight = Math.max(18, Math.round((safeTotal / maxTotal) * maxBarHeight));
      bar.style.height = `${scaledHeight}px`;

      // Build segments
      bar.querySelectorAll(".seg").forEach((n) => n.remove());

      const denom = safeTotal || 1;
      const baseH = Math.round((base / denom) * scaledHeight);
      const taxH = Math.round((taxPaid / denom) * scaledHeight);
      const used = baseH + taxH;
      const contributionsH = Math.max(0, scaledHeight - used);

      const segBase = document.createElement("span");
      segBase.className = "seg base";
      segBase.style.height = `${baseH}px`;

      const segTax = document.createElement("span");
      segTax.className = "seg tax";
      segTax.style.height = `${taxH}px`;

      const segContributions = document.createElement("span");
      segContributions.className = "seg contributions";
      segContributions.style.height = `${contributionsH}px`;

      bar.appendChild(segBase);
      bar.appendChild(segTax);
      bar.appendChild(segContributions);
    });

    // Year switching scoped to THIS component
    const yearsWrap = component.querySelector(".breakdown_years");
    if (yearsWrap && !yearsWrap.dataset.bound) {
      yearsWrap.dataset.bound = "1";
      updateBreakdownYearControls(component, activePayrollBreakdownYear);

      yearsWrap.addEventListener("click", (e) => {
        const navBtn = e.target.closest("[data-year-nav]");
        if (!navBtn) return;

        const direction = navBtn.dataset.yearNav === "next" ? 1 : -1;
        const proposedYear = direction > 0
          ? Math.min(CURRENT_YEAR, activePayrollBreakdownYear + 1)
          : activePayrollBreakdownYear - 1;

        if (proposedYear === activePayrollBreakdownYear) return;
        if (typeof window.loadPayrollBreakdown === "function") {
          window.loadPayrollBreakdown(proposedYear);
        }
      });
    }
  });
}

// Make callable from router init if you want
window.wireEmployeesTable = wireEmployeesTable;
window.initPayrollBreakdownCharts = initPayrollBreakdownCharts;

// Run on load; router handles section-specific init (initOverviewPage, initEmployeesPage, etc.)
document.addEventListener("DOMContentLoaded", () => {
  initReports();
  if (window.handleLocation) window.handleLocation();
});
