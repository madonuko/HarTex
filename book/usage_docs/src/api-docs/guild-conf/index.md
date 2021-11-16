# Guild Configuration

This object is for guild-specific configuration, for example timezone, guild nickname, and some other things.

### Guild Configuration Object

#### Guild Configuration Structure

| FIELD              | TYPE     | DESCRIPTION                                                                              |
| ------------------ | -------- | ---------------------------------------------------------------------------------------- |
| nickname           | string?  | the nickname of the bot in the guild; "HarTex" by default                                |
| timezone           | string?  | the timezone of the guild, used when representing time; "UTC" by default                 |
| dmCannotUseCommand | boolean? | whether to send a DM to a user when they cannot execute some command; `false` by default |

#### Example Guild Configuration Object
```toml
[GuildConfiguration]
nickname = "HarTex"
timezone = "UTC"
dmCannotUseCommand = true
```

#### Valid Timezones

| TIMEZONE       | OFFSET    |
| -------------- | ------    |
| Asia/Hong_Kong | UTC+08:00 |
| UTC            | 0         |