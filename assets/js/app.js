const header = document.getElementById("header-bar");
const headerHeight = header.offsetHeight;

document.documentElement.style.setProperty(
    "--header-height",
    headerHeight + "px",
);
