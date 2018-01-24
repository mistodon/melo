# Midscript (working title)

## TODOs

### Syntax

1.  Requiring commas after every attribute is error-prone - take newlines into account
2.  Requiring you to quote strings also sucks
3.  A missed out stave line before a blank line should be filled with rests instead of just unintuitively skipping ahead.
4.  On that note - maybe groups of stave lines should be required to be the same length in bars.

### Internals

1.  It kind of sucks having to manually set the channel for each voice - might default them to auto-incrementing.

### Errors

1.  Sequencing and generation phases need proper errors

### Features

1.  Urgently need note lengths supported
2.  Voice volume would be useful

### Bugs

1.  Generation crashes when a voice contains no bars/notes.
2.  Notes in tuplets are not split up properly

