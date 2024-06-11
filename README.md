# meleemajors.gg

the successor to meleemajors.com

## backend

- on the backend, we are working on a static site generator written in rust - it reads a [tournaments.json](ssg/src/tournaments.json), and generates a tournament from each entry in that array

## hosting

- we use [github pages](https://pages.github.com) to do all the work for us, as long as our website stays static
- to set up, we registered a domain with [aws route 53](https://aws.amazon.com/route53/), verified the domain in `github settings > pages > verified domains`, and added the domain in the pages section of this repo
- after everything was set up, our domain records looked like this:

| type  | domain name                          | content             |
|-------|--------------------------------------|---------------------|
| A     | meleemajors.gg                       | 185.199.108.153     |
| A     | meleemajors.gg                       | 185.199.109.153     |
| A     | meleemajors.gg                       | 185.199.110.153     |
| A     | meleemajors.gg                       | 185.199.111.153     |
| AAAA  | meleemajors.gg                       | 2606:50c0:8000::153 |
| AAAA  | meleemajors.gg                       | 2606:50c0:8001::153 |
| AAAA  | meleemajors.gg                       | 2606:50c0:8002::153 |
| AAAA  | meleemajors.gg                       | 2606:50c0:8003::153 |
| ANAME | meleemajors.gg                       | jtof-dev.github.io  |
| TXT   | `challenge subdomain`.meleemajors.gg | `verification code` |
| CNAME | www.meleemajors.gg                   | jtof-dev.github.io  |

## analytics

- we use [umami](https://umami.is/) for basic analytics, like daily site views and how visitors interact with the website. while this could be useful improving the website, this is mostly because we want to know how much the website is getting used
- umami is a good choice for analytics, both because it has a free tier, and because it is relatively privacy-friendly (at least, according umami, for what that's worth)

## meleemajors.com

- a sample copy of `meleemajors.com` scraped from the wayback machine can be found [here](https://github.com/jtof-dev/meleemajors.gg/tree/meleemajors.com), taken from [the wayback machine](https://web.archive.org/web/20221202045414/https://www.meleemajors.com/)

## todo

- [x] add a black background image filter to the website background
- [x] finish formatting a template card
- [x] add custom fonts
- [x] fix footer not staying at the bottom of the page properly
- [x] flesh out footer with contributing instructions
- [x] add better padding for title
- [x] fix mobile formatting
- [x] add last card like the original website had
- [x] add dark mode / switch to dark mode
- [x] credit the artist for the mobile picture (I can't find the original artist)
- [x] make footer fit better relative to the text on-screen
- [ ] add mobile view with scroll snapping between cards
- [ ] add a currently live title for live tournaments
- [ ] add a way to view schedule images natively (have the card flip, expand,
  move to the side etc)
- [ ] and, of course, write the backend