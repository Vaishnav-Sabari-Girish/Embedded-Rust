use ratatui::{
    Terminal,
    layout::{Alignment, Constraint, Layout},
    style::Style,
    widgets::{Block, Paragraph},
};

const LOGO: &str = r#"
 ██▒   █▓  ██████    ▄████ 
▓██░   █▒▒██    ▒ ▒ ██▒ ▀█▒
 ▓██  █▒░░ ▓██▄   ░▒██░▄▄▄░
  ▒██ █░░  ▒   ██▒░░▓█  ██▓
   ▒▀█░  ▒██████▒▒░▒▓███▀▒░
   ░ ▐░  ▒ ▒▓▒ ▒ ░ ░▒   ▒  
   ░ ░░  ░ ░▒  ░    ░   ░  
     ░░  ░  ░  ░  ░ ░   ░ ░
      ░        ░        ░  
      "#;

pub struct Startup;

impl Startup {
    pub fn start<B>(terminal: &mut Terminal<B>)
    where
        B: ratatui::backend::Backend,
    {
        terminal
            .draw(|frame| {
                let epaper_theme = Style::default()
                    .fg(ratatui::style::Color::Black)
                    .bg(ratatui::style::Color::White);

                let block_text = Block::default().style(epaper_theme);
                frame.render_widget(block_text, frame.area());

                let layout = Layout::vertical([
                    Constraint::Percentage(30),
                    Constraint::Min(10),
                    Constraint::Percentage(20),
                ])
                .split(frame.area());

                let logo_widget = Paragraph::new(LOGO)
                    .style(Style::default())
                    .alignment(Alignment::Center);

                frame.render_widget(logo_widget, layout[1]);
            })
            .unwrap();
    }
}
