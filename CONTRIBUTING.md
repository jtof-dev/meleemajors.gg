# Contributing to meleemajors.gg

- thanks for taking the time to contribute! ❤️
- since this is a small project, contributing is pretty casual. if you find a bug or want to contribute to the website, just open an issue or a pull request and we can talk about it there. but, for adding tournaments and tournament info, the easiest way would be to add / edit to [tournaments.json](ssg/src/tournaments.json)

## tournaments.json

- `tournaments.json` contains an array of tournament entries, each looking like the template one below

```jsonc
{
  "start.gg-melee-singles-url": "url to melee singles",
  "top8-start-time": "YYYY-MM-DD hh:mmPM", // in the tournament's timezone
  "schedule-url": "either empty quotes or url or path to image",
  "stream-url": "either empty quotes or url to stream",
}
```

- the `start.gg-melee-singles-url` field is for the start.gg url to melee singles (like [https://www.start.gg/tournament/tipped-off-15-connected-1/event/melee-singles](https://www.start.gg/tournament/tipped-off-15-connected-1/event/melee-singles))
- the schedule url can either be a link to a website, or a relative link to an image starting from `site` - for example, `assets/schedules/tipped-off-15-schedule.webp`
- for both `schedule-url` and `stream-url`, just leave the value blank (`""`) if there is no information about either yet
- finally, don't forget to add commas between tournament entries, like this:

```json
[
    {
        ...
    },
    {
        ...
    },
    {
        ...
    }
]
```
