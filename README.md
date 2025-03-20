# goldendict-helper

[Goldendict-ng](https://github.com/xiaoyifang/goldendict-ng) doesn't have an inherient review system(what a pitty).

Usally we use goldendict-ng with anki : https://xiaoyifang.github.io/goldendict-ng/topic_anki/

This approach depends on anki-connect: we have to always open anki, which is quite annoying.

So we made this project to replace anki

![Alt Text](assets/output.gif)


- pros
    - 避免了 anki 中的 bug
        - https://github.com/xiaoyifang/goldendict-ng/discussions/1885
    - 不用打开 anki-connect
        - 相当于 headless anki
    - 比 anki 少点一次鼠标
    - Customize review strategy
        - For example, similar words first, verb first ...

- cons
    - 复习计划制定的不如 anki
    - theme 不够好看
    - only work on [awesomewm](https://awesomewm.org/)
        - Possible to extend to other window manager

## Install

```
git clone https://github.com/lengyijun/goldendict-ng-reviewer
cd goldendict-ng-reviewer
make install
```

Setup `add_word %GDWORD%` to goldendict-ng's program dictionary: 
`add_word` will insert every word to sqlite


## How to review

1. Start goldendict-ng


2. `~/.config/awesome/rc.lua`
```
{
    rule = { instance = "goldendict"},
    properties = {
        width = 1900,
        height = 800,
        floating = true,
        titlebars_enabled = false,
        requests_no_titlebar = true,
        x = 10,
        y = 200,
        focus = false,
        border_width = 0,
        ontop = true,
    }
}
```

3. `review`


## Note

1. Use this `goldendict_wrapper` to fix history

```
#!/usr/bin/env bash

sqlite3 ~/.local/share/goldendict/history.db 'SELECT word FROM fsrs ORDER BY rowid DESC' | awk  '{print "0 " $0}' > ~/.local/share/goldendict/history
goldendict
sqlite3 ~/.local/share/goldendict/history.db 'SELECT word FROM fsrs ORDER BY rowid DESC' | awk  '{print "0 " $0}' > ~/.local/share/goldendict/history
```

## Q&A

#### Why not anki

anki-connect require anki window open: quite annoying


#### Can we use on other desktop envrionment ?

As long as the desktop manager support configuration of goldendict-ng's position, height and width

