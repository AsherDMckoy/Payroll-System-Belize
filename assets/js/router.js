// router.js

// --------- View / Page Switching ---------
const getMainSections = () => {
  // Only top-level pages
  return Array.from(document.querySelectorAll("main > section"));
};

const setActiveNav = (path) => {
  const navLinks = Array.from(document.querySelectorAll(".main_navigation a"));

  navLinks.forEach((a) => {
    const isActive = normalizePath(a.pathname) === normalizePath(path);
    a.classList.toggle("is-active", isActive);

    if (isActive) a.setAttribute("aria-current", "page");
    else a.removeAttribute("aria-current");
  });
};

const showSectionById = (id) => {
  const sections = getMainSections();
  sections.forEach((section) => {
    section.hidden = section.id !== id;
  });
};

const updateView = async (path, sectionId, initFn) => {
  setActiveNav(path);
  showSectionById(sectionId);

  // Give the browser a beat to apply layout before running init
  requestAnimationFrame(() => {
    if (typeof initFn === "function") initFn();
  });
};

// --------- Routing ---------
const routes = {
  "/": {
    view: () => updateView("/", "overview-section", window.initOverviewPage),
    description: "Main dashboard of the payroll app",
  },
  "/payroll": {
    view: () => updateView("/payroll", "payroll-section", window.initPayrollPage),
    description: "Process employee payroll",
  },
  "/employees": {
    view: () =>
      updateView("/employees", "employees-section", window.wireEmployeesTable),
    description: "Employees page of the payroll app",
  },
  "/reports": {
    view: () => updateView("/reports", "reports-section", window.initReportsPage),
    description: "Generate payroll reports",
  },
};

function normalizePath(p) {
  if (!p) return "/";
  // ensure leading slash
  let out = p.startsWith("/") ? p : `/${p}`;
  // remove trailing slashes (except root)
  out = out.replace(/\/+$/, "");
  return out === "" ? "/" : out;
}

const handleLocation = async () => {
  const path = normalizePath(window.location.pathname);
  const route = routes[path] || routes["/"];
  route.view();
};

handleLocation();

// --------- Link Interception (SPA) ---------
document.body.addEventListener("click", (event) => {
  const a = event.target.closest("a");
  if (!a) return;

  // Ignore modified clicks (new tab, etc.)
  if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;

  // Ignore external links
  const url = new URL(a.href, window.location.origin);
  if (url.origin !== window.location.origin) return;

  // Ignore anchors to same-page sections
  if (a.getAttribute("href")?.startsWith("#")) return;

  // Ignore downloads
  if (a.hasAttribute("download")) return;

  // Only handle links that match our routes
  const targetPath = normalizePath(url.pathname);
  if (!routes[targetPath]) return;

  event.preventDefault();
  window.history.pushState({}, "", targetPath);
  handleLocation();
});

window.addEventListener("popstate", handleLocation);
