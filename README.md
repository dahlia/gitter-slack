`gitter-slack`: One-way relay from Gitter to Slack
==================================================

Relay chat messages from Gitter to Slack.

    cargo run -- GITTER_ACCESS_TOKEN GITTER_ROOM SLACK_WEBHOOK_URL

You can get/determine these arguments through the following ways:

 -  `GITTER_ACCESS_TOKEN`: [Gitter Developer Program][1]
 -  `GITTER_ROOM`: A displayed room name on the *All Conversations* side bar
    (e.g. ``dahlia/gitter-slack``)
 -  `SLACK_WEBHOOK_URL`: [Slack Incoming WebHooks][2]

Distributed under [AGPLv3][3] or later.

[1]: https://developer.gitter.im/apps
[2]: https://spoqa.slack.com/apps/A0F7XDUAZ
[3]: https://www.gnu.org/licenses/agpl.html
