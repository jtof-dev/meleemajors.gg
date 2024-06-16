# Contributing to meleemajors.gg

- first off, thanks for taking the time to contribute! ❤️
- since this is a small project, contributing is pretty casual. if you find a bug or want to contribute to the website, just open an issue or a pull request and we can talk about it there. but, for adding tournaments and tournament info, the easiest way would be to add / edit to [tournaments.json](ssg/src/tournaments.json)

## tournaments.json

- `tournaments.json` contains an array of tournament entries, each looking like the template one below

```
[
    {
        "start.gg-tournament-name": "<url tournament name>",
        "start.gg-melee-singles": "tournament/<url tournament name>/event/<melee singles event name>",
        "schedule": "<url or path to image>",
        "featured-players": [
            "TBD",
            "TBD",
            "TBD",
            "TBD",
            "TBD",
            "TBD",
            "TBD",
            "TBD"
        ],
        "stream": "<either NA or url to stream>"
    }
]
```

- both the `start.gg-tournament-name` and the `start.gg-melee-singles` fields are taken from the start.gg link to a tournament
  - for example, from [https://www.start.gg/tournament/tipped-off-15-connected-1/event/melee-singles](https://www.start.gg/tournament/tipped-off-15-connected-1/event/melee-singles), the `tournament-name` would be `tipped-off-15-connected-1`, and `melee-singles` would be `tournament/tipped-off-15-connected-1/event/melee-singles`

- the schedule url can either be a link to a website, or a relative link to an image starting from `site` - for example, `assets/schedules/tipped-off-15-schedule.webp`f

- finally, don't forget to add commas between tournament entries, like this:

```
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