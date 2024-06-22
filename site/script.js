// check light / dark mode on startup and write the setting to localStorage
function initialSetup() {
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

// calls initial_setup() as soon as possible on page load
document.addEventListener("DOMContentLoaded", initialSetup)

// change the class on <body> and write the setting to localStorage
function switchColors(event) {
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

function calendarButton(event) {
    const button = event.currentTarget
    button.innerText = "copied!"
    setTimeout(() => {button.innerText = "calendar"}, 3500)
    copyToClipboard("https://meleemajors.gg/calendar.ics")
}

function copyToClipboard(text) {
  if (!navigator.clipboard) {
    console.error("Clipboard API not supported", err);
    return;
  }
  navigator.clipboard.writeText(text).then(function () {
    console.log("Copied");
  }, function (err) {
    console.error("Copy", err);
  });
}