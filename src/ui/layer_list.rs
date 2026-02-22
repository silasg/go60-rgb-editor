use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use go60_rgb_editor_domain::Layer;

pub struct LayerListWidget<'a> {
    layers: &'a [Layer],
    selected: usize,
}

impl<'a> LayerListWidget<'a> {
    pub fn new(layers: &'a [Layer], selected: usize) -> Self {
        Self { layers, selected }
    }
}

impl<'a> Widget for LayerListWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Layers [J/K] ")
            .borders(Borders::ALL);

        let items: Vec<ListItem> = self
            .layers
            .iter()
            .enumerate()
            .map(|(i, layer)| {
                let prefix = if i == self.selected { "▶ " } else { "  " };
                let style = if i == self.selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!("{}{}", prefix, layer.name)).style(style)
            })
            .collect();

        let list = List::new(items).block(block);
        Widget::render(list, area, buf);
    }
}
