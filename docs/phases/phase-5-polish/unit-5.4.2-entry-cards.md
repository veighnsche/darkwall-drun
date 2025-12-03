# Unit 5.4.2: Entry Cards

> **Parent:** Unit 5.4 (Theming)  
> **Complexity:** Medium  
> **Skills:** Ratatui widgets, text styling

---

## Objective

Render each entry as a multi-line card showing icon, name, generic name, comment, and categories.

---

## Target Appearance

```
┌─────────────────────────────┐
│ 🦊 Firefox                  │  ← Line 1: Icon + Name (bold)
│    Web Browser              │  ← Line 2: GenericName
│    Browse the web           │  ← Line 3: Comment (dimmed)
│    Network;WebBrowser       │  ← Line 4: Categories (more dimmed)
└─────────────────────────────┘
```

---

## Entry Card Structure

### Height: 4-5 lines

| Line | Content | Style |
|------|---------|-------|
| 1 | Icon + Name | Bold, primary foreground |
| 2 | GenericName | Normal, primary foreground |
| 3 | Comment | Dimmed (`theme.dimmed`) |
| 4 | Categories | More dimmed (`theme.dimmed_alt`) |

### Width

- Fills available column width
- Text truncated with `…` if too long

---

## Implementation

### EntryCard Widget

```rust
pub struct EntryCard<'a> {
    entry: &'a Entry,
    selected: bool,
    theme: &'a Theme,
    config: &'a EntryDisplayConfig,
}

impl<'a> Widget for EntryCard<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bg = if self.selected {
            self.theme.selection_bg
        } else {
            self.theme.background
        };

        // Fill background
        buf.set_style(area, Style::default().bg(bg));

        let inner = area.inner(Margin::new(1, 0));
        let mut y = inner.y;

        // Line 1: Icon + Name
        let name_line = format!("{} {}", self.entry.icon_char(), self.entry.name);
        let name_style = Style::default()
            .fg(self.theme.foreground)
            .add_modifier(Modifier::BOLD);
        buf.set_string(inner.x, y, truncate(&name_line, inner.width), name_style);
        y += 1;

        // Line 2: GenericName
        if self.config.show_generic {
            if let Some(generic) = &self.entry.generic_name {
                let style = Style::default().fg(self.theme.foreground);
                buf.set_string(inner.x + 3, y, truncate(generic, inner.width - 3), style);
            }
            y += 1;
        }

        // Line 3: Comment
        if self.config.show_comment {
            if let Some(comment) = &self.entry.comment {
                let style = Style::default().fg(self.theme.dimmed);
                buf.set_string(inner.x + 3, y, truncate(comment, inner.width - 3), style);
            }
            y += 1;
        }

        // Line 4: Categories
        if self.config.show_categories && !self.entry.categories.is_empty() {
            let cats = self.entry.categories.join(";");
            let style = Style::default().fg(self.theme.dimmed_alt);
            buf.set_string(inner.x + 3, y, truncate(&cats, inner.width - 3), style);
        }
    }
}
```

### Text Truncation

```rust
fn truncate(s: &str, max_width: u16) -> Cow<str> {
    let width = s.width();
    if width <= max_width as usize {
        Cow::Borrowed(s)
    } else {
        let mut result = String::new();
        let mut current_width = 0;
        for c in s.chars() {
            let char_width = c.width().unwrap_or(0);
            if current_width + char_width + 1 > max_width as usize {
                result.push('…');
                break;
            }
            result.push(c);
            current_width += char_width;
        }
        Cow::Owned(result)
    }
}
```

---

## Configuration

```toml
[appearance.entry]
show_generic = true      # Show GenericName line
show_comment = true      # Show Comment line
show_categories = true   # Show Categories line
```

When all are disabled, entry is 1 line (icon + name only).

---

## Selection Highlight

| State | Background | Border |
|-------|------------|--------|
| Normal | `theme.background` | None |
| Selected | `theme.selection_bg` | Optional accent border |

---

## Acceptance Criteria

- [ ] 4-line card renders correctly
- [ ] Icon displays (Nerd Font glyph)
- [ ] Name is bold
- [ ] Comment/categories are dimmed
- [ ] Long text truncates with `…`
- [ ] Selection changes background
- [ ] Missing fields handled (skip line)
