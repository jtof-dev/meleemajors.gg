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

function copyCalendar() {
  copyToClipboard("https://meleemajors.gg/calendar.ics")
}

function calendarButton(event) {
  const contents = document.querySelector(".calendar-note")
  contents.classList.toggle("calendar-note-hidden")
  if (!contents.classList.contains("calendar-note-hidden") && window.scrollY === 0) {
    scrollToBottom()
  }

  // // Calculate the bottom threshold as a percentage of the total scrollable height
  // const bottomThreshold = 0.3; // Adjust this value as needed (0.1 means 10% from the bottom)

  // // Calculate how far from the bottom the user is
  // const scrollableHeight = document.documentElement.scrollHeight;
  // const viewportHeight = window.innerHeight;
  // const scrollPosition = window.scrollY || document.documentElement.scrollTop;

  // const bottomOffset = scrollableHeight - viewportHeight;

  // if (!contents.classList.contains("calendar-note-hidden") && scrollPosition >= bottomOffset * (1 - bottomThreshold)) {
  //   // this correctly identifies the bottom of the page, but the scrolling is jerky
  //   scrollToBottom();
  // }
}

function scrollToBottom() {
  const startTime = Date.now();
  const animationDuration = 400;
  const speed = 20 * 60 // pixels per second
  let lastFrameTime = undefined;
  const animateScroll = (time) => {
    const deltaTime = time - (lastFrameTime || time)
    lastFrameTime = time
    scrollTo({ top: window.scrollY + (deltaTime / 1000) * speed, behavior: 'instant' })
    if (Date.now() - startTime < animationDuration) requestAnimationFrame(animateScroll)
  }
  animateScroll();
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