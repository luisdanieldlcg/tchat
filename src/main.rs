use std::net::TcpListener;

/* use std::io;


use crossterm::{
    event::{EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
 */
fn main()  {
    app_init();
    let server = TcpListener::bind("127.0.0.1:8194").unwrap();
}

fn app_init()  {
   /*  enable_raw_mode().unwrap();
    let mut stdout =  io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend);
    disable_raw_mode().unwrap(); */
}
