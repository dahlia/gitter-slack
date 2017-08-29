`gitter-slack`: One-way relay from Gitter to Slack
==================================================

[![Cargo package][cargo-badge]][cargo]
[![Build status][travis-ci-badge]][travis-ci]

Relay chat messages from Gitter to Slack.

You can download a prebuilt executable binary (for Linux and macOS) from
the [releases][] page.  Of course, you need to give `+x` to it.

It's statically linked and mostly standalone:

    $ gitter-slack GITTER_ACCESS_TOKEN GITTER_ROOM SLACK_WEBHOOK_URL

You can get/determine these arguments through the following ways:

 -  `GITTER_ACCESS_TOKEN`: [Gitter Developer Program][1]
 -  `GITTER_ROOM`: A displayed room name on the *All Conversations* side bar
    (e.g. ``dahlia/gitter-slack``)
 -  `SLACK_WEBHOOK_URL`: [Slack Incoming WebHooks][2]

Distributed under [AGPLv3][3] or later.

[cargo-badge]: https://img.shields.io/crates/v/gitter-slack.svg
[cargo]: https://crates.io/crates/gitter-slack
[travis-ci-badge]: https://travis-ci.org/dahlia/gitter-slack.svg?branch=master
[travis-ci]: https://travis-ci.org/dahlia/gitter-slack
[releases]: https://github.com/dahlia/gitter-slack/releases
[1]: https://developer.gitter.im/apps
[2]: https://spoqa.slack.com/apps/A0F7XDUAZ
[3]: https://www.gnu.org/licenses/agpl.html


Changelog
---------

### Version 0.1.0

Initial release.  Released on August 26, 2017.
