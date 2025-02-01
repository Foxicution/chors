# Chors - Task Manager

Chors is a simple yet powerful terminal-based task manager designed for
efficiency and flexibility. With support for nested tasks, filtering, and a
built-in calendar view, Chors helps you stay on top of your tasks directly from
the terminal.

# To run:
Windows (can't put into chors folder yet):
```powershell
cargo run -- -f "~\.config\model.json"
```

Linux:
```bash
cargo run -- -f "~/.config/chors/model.json"
```

# TODO for v0.1.0:
- [ ] Add a help (?) command overview and a reference that's auto generated
- [ ] Make pasting instant
- [ ] Filter view overlay editing
- [ ] View saving, editing
- [ ] Group operations in views
- [ ] Calendar view working fully (add tasks, edit tasks, view tasks, change times of tasks, navigate the calendar)
- [ ] Mobile support
