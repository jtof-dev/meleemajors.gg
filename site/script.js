function initialSetup() {
  setTheme()
  setCurrentlyLive()
  hidePastTournaments()
  setStreambutton()
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
    document.querySelector(".theme-toggle").innerText = "dark mode"
  }
}


function setCurrentlyLive() {
  const cards = Array.from(document.querySelectorAll(".card"))
  
  if (cards.length === 0) return;

  const earliestCard = cards.reduce((earliest, current) => {
    const earliestTime = parseInt(earliest.getAttribute("data-start-time"), 10);
    const currentTime= parseInt(current.getAttribute("data-start-time"), 10);

    return currentTime < earliestTime ? current : earliest;
  });

  const startTime = parseInt(earliestCard.getAttribute("data-start-time"), 10);
  const endTime = parseInt(earliestCard.getAttribute("data-end-time"), 10);
  const now = Date.now() / 1000;
  if (startTime <= now && now <= endTime) {
    const div = document.createElement("div")
    div.className = "live-badge"
    div.innerText = "LIVE NOW"
    card.appendChild(div)
  } else if (now < startTime) {
    const timeDiff = startTime - now;
    const days = Math.floor(timeDiff / 86400);
    const hours = Math.floor((timeDiff / 3600) % 24);
    const minutes = Math.floor((timeDiff / 60) % 60);

    const div = document.createElement("div")
    div.className = "live-badge"
    div.textContent = `${days}D ${hours}H ${minutes}M`;
    earliestCard.appendChild(div);

    startLiveCountdown(div, startTime);
  }
}


function startLiveCountdown(badgeElement, startTime) {
  const timerId = setInterval(() => {
    // if the badge is removed from the DOM, stop the timer
    if (!document.body.contains(badgeElement)) {
      clearInterval(timerId);
      return;
    }

    const currentNow = Math.floor(Date.now() / 1000); 
    const timeDiff = startTime - currentNow;

    if (timeDiff <= 0) {
      clearInterval(timerId);
      badgeElement.textContent = "Started!";
      return;
    }

    const days = Math.floor(timeDiff / 86400);
    const hours = Math.floor((timeDiff / 3600) % 24);
    const minutes = Math.floor((timeDiff / 60) % 60);

    badgeElement.textContent = `${days}D ${hours}H ${minutes}M`;
  }, 1000);

  // return the timer ID just in case we ever need to manually stop it elsewhere
  return timerId;
}


function hidePastTournaments() {
  const cards = document.querySelectorAll(".card")
  for (const card of cards) {
    const startTime = parseInt(card.getAttribute("data-start-time"))
    const endTime = parseInt(card.getAttribute("data-end-time"))
    const now = new Date().getTime() / 1000
    if (now > endTime) {
      card.remove()
    }
  }
}

// change the class on <body> and write the setting to localStorage
function switchColors(event) {
  if (window.localStorage.getItem("dark") === "true") { // if true
    document.body.className = "light-mode"
    window.localStorage.setItem("dark", false)
    event.currentTarget.innerText = "dark mode"
  }
  else {
    document.body.className = "dark-mode"
    window.localStorage.setItem("dark", true)
    event.currentTarget.innerText = "light mode"
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


function setStreambutton() {
  const cards = document.querySelectorAll(".card")
  for (const card of cards) {
    const startTime = parseInt(card.getAttribute("data-start-time"))
    const weekBeforeStart = startTime - 604800 // 604800 = 1 week in seconds
    // const endTime = parseInt(card.getAttribute("data-end-time"))
    // const weekBeforeEnd = endTime - 604800
    const now = Date.now() / 1000;
    if (weekBeforeStart >= now) {
      card.querySelector(".stream-button").style.display="none"
    }
  }
}