// check light / dark mode on startup and write the setting to localStorage
function initial_setup() {
    if (window.localStorage.getItem("dark") === null) {
        window.localStorage.setItem("dark", true)
    }
    if (window.localStorage.getItem("dark") === "true") {
        document.body.className = "dark-mode"
        document.getElementById("checkbox").checked = 1
    }
    else {
        document.body.className = "light-mode"
        document.getElementById("checkbox").checked = 0
    }
}

// change the class on <body> and write the setting to localStorage
function switchcolors(event) {
    if (window.localStorage.getItem("dark") === "true") { // if true
        document.body.className = "light-mode"
        window.localStorage.setItem("dark", false)
        event.currentTarget.innerText = "switch to dark mode"
    }
    else {
        document.body.className = "dark-mode"
        window.localStorage.setItem("dark", true)
        event.currentTarget.innerText = "switch to light mode"
    }
}

// calls initial_setup() as soon as possible on page load
document.addEventListener("DOMContentLoaded", initial_setup)