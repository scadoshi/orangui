```text
:..:::                   .
:.:::::           .-====-:-:.  .
::....         .:-=+==+==++====-.
:::           :=======-==++++=====-.
  .          -=+=-+#*#=#######++==+=:
            :===+*#############+==+==-
  ..  :-**##*==+#+##*##########*+**##*=:
  . .*##***=++:.   .=*+=:    :**+==***#*=.
   .*#*##*=::                  ..-#*#**###-:
   =##=:--:.-.       :-       ..--=#*+*###+=:
..-###+..:::..... :=-#+=-.....:..:.:..:####===:
:=*###-.:::.....=*##**####*=-::.. ::..:####====:
+==++=:::-:. ..*####*#######*==-. ..:::####+====
++==....==. ..=##############*==-. ....:-+====-=
+===:..:=-....*#**+**#****##**=-=.... ...======-
====...===. .-#*##*===+*##**#*==-.  . ..:=======
===:..:=-:   .:+###########**=:-.  .....:=======
+=-:..-:..   ...-+*#####*+=:..... ... ..:=======
+==.. ... ..  .  ....:.... ..  .....  ..-=======
==:. ...   ...    .  ...  ..  ..::...  .-=======
=:.. ...  .. ...    .       ..-===..  ..--======
-:..  .  ..:-:.....   ......-=====:.  ..::======
:..... ...-===--::......:=-=======:.. ....:=====
..     .:===========-=============--..  ...:--==
.    ..-============================-..   .....-
      :==============================....  .....
```

# orangui

A Dioxus UI kit with the themes, panels, nav, charts and entrance effects my apps and sites share. The name is orangutan, a call sign of mine, plus GUI.

## Hosts

- [zwipe](https://github.com/scadoshi/zwipe), the app and zwipe.net
- [portfolio](https://github.com/scadoshi/portfolio), scottyfermo.com, where the [orangui page](https://scottyfermo.com/projects/orangui/) shows the components live
- [cairn](https://github.com/scadoshi/cairn), a counter app

## Use

```toml
[dependencies]
orangui = { git = "https://github.com/scadoshi/orangui" }
```

Add `features = ["toast"]` for the toast stack. The full list of components is in `src/lib.rs`.

Inline the stylesheets in this order:

- `THEMES_CSS`
- `COMPONENTS_CSS`
- `APP_CSS` for an app, or `SITE_CSS` for a website

The host's own sheet can load before or after these. The rules the kit keeps and the tests that hold them are in [context/](context/).
