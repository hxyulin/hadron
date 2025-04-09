use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

use crate::config::Config;

enum ConfigValue {
    Bool(bool),
}

impl ConfigValue {
    fn as_bool(&self) -> bool {
        match self {
            ConfigValue::Bool(b) => *b,
        }
    }
}

trait ConfigItem {
    fn get_value(&self) -> String;
    // TODO: Rename this to get_value, and use a render approach instead
    fn get_value_type(&self) -> ConfigValue;
    fn on_event(&mut self, event: KeyEvent) -> bool;
}

struct ConfigToggle {
    name: String,
    value: bool,
}

impl ConfigItem for ConfigToggle {
    fn get_value(&self) -> String {
        format!("{} {}", if self.value { "[x]" } else { "[ ]" }, self.name)
    }

    fn get_value_type(&self) -> ConfigValue {
        ConfigValue::Bool(self.value)
    }

    fn on_event(&mut self, event: KeyEvent) -> bool {
        if event.code == KeyCode::Enter {
            self.value = !self.value;
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
struct ConfigMenu {
    items: Vec<Box<dyn ConfigItem>>,
    selected: usize,
}

impl ConfigMenu {
    fn add_item(&mut self, item: Box<dyn ConfigItem>) {
        self.items.push(item);
    }

    fn on_event(&mut self, event: KeyEvent) -> bool {
        if event.code == KeyCode::Up {
            self.selected = if self.selected == 0 {
                self.items.len() - 1
            } else {
                self.selected - 1
            };
            true
        } else if event.code == KeyCode::Down {
            self.selected = (self.selected + 1) % self.items.len();
            true
        } else if event.code == KeyCode::Enter {
            if let Some(item) = self.items.get_mut(self.selected) {
                item.on_event(event);
            }
            true
        } else {
            self.items
                .get_mut(self.selected)
                .is_some_and(|item| item.as_mut().on_event(event))
        }
    }

    fn render(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let mut style = Style::default();
                if idx == self.selected {
                    style.fg = Some(Color::Yellow);
                }
                ListItem::new(item.get_value()).style(style)
            })
            .collect();
        let list = List::new(items).block(Block::default());
        f.render_widget(list, area);
    }
}

pub fn run() -> Result<Config, Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();

    let mut menu = ConfigMenu::default();
    menu.add_item(Box::new(ConfigToggle {
        name: "Enable debug".to_string(),
        value: true,
    }));
    menu.add_item(Box::new(ConfigToggle {
        name: "Enable SMP".to_string(),
        value: false,
    }));

    loop {
        terminal.draw(|f| {
            // Step 1: Define the help menu content
            let help_items_left = ["↑/↓: Navigate", "q/esc: Quit"];
            let help_items_right = ["Enter: Select", "Space: Details"];
            let help_menu_height = help_items_left.len().max(help_items_right.len()) as u16;

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(help_menu_height)])
                .split(f.area());

            let config_block = Block::default().title("Config").borders(Borders::ALL);

            let help_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(layout[1]);

            f.render_widget(
                List::new(help_items_left.iter().map(|s| ListItem::new(*s)))
                    .block(Block::default().borders(Borders::NONE)),
                help_layout[0],
            );
            f.render_widget(
                List::new(help_items_right.iter().map(|s| ListItem::new(*s)))
                    .block(Block::default().borders(Borders::NONE)),
                help_layout[1],
            );

            menu.render(f, config_block.inner(layout[0]));
            f.render_widget(config_block, layout[0]);
        })?;

        if let Event::Key(key) = crossterm::event::read()? {
            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                break;
            }
            menu.on_event(key);
        }
    }
    ratatui::restore();
    Ok(Config {
        debug: menu.items[0].get_value_type().as_bool(),
        smp: menu.items[1].get_value_type().as_bool(),
    })
}
