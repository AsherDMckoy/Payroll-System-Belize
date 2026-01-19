const header = document.getElementById("header-bar");
const headerHeight = header.offsetHeight;

document.documentElement.style.setProperty(
    "--header-height",
    headerHeight + "px",
);

// =============================
// Reports UI (frontend-only)
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

    modalGenerate.addEventListener("click", () => {
        // Frontend-only placeholder: later call backend and download file.
        setModalOpen("main", false);
        // Lightweight toast using alert for now; swap with a real toast later.
        alert("Report generation hooked up! (UI placeholder)\n\nNext step: POST filter values to your backend and download the file.");
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

document.addEventListener("DOMContentLoaded", () => {
    initReports();
});
