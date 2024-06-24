function initialSetup() {
  setTheme()
  setCurrentlyLive()
}

// calls initialSetup() as soon as possible on page load
document.addEventListener("DOMContentLoaded", initialSetup)

// check light / dark mode on startup and write the setting to localStorage
function setTheme() {
    if (window.localStorage.getItem("dark") === null) {
        window.localStorage.setItem("dark", true)
    }
    if (window.localStorage.getItem("dark") === "true") {
        document.body.className = "dark-mode"
    }
    else {
        document.body.className = "light-mode"
        document.querySelector(".theme-toggle").innerText = "switch to dark mode"
    }
}

// check if any tournaments are currently live
function setCurrentlyLive() {
  const cards = document.querySelectorAll(".card")
  for (const card of cards) {
    const startTime = parseInt(card.getAttribute("data-start-time"))
    const endTime = parseInt(card.getAttribute("data-end-time"))
    const now = new Date().getTime() / 1000
    if (startTime <= now && now <= endTime) {
      const div = document.createElement("div")
      div.className = "live-badge"
      div.innerText = "LIVE NOW"
      card.appendChild(div)
    }
  }
}

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