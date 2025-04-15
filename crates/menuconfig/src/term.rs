use std::{collections::HashMap, str::FromStr};

use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};

use crate::config::{Config, Target};

#[derive(Debug, Clone)]
enum ConfigValue {
    Bool(bool),
    String(String),
    List(Vec<ConfigValue>),
}

impl ConfigValue {
    fn as_bool(&self) -> bool {
        match self {
            ConfigValue::Bool(b) => *b,
            _ => panic!("Not a bool"),
        }
    }

    fn as_string(&self) -> String {
        match self {
            ConfigValue::String(s) => s.clone(),
            _ => panic!("Not a string"),
        }
    }

    fn as_list(&self) -> Vec<ConfigValue> {
        match self {
            ConfigValue::List(l) => l.clone(),
            _ => panic!("Not a list"),
        }
    }
}

trait ConfigItem {
    fn get_value(&self) -> ConfigValue;
    fn get_height(&self) -> u16;
    fn render(&self, f: &mut Frame, area: Rect, selected: bool);
    fn on_event(&mut self, event: KeyEvent) -> bool;
    fn on_deselect(&mut self) {}
}

struct ConfigToggle {
    name: String,
    value: bool,
}

impl ConfigItem for ConfigToggle {
    fn get_value(&self) -> ConfigValue {
        ConfigValue::Bool(self.value)
    }

    fn get_height(&self) -> u16 {
        2
    }

    fn render(&self, f: &mut Frame, area: Rect, selected: bool) {
        let block = Block::default().borders(Borders::BOTTOM);
        f.render_widget(&block, area);
        let content = Paragraph::new(format!(
            "{} {}",
            if self.value { "[x]" } else { "[ ]" },
            self.name.as_str()
        ))
        .style(if selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
        f.render_widget(content, block.inner(area));
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

#[derive(Debug)]
struct ConfigChoice<T: Clone + std::fmt::Display> {
    name: String,
    selected: usize,
    choices: Vec<T>,
}

impl<T> ConfigChoice<T>
where
    T: Clone + std::fmt::Display,
{
    pub fn new(name: String, choices: Vec<T>, selected: usize) -> Self {
        Self {
            name,
            selected,
            choices,
        }
    }
}

impl<T> ConfigItem for ConfigChoice<T>
where
    T: Clone + std::fmt::Display,
{
    fn get_value(&self) -> ConfigValue {
        ConfigValue::String(self.choices[self.selected].to_string())
    }

    fn get_height(&self) -> u16 {
        2
    }

    fn render(&self, f: &mut Frame, area: Rect, selected: bool) {
        let block = Block::default().borders(Borders::BOTTOM);
        f.render_widget(&block, area);
        let content = Paragraph::new(format!(
            "[{}] {}",
            self.choices[self.selected].to_string(),
            self.name.as_str()
        ))
        .style(if selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
        f.render_widget(content, block.inner(area));
    }

    fn on_event(&mut self, event: KeyEvent) -> bool {
        if event.code == KeyCode::Right {
            self.selected = (self.selected + 1) % self.choices.len();
            true
        } else if event.code == KeyCode::Left {
            self.selected = (self.selected + self.choices.len() - 1) % self.choices.len();
            true
        } else {
            false
        }
    }
}

struct ConfigSection {
    name: String,
    items: Vec<Box<dyn ConfigItem>>,
    selected: Option<usize>,
}

impl ConfigSection {
    fn add_item(&mut self, item: Box<dyn ConfigItem>) {
        self.items.push(item);
    }
}

impl ConfigItem for ConfigSection {
    fn get_value(&self) -> ConfigValue {
        ConfigValue::List(self.items.iter().map(|item| item.get_value()).collect())
    }

    fn get_height(&self) -> u16 {
        if self.selected.is_some() { 0 } else { 3 }
    }

    fn render(&self, f: &mut Frame, area: Rect, selected: bool) {
        let block = Block::default().borders(Borders::ALL);
        f.render_widget(&block, area);
        if let Some(selected) = self.selected {
            let mut constraints = Vec::new();
            for item in self.items.iter() {
                let height = item.get_height();
                if height == 0 {
                    // This means that the item will instead take up the entire space
                    item.render(f, area, true);
                    return;
                }
                constraints.push(Constraint::Length(height));
            }
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(block.inner(area));
            for (i, item) in self.items.iter().enumerate() {
                item.render(f, layout[i], selected == i);
            }
        } else {
            let content = Paragraph::new(self.name.as_str()).style(if selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });
            f.render_widget(content, block.inner(area));
        }
    }

    fn on_event(&mut self, event: KeyEvent) -> bool {
        // Capture input if we are selected
        if let Some(selected) = self.selected {
            if self
                .items
                .get_mut(selected)
                .is_some_and(|item| item.as_mut().on_event(event))
            {
                return true;
            } else if event.code == KeyCode::Esc {
                self.selected = None;
                return true;
            } else if event.code == KeyCode::Up {
                self.selected.replace(if selected == 0 {
                    self.items.len() - 1
                } else {
                    selected - 1
                });
                return true;
            } else if event.code == KeyCode::Down {
                self.selected.replace(if selected == self.items.len() - 1 {
                    0
                } else {
                    selected + 1
                });
                return true;
            }
        }

        if event.code == KeyCode::Enter {
            self.selected = Some(self.selected.unwrap_or(0));
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
struct ConfigMenu {
    item_ids: HashMap<&'static str, usize>,
    items: Vec<Box<dyn ConfigItem>>,
    selected: usize,
}

impl ConfigMenu {
    fn add_item(&mut self, name: &'static str, item: Box<dyn ConfigItem>) {
        self.item_ids.insert(name, self.items.len());
        self.items.push(item);
    }

    fn get_item(&self, name: &'static str) -> Option<&Box<dyn ConfigItem>> {
        self.items.get(*self.item_ids.get(name)?)
    }

    fn on_event(&mut self, event: KeyEvent) -> bool {
        if self
            .items
            .get_mut(self.selected)
            .is_some_and(|item| item.as_mut().on_event(event))
        {
            return true;
        }

        if event.code == KeyCode::Up {
            self.items[self.selected].on_deselect();
            self.selected = if self.selected == 0 {
                self.items.len() - 1
            } else {
                self.selected - 1
            };
            true
        } else if event.code == KeyCode::Down {
            self.items[self.selected].on_deselect();
            self.selected = (self.selected + 1) % self.items.len();
            true
        } else if event.code == KeyCode::Enter {
            if let Some(item) = self.items.get_mut(self.selected) {
                item.on_event(event);
            }
            true
        } else {
            false
        }
    }

    fn render(&self, f: &mut Frame, area: Rect) {
        let mut constraints = Vec::new();
        for item in self.items.iter() {
            let height = item.get_height();
            if height == 0 {
                // This means that the item will instead take up the entire space
                item.render(f, area, true);
                return;
            }
            constraints.push(Constraint::Length(height));
        }
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);
        for (i, item) in self.items.iter().enumerate() {
            item.render(f, layout[i], i == self.selected);
        }
    }
}

pub fn run(config: Config) -> Result<Config, Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();

    let mut menu = ConfigMenu::default();

    let mut boot_section = ConfigSection {
        name: "Boot Options".to_string(),
        items: Vec::new(),
        selected: None,
    };
    let targets = vec![Target::X86_64, Target::AArch64];
    let selected_target = targets.iter().position(|t| t == &config.target).unwrap_or(0);
    boot_section.add_item(Box::new(ConfigChoice::new(
        "Target".to_string(),
        targets,
        selected_target,
    )));

    menu.add_item("boot", Box::new(boot_section));

    menu.add_item(
        "debug",
        Box::new(ConfigToggle {
            name: "Enable debug".to_string(),
            value: config.debug,
        }),
    );
    menu.add_item(
        "serial",
        Box::new(ConfigToggle {
            name: "Enable serial".to_string(),
            value: config.serial,
        }),
    );
    menu.add_item(
        "backtrace",
        Box::new(ConfigToggle {
            name: "Enable backtrace".to_string(),
            value: config.backtrace,
        }),
    );
    menu.add_item(
        "smp",
        Box::new(ConfigToggle {
            name: "Enable SMP".to_string(),
            value: config.smp,
        }),
    );

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
            if !menu.on_event(key) {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }
    }
    ratatui::restore();

    let boot_options = menu.items[0].get_value().as_list();

    Ok(Config {
        target: Target::from_str(&boot_options[0].as_string()).unwrap(),
        debug: menu.get_item("debug").unwrap().get_value().as_bool(),
        serial: menu.get_item("serial").unwrap().get_value().as_bool(),
        backtrace: menu.get_item("backtrace").unwrap().get_value().as_bool(),
        smp: menu.get_item("smp").unwrap().get_value().as_bool(),
    })
}
