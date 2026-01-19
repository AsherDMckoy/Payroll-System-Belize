const updateView = async (path, id) => {
    const navs = Array.from(document.getElementsByTagName("a"));
    const sections = Array.from(document.getElementsByTagName("section"));

    // Update navigation styling
    navs.forEach((nav) => {
        nav.style.color = nav.pathname === path ? "red" : "";
    });

    // Show/hide sections
    sections.forEach((section) => {
        section.hidden = section.id !== id;
    });
};

const routes = {
    "/": {
        view: () => updateView("/", "overview-section"),
        description: "Main dashboard of the payroll app",
    },
    "/payroll": {
        view: () => updateView("/payroll", "payroll-section"),
        description: "Process employee payroll",
    },
    "/employees": {
        view: () => updateView("/employees", "employees-section"),
        description: "Employees page of the payroll app",
    },
    "/reports": {
        view: () => updateView("/reports", "reports-section"),
        description: "Generate payroll reports",
    },
};

const handleLocation = async () => {
    const path = window.location.pathname;
    const route = routes[path] || routes["/"];
    route.view();
};

handleLocation();
// Intercept all <a> clicks for SPA routing
document.body.addEventListener("click", (event) => {
    const target = event.target.closest("a");
    console.log("click event triggered");
    // Check if it's an internal link
    if (target && target.href.startsWith(window.location.origin)) {
        event.preventDefault();
        window.history.pushState({}, "", target.href);
        handleLocation();
    }
});
window.onpopstate = handleLocation;
