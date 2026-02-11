const header = document.getElementById("header-bar");
const headerHeight = header.offsetHeight;

document.documentElement.style.setProperty(
  "--header-height",
  headerHeight + "px",
);

// =============================
// API-driven data (JSON from Rust backend)
// =============================

function formatCompactCurrency(numStr) {
  const n = parseInt(numStr, 10);
  if (!Number.isFinite(n)) return "—";
  if (n >= 1000000) return "$" + (n / 1000000).toFixed(1) + "M";
  if (n >= 1000) return "$" + (n / 1000).toFixed(0) + "K";
  return "$" + n;
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

async function loadDashboardOverview() {
  const grid = document.getElementById("dashboard-overview-grid");
  const tbody = document.getElementById("recent-activity-tbody");
  if (!grid || !tbody) return;

  try {
    const res = await fetch("/api/dashboard/overview");
    if (!res.ok) throw new Error(res.statusText);
    const data = await res.json();

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

    // Update pay period date range
    const startEl = grid.querySelector("[data-pay-period-start]");
    const endEl = grid.querySelector("[data-pay-period-end]");
    if (startEl) startEl.textContent = pp.start_date ?? "—";
    if (endEl) endEl.textContent = pp.end_date ?? "—";

    // Update and position payday bubble
    if (pp.payday) {
      const bubble = grid.querySelector("[data-payday-bubble]");
      const paydayDateEl = grid.querySelector("[data-payday-date]");

      if (bubble && paydayDateEl) {
        paydayDateEl.textContent = pp.payday.date || "—";

        // Calculate bubble position as percentage of total days
        const dayOfMonth = pp.payday.day_of_month;
        const totalDays = pp.payday.total_days_in_period;

        if (dayOfMonth && totalDays) {
          const percentage = (dayOfMonth / totalDays) * 100;
          bubble.style.left = `${percentage}%`;
          bubble.hidden = false;
        }
      }

      // Update breakdown details
      const breakdownSection = grid.querySelector("[data-payday-breakdown]");
      if (breakdownSection) {
        const baseEl = breakdownSection.querySelector("[data-payday-base]");
        const overtimeEl = breakdownSection.querySelector("[data-payday-overtime]");
        const incentivesEl = breakdownSection.querySelector("[data-payday-incentives]");
        const totalEl = breakdownSection.querySelector("[data-payday-total]");

        if (baseEl) baseEl.textContent = pp.payday.base_salary || "—";
        if (overtimeEl) overtimeEl.textContent = pp.payday.overtime || "—";
        if (incentivesEl) incentivesEl.textContent = pp.payday.incentives || "—";
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
  } catch (err) {
    console.error("loadDashboardOverview:", err);
    if (tbody) {
      tbody.innerHTML = "<tr><td colspan=\"4\">Failed to load</td></tr>";
    }
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
    const res = await fetch("/api/payroll-breakdown?year=" + encodeURIComponent(year || 2026));
    if (!res.ok) throw new Error(res.statusText);
    const data = await res.json();

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
        const overtime = m ? m.overtime : 0;
        const incentives = m ? m.incentives : 0;

        const totalEl = b.querySelector(".breakdown_total");
        const bar = b.querySelector(".breakdown_bar");
        const monthEl = b.querySelector(".breakdown_month");
        if (totalEl) totalEl.textContent = String(total);
        if (bar) {
          bar.dataset.base = String(base);
          bar.dataset.overtime = String(overtime);
          bar.dataset.incentives = String(incentives);
        }
        if (monthEl) monthEl.textContent = m ? m.label : monthLabels[i] || "";

        const tooltipList = b.querySelectorAll(".tooltip_list_item .tooltip_breakdown_value");
        if (tooltipList.length >= 3) {
          tooltipList[0].textContent = String(base);
          tooltipList[1].textContent = String(overtime);
          tooltipList[2].textContent = String(incentives);
        }
      }
    });

    initPayrollBreakdownCharts();
  } catch (err) {
    console.error("loadPayrollBreakdown:", err);
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
      tr.innerHTML = `
        <td>${escapeHtml(emp.name)}</td>
        <td>${escapeHtml(emp.position)}</td>
        <td>${escapeHtml(emp.department)}</td>
        <td class="is-numeric">${formatCurrency(emp.net_salary)}</td>
        <td class="is-numeric">${formatCurrency(emp.contributions)}</td>
        <td class="is-numeric">${formatCurrency(emp.deductions)}</td>
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
  const chart = document.querySelector('.payroll_breakdown[data-chart="payroll-breakdown"]');
  const activeYearBtn = chart && chart.querySelector(".year_chip.is-active");
  const year = activeYearBtn ? (activeYearBtn.dataset.year || "2026") : "2026";
  loadPayrollBreakdown(year);
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
  return `${period.start_date} to ${period.end_date}${status}`;
}

async function loadPayrollPeriods() {
  const select = document.getElementById("pay-period");
  if (!select) return;

  try {
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
    }
  } catch (err) {
    console.error("loadPayrollPeriods:", err);
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
    }),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || "Failed to generate payroll.");
  }

  return res.json();
}

function wirePayrollActions() {
  if (payrollActionsWired) return;
  payrollActionsWired = true;

  const newPayrollBtn = document.querySelector(".new_payroll_btn");
  if (!newPayrollBtn) return;

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
      loadDashboardOverview();
      loadEmployees();
      loadPayrollPeriods();
    } catch (err) {
      console.error("generatePayrollForSelectedPeriod:", err);
      alert(`Payroll generation failed: ${err.message || err}`);
    } finally {
      newPayrollBtn.disabled = false;
      newPayrollBtn.textContent = originalText;
    }
  });
}

// Payroll process page init – populate chart so it works when landing directly on /payroll
function initPayrollPage() {
  const chart = document.querySelector('.payroll_breakdown[data-chart="payroll-breakdown"]');
  const activeYearBtn = chart && chart.querySelector(".year_chip.is-active");
  const year = activeYearBtn ? (activeYearBtn.dataset.year || "2026") : "2026";
  loadPayrollBreakdown(year);
  loadPayrollPeriods();
  wirePayrollActions();
  updatePayrollPeriodDetails();
}

// Update pay period details with circle graph animation
function updatePayrollPeriodDetails() {
  // This would normally come from an API call
  // For now, using the hardcoded values from the HTML and calculating from them
  const grossPayEl = document.querySelector('[data-gross-pay]');
  const deductionsEl = document.querySelector('[data-deductions]');
  const totalNetPayEl = document.querySelector('[data-total-net-pay]');
  const circleProgress = document.querySelector('[data-circle-progress]');
  const circlePercentage = document.querySelector('[data-circle-percentage]');

  if (!grossPayEl || !deductionsEl || !circleProgress || !circlePercentage) return;

  // Parse the monetary values
  const grossPay = parseMoney(grossPayEl.textContent);
  const deductions = parseMoney(deductionsEl.textContent);

  if (!grossPay || !deductions) return;

  // Calculate net pay and percentage
  const netPay = grossPay - deductions;
  const netPercentage = (netPay / grossPay) * 100;

  // Update total net pay display
  if (totalNetPayEl) {
    totalNetPayEl.textContent = formatMoney(netPay);
  }

  // Calculate circle progress (circumference = 2 * π * r = 2 * π * 40 ≈ 251.2)
  const radius = 40;
  const circumference = 2 * Math.PI * radius;
  const progressLength = (netPercentage / 100) * circumference;

  // Animate the circle
  requestAnimationFrame(() => {
    circleProgress.style.strokeDasharray = `${progressLength} ${circumference}`;
    circlePercentage.textContent = `${Math.round(netPercentage)}%`;
  });
}

// Alternative version that accepts data from API
function updatePayrollPeriodDetailsFromData(data) {
  const {
    payrollPeriod,
    totalEmployees,
    payDay,
    status,
    grossPay,
    deductions,
  } = data;

  // Update info grid
  const periodEl = document.querySelector('[data-payroll-period]');
  const employeesEl = document.querySelector('[data-total-employees]');
  const payDayEl = document.querySelector('[data-pay-day]');
  const statusEl = document.querySelector('[data-payroll-status]');

  if (periodEl) periodEl.textContent = payrollPeriod || '—';
  if (employeesEl) employeesEl.textContent = totalEmployees || '—';
  if (payDayEl) payDayEl.innerHTML = payDay || '—';

  if (statusEl && status) {
    const indicator = statusEl.querySelector('.status_indicator');
    statusEl.childNodes.forEach(node => {
      if (node.nodeType === Node.TEXT_NODE) {
        node.textContent = status;
      }
    });

    // Update status indicator color
    if (indicator) {
      indicator.className = 'status_indicator';
      if (status.toLowerCase() === 'paid') {
        indicator.classList.add('status_paid');
      } else if (status.toLowerCase() === 'pending') {
        indicator.classList.add('status_pending');
      } else {
        indicator.classList.add('status_scheduled');
      }
    }
  }

  // Update financial values
  const grossPayEl = document.querySelector('[data-gross-pay]');
  const deductionsEl = document.querySelector('[data-deductions]');
  const totalNetPayEl = document.querySelector('[data-total-net-pay]');

  if (grossPayEl) grossPayEl.textContent = formatMoney(grossPay);
  if (deductionsEl) deductionsEl.textContent = formatMoney(deductions);

  // Calculate and update circle graph
  const netPay = grossPay - deductions;
  const netPercentage = (netPay / grossPay) * 100;

  if (totalNetPayEl) totalNetPayEl.textContent = formatMoney(netPay);

  const circleProgress = document.querySelector('[data-circle-progress]');
  const circlePercentage = document.querySelector('[data-circle-percentage]');

  if (circleProgress && circlePercentage) {
    const radius = 40;
    const circumference = 2 * Math.PI * radius;
    const progressLength = (netPercentage / 100) * circumference;

    requestAnimationFrame(() => {
      circleProgress.style.strokeDasharray = `${progressLength} ${circumference}`;
      circlePercentage.textContent = `${Math.round(netPercentage)}%`;
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

  const tabs = Array.from(root.querySelectorAll(".tab"));
  const cardsWrap = root.querySelector("#reports-cards");
  const listTitle = root.querySelector("#reports-list-title");
  const listSubtitle = root.querySelector("#reports-list-subtitle");
  const search = root.querySelector("#reports-search");
  const dynamicFilters = root.querySelector("#reports-dynamic-filters");
  const selectedHint = root.querySelector("#reports-selected-hint");
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
      title: "Employee Data",
      subtitle: "Listings, profiles, and pay summaries per employee.",
    },
    deductions: {
      title: "Deductions",
      subtitle: "SSB/Tax/Court/Lodgements/Vacation/Insurance and employer contributions.",
    },
    general: {
      title: "General Data",
      subtitle: "Gross expenses, trends, growth/shrinkage, and period comparisons.",
    },
  };

  // Define reports + the extra filters each one needs.
  const reports = [
    // Employee Data
    {
      id: "emp_listing",
      category: "employee",
      title: "Employee Listing",
      desc: "A clean list of employees with department, status, and hire date.",
      tags: ["HR", "Directory"],
      filters: [
        { type: "select", name: "status", label: "Status", options: ["All", "Active", "Inactive"] },
        { type: "select", name: "department", label: "Department", options: ["All", "Admin", "Sales", "Operations", "Finance"] },
      ],
    },
    {
      id: "emp_profile",
      category: "employee",
      title: "Employee Information",
      desc: "Detailed information for a single employee (profile view).",
      tags: ["HR"],
      filters: [
        { type: "employee", name: "employee", label: "Employee" },
        { type: "toggle", name: "include_contacts", label: "Include contact details" },
      ],
    },
    {
      id: "emp_pay",
      category: "employee",
      title: "Employee Pay Summary",
      desc: "Gross, deductions, and net pay for employees across a period.",
      tags: ["Payroll", "Summary"],
      filters: [
        { type: "select", name: "group_by", label: "Group by", options: ["Employee", "Department"] },
        { type: "toggle", name: "include_overtime", label: "Include overtime breakdown" },
      ],
    },

    // Deductions
    {
      id: "deductions_summary",
      category: "deductions",
      title: "Deductions Summary",
      desc: "Totals per deduction type (employee + employer contributions if applicable).",
      tags: ["SSB", "Tax"],
      filters: [
        {
          type: "multiselect",
          name: "deduction_types",
          label: "Deduction types",
          options: ["Social Security", "Tax", "Courts", "Lodgements", "Vacation", "Insurance", "Employer Contributions"],
        },
        { type: "toggle", name: "split_employer", label: "Split employer vs employee" },
      ],
    },
    {
      id: "tax_register",
      category: "deductions",
      title: "Tax Register",
      desc: "Tax withheld per employee, suitable for filing/reconciliation.",
      tags: ["Compliance"],
      filters: [
        { type: "select", name: "tax_mode", label: "Tax mode", options: ["All", "PAYE", "Flat", "Other"] },
        { type: "toggle", name: "include_tin", label: "Include TIN/ID column" },
      ],
    },

    // General Data
    {
      id: "gross_expenses",
      category: "general",
      title: "Gross Payroll Expenses",
      desc: "Total payroll cost for the selected period, with trend line.",
      tags: ["Finance"],
      filters: [
        { type: "select", name: "granularity", label: "Granularity", options: ["Monthly", "Bi-weekly", "Weekly"] },
        { type: "toggle", name: "include_employer", label: "Include employer contributions" },
      ],
    },
    {
      id: "growth_shrinkage",
      category: "general",
      title: "Growth / Shrinkage",
      desc: "Compare headcount and payroll changes across two periods.",
      tags: ["Trends"],
      filters: [
        { type: "select", name: "compare_to", label: "Compare to", options: ["Previous period", "Same period last year", "Custom"] },
        { type: "toggle", name: "show_headcount", label: "Include headcount" },
      ],
    },
    {
      id: "payroll_annual",
      category: "general",
      title: "Monthly / Annual Payroll",
      desc: "Breakdown of payroll totals by month, plus annual roll-up.",
      tags: ["Budget"],
      filters: [
        { type: "select", name: "year", label: "Year", options: ["2026", "2025", "2024"] },
        { type: "toggle", name: "include_charts", label: "Include charts" },
      ],
    },
  ];

  let activeCategory = "employee";
  let selectedReportId = null;

  function setModalOpen(which, open) {
    const el = which === "help" ? helpModal : modal;
    if (!el) return;
    el.hidden = !open;
    el.setAttribute("aria-hidden", String(!open));
    document.body.style.overflow = open ? "hidden" : "";
  }

  function renderDynamicFilters(report) {
    dynamicFilters.innerHTML = "";
    if (!report) return;

    for (const f of report.filters) {
      const row = document.createElement("div");
      row.className = "form-row";

      if (f.type === "toggle") {
        row.innerHTML = `
          <label class="chip" style="justify-content:flex-start;">
            <input type="checkbox" name="${f.name}" value="1" /> ${f.label}
          </label>`;
        dynamicFilters.appendChild(row);
        continue;
      }

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
        for (const opt of f.options) {
          const o = document.createElement("option");
          o.value = opt.toLowerCase().replace(/\s+/g, "_");
          o.textContent = opt;
          select.appendChild(o);
        }
        row.appendChild(select);
      } else if (f.type === "multiselect") {
        const wrap = document.createElement("div");
        wrap.className = "chips";
        for (const opt of f.options) {
          const chip = document.createElement("label");
          chip.className = "chip";
          chip.innerHTML = `<input type="checkbox" name="${f.name}" value="${opt}" checked /> ${opt}`;
          wrap.appendChild(chip);
        }
        row.appendChild(wrap);
      } else if (f.type === "employee") {
        const select = document.createElement("select");
        select.className = "select";
        select.id = id;
        select.name = f.name;
        // Placeholder options (wire to API later)
        ["Select…", "John Doe", "Jane Smith", "Alex Williams"].forEach((name, idx) => {
          const o = document.createElement("option");
          o.value = idx === 0 ? "" : name;
          o.textContent = name;
          if (idx === 0) o.disabled = false;
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

  function setCategory(cat) {
    activeCategory = cat;
    selectedReportId = null;
    generateBtn.disabled = true;
    previewBtn.disabled = true;
    selectedHint.textContent = "Select a report to configure options.";
    renderDynamicFilters(null);
    search.value = "";
    listTitle.textContent = categories[cat].title;
    listSubtitle.textContent = categories[cat].subtitle;

    tabs.forEach((t) => {
      const active = t.dataset.category === cat;
      t.classList.toggle("is-active", active);
      t.setAttribute("aria-selected", String(active));
    });
    renderCards();
  }

  function setReport(id) {
    selectedReportId = id;
    const report = reports.find((r) => r.id === id);
    selectedHint.textContent = report ? `Selected: ${report.title}` : "Select a report to configure options.";
    generateBtn.disabled = !report;
    previewBtn.disabled = !report;
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

  function mapReportType(reportId) {
    if (!reportId) return null;
    if (reportId === "emp_pay") return "employee_payroll";
    if (reportId === "deductions_summary" || reportId === "tax_register") {
      return "deductions_summary";
    }
    if (reportId === "gross_expenses" || reportId === "growth_shrinkage" || reportId === "payroll_annual") {
      return "payroll_period_summary";
    }
    return null;
  }

  function extractFilename(contentDisposition, fallback) {
    if (!contentDisposition) return fallback;
    const match = contentDisposition.match(/filename=\"?([^"]+)\"?/i);
    return match ? match[1] : fallback;
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

    const reportType = mapReportType(report.id);
    if (!reportType) {
      alert("This report is not yet wired to backend generation.");
      return;
    }

    const form = root.querySelector("#reports-filters-form");
    const formData = new FormData(form);
    const preferredFormat = String(formData.get("format") || "csv").toLowerCase();
    const backendFormat = preferredFormat === "pdf" || preferredFormat === "xlsx"
      ? "csv"
      : preferredFormat;

    if (preferredFormat !== backendFormat) {
      alert(`Backend export currently supports CSV/JSON. Generating ${backendFormat.toUpperCase()} instead.`);
    }

    const payload = {
      report_type: reportType,
      format: backendFormat,
    };

    const payrollPeriodSelect = document.getElementById("pay-period");
    if (payrollPeriodSelect && payrollPeriodSelect.value) {
      const periodId = Number(payrollPeriodSelect.value);
      if (Number.isFinite(periodId)) payload.payroll_period_id = periodId;
    }

    if (formData.get("range") === "custom") {
      const start = formData.get("start");
      const end = formData.get("end");
      if (start) payload.start_date = start;
      if (end) payload.end_date = end;
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

      if (backendFormat === "csv") {
        const blob = await res.blob();
        const filename = extractFilename(
          res.headers.get("content-disposition"),
          `${reportType}.csv`,
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
    const obj = {};
    for (const [k, v] of fd.entries()) {
      if (obj[k]) {
        if (Array.isArray(obj[k])) obj[k].push(v);
        else obj[k] = [obj[k], v];
      } else {
        obj[k] = v;
      }
    }

    const pretty = (val) => {
      if (Array.isArray(val)) return val.join(", ");
      return val || "—";
    };

    const rows = Object.entries(obj)
      .map(([k, v]) => `<div style="display:flex;justify-content:space-between;gap:12px;"><span style="color:rgba(255,255,255,0.72)">${k}</span><span>${pretty(v)}</span></div>`)
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
    const deductNum = parseMoney(cells[5].textContent);

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
    setMetricByLabel(card, "Net Salary", netNum !== null ? formatMoney(netNum) : "—");
    setMetricByLabel(card, "Contributions", contribNum !== null ? formatMoney(contribNum) : "—");

    // Show deductions as negative in the card
    if (deductNum !== null) {
      setMetricByLabel(card, "Deductions", `-${formatMoney(Math.abs(deductNum))}`, "negative");
    } else {
      setMetricByLabel(card, "Deductions", "—", "negative");
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
      const overtime = Number(bar.dataset.overtime || 0);
      const incentives = Number(bar.dataset.incentives || 0);

      // Fallback to computed total if total is missing/0
      const computedTotal = base + overtime + incentives;
      const safeTotal = total > 0 ? total : computedTotal;

      // Scale bar height
      const scaledHeight = Math.max(18, Math.round((safeTotal / maxTotal) * maxBarHeight));
      bar.style.height = `${scaledHeight}px`;

      // Build segments
      bar.querySelectorAll(".seg").forEach((n) => n.remove());

      const denom = safeTotal || 1;
      const baseH = Math.round((base / denom) * scaledHeight);
      const overtimeH = Math.round((overtime / denom) * scaledHeight);
      const used = baseH + overtimeH;
      const incentivesH = Math.max(0, scaledHeight - used);

      const segBase = document.createElement("span");
      segBase.className = "seg base";
      segBase.style.height = `${baseH}px`;

      const segOver = document.createElement("span");
      segOver.className = "seg overtime";
      segOver.style.height = `${overtimeH}px`;

      const segInc = document.createElement("span");
      segInc.className = "seg incentives";
      segInc.style.height = `${incentivesH}px`;

      bar.appendChild(segBase);
      bar.appendChild(segOver);
      bar.appendChild(segInc);
    });

    // Year switching scoped to THIS component
    const yearsWrap = component.querySelector(".breakdown_years");
    if (yearsWrap && !yearsWrap.dataset.bound) {
      yearsWrap.dataset.bound = "1";

      yearsWrap.addEventListener("click", (e) => {
        const btn = e.target.closest(".year_chip");
        if (!btn) return;

        yearsWrap.querySelectorAll(".year_chip").forEach((b) => b.classList.remove("is-active"));
        btn.classList.add("is-active");

        const year = btn.dataset.year;
        if (typeof window.loadPayrollBreakdown === "function") {
          window.loadPayrollBreakdown(year);
        } else {
          initPayrollBreakdownCharts(component);
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
