# Tauri + Vanilla TS

This template should help get you started developing with Tauri in vanilla HTML, CSS and Typescript.

## dbus example

```bash
gdbus call --session \
      --dest org.o324.gui \
      --object-path /org/o324/gui \
      --method org.o324.gui.NotifyTaskChange \
      '("Delete", <("test",)>)'
```
