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
  const calendarContents = document.querySelector(".calendar-note")
  const emailContents = document.querySelector(".email-note")

  const isEmailOpen = !emailContents.classList.contains("email-note-hidden")

  if (isEmailOpen) {
    emailContents.classList.add("email-note-hidden")
  }

  setTimeout(() => {
    calendarContents.classList.toggle("calendar-note-hidden")
    if (!calendarContents.classList.contains("calendar-note-hidden")) {
      scrollToBottom()
    }
  }, isEmailOpen ? 300 : 0)
}

function emailButton(event) {
  const calendarContents = document.querySelector(".calendar-note")
  const emailContents = document.querySelector(".email-note")

  const isCalendarOpen = !calendarContents.classList.contains("calendar-note-hidden")

  if (isCalendarOpen) {
    calendarContents.classList.add("calendar-note-hidden")
  }

  setTimeout(() => {
    emailContents.classList.toggle("email-note-hidden")
    if (!emailContents.classList.contains("email-note-hidden")) {
      scrollToBottom()
    }
  }, isCalendarOpen ? 300 : 0)
}

function scrollToBottom() {
  const startTime = Date.now();
  const animationDuration = 400;
  const speed = 30 * 60 // pixels per second
  let lastFrameTime = undefined;
  const animateScroll = (time) => {
    const deltaTime = time - (lastFrameTime || time)
    lastFrameTime = time
    const top = window.scrollY + (deltaTime / 1000) * speed
    scrollTo({ top, behavior: 'instant' })
    if (Date.now() - startTime < animationDuration) requestAnimationFrame(animateScroll)
  }
  requestAnimationFrame(animateScroll)
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

async function emailSignup(event) {
  // Loading
  event.target.innerText = "Sending..."
  event.target.disabled = true

  // Request
  const emailInput = document.getElementById("mce-EMAIL")
  const email = emailInput.value
  const url = "https://meleemajors.us11.list-manage.com/subscribe/post?u=e07dc2c0f2663f546ed1d7448&amp;id=0e73a47f3c&amp;f_id=00c918e0f0"
  const response = await fetch(url, {
    method: "POST",
    body: new URLSearchParams({ EMAIL: email }),
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    mode: "no-cors"
  })

  // Success
  console.log(`added ${email}`)
  event.target.innerText = "Subscribed!"

  // Reset
  setTimeout(() => {
    event.target.innerText = "Subscribe"
    event.target.disabled = false
  }, 3000)
}