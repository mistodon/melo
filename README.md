# Midscript (working title)

## TODOs

### Syntax

1.  Requiring commas after every attribute is error-prone - take newlines into account
2.  Requiring you to quote strings also sucks
3.  A missed out stave line before a blank line should be filled with rests instead of just unintuitively skipping ahead.
4.  On that note - maybe groups of stave lines should be required to be the same length in bars, with an optional `~` at the end to pad with rests.
5.  It kind of sucks having to manually set the channel for each voice - might default them to auto-incrementing.

### Errors

1.  Error about unsupported tuplets cannot always tell you which bar caused the problem.

### Features

1.  Repeats of previous bars
2.  Declaring parts that can be played in multiple places

