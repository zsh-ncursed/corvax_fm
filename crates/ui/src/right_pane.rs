use ratatui::{
    prelude::{Rect, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use rtfm_core::app_state::TabState;

pub fn render_right_pane(frame: &mut Frame, area: Rect, tab_state: &TabState) {
    let preview_content = match &tab_state.preview_content {
        Some(content) => content.clone(),
        None => "No item selected".to_string(),
    };

    let block = Block::default().borders(Borders::ALL).title("Preview");
    let inner_area = block.inner(area);

    let paragraph = Paragraph::new(preview_content)
        .style(Style::default())
        .block(Block::default())
        .scroll(tab_state.preview_scroll);

    frame.render_widget(block, area);
    frame.render_widget(paragraph, inner_area);
}
