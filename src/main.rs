mod constants;

use clap::Parser;
use constants::*;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, size},
};
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about = "SL (Steam Locomotive) runs across your terminal.", long_about = None)]
struct Args {
    #[arg(short = 'a', long)]
    accident: bool,

    #[arg(short = 'F', long)]
    fly: bool,

    #[arg(short = 'l', long)]
    logo: bool,

    #[arg(short = 'c', long)]
    c51: bool,
}

struct Smoke {
    y: i32,
    x: i32,
    ptrn: usize,
    kind: usize,
}

struct App {
    args: Args,
    smokes: Vec<Smoke>,
    smoke_sum: usize,
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), Show);
        let _ = disable_raw_mode();
    }
}

impl App {
    fn new(args: Args) -> Self {
        Self {
            args,
            smokes: Vec::new(),
            smoke_sum: 0,
        }
    }

    fn add_smoke(&mut self, y: i32, x: i32) {
        if x % 4 == 0 {
            for s in &mut self.smokes {
                s.y -= DY[s.ptrn];
                s.x += DX[s.ptrn];
                if s.ptrn < SMOKE_PATTERNS - 1 {
                    s.ptrn += 1;
                }
            }
            self.smokes.push(Smoke {
                y,
                x,
                ptrn: 0,
                kind: self.smoke_sum % 2,
            });
            self.smoke_sum += 1;
        }
    }

    fn draw_str_queued(&self, out: &mut std::io::Stdout, y: i32, x: i32, s: &str, cols: u16, lines: u16) {
        if y < 0 || y >= lines as i32 { return; }
        let s_len = s.len() as i32;
        if x <= -s_len || x >= cols as i32 { return; }

        let start = x.max(0);
        let mut end = (x + s_len).min(cols as i32);
        
        // Prevent scrolling by not drawing at the bottom-right character
        if y == lines as i32 - 1 && end == cols as i32 {
            end = cols as i32 - 1;
        }

        if start < end {
            let offset = (start - x) as usize;
            let len = (end - start) as usize;
            let sub = &s[offset..offset+len];
            let _ = queue!(out, MoveTo(start as u16, y as u16), Print(sub));
        }
    }

    fn draw_man_queued(&self, out: &mut std::io::Stdout, y: i32, x: i32, cols: u16, lines: u16) {
        for i in 0..2 {
            let man_idx = ((LOGOLENGTH as i32 + x).abs() / 12 % 2) as usize;
            self.draw_str_queued(out, y + i as i32, x, MAN[man_idx][i], cols, lines);
        }
    }

    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Ignore SIGINT like the original
        unsafe {
            let _ = signal_hook::low_level::register(signal_hook::consts::SIGINT, || {});
        }

        enable_raw_mode()?;
        let _guard = TerminalGuard;
        let mut out = stdout();
        execute!(out, Hide, Clear(ClearType::All))?;

        let (mut cols, mut lines) = size()?;
        let _ = lines;
        let mut x = cols as i32 - 1;

        let length = if self.args.logo {
            LOGOLENGTH as i32
        } else if self.args.c51 {
            C51LENGTH as i32
        } else {
            D51LENGTH as i32
        };

        loop {
            if x < -length {
                break;
            }

            queue!(out, Clear(ClearType::All))?;

            let (new_cols, new_lines) = size()?;
            cols = new_cols;
            lines = new_lines;

            let mut y = lines as i32 / 2 - 5;
            let mut dy = 0;

            if self.args.fly {
                let slope = if self.args.logo { 6 } else { 7 };
                y = (x / slope) + lines as i32 - (cols as i32 / slope) - D51HEIGHT as i32;
                dy = 1;
            }

            if self.args.logo {
                let mut y_fly = lines as i32 / 2 - 3;
                let (mut py1, mut py2, mut py3) = (0, 0, 0);
                if self.args.fly {
                    y_fly = (x / 6) + lines as i32 - (cols as i32 / 6) - LOGOHEIGHT as i32;
                    py1 = 2; py2 = 4; py3 = 6;
                }
                
                let pattern = ((LOGOLENGTH as i32 + x).abs() / 3 % LOGOPATTERNS as i32) as usize;
                for i in 0..=LOGOHEIGHT {
                    let sl_str = match i {
                        0 => LOGO1, 1 => LOGO2, 2 => LOGO3, 3 => LOGO4,
                        4 => LWHL[pattern][0], 5 => LWHL[pattern][1],
                        _ => DELLN,
                    };
                    self.draw_str_queued(&mut out, y_fly + i as i32, x, sl_str, cols, lines);
                    self.draw_str_queued(&mut out, y_fly + i as i32 + py1, x + 21, LCOAL[i], cols, lines);
                    self.draw_str_queued(&mut out, y_fly + i as i32 + py2, x + 42, LCAR[i], cols, lines);
                    self.draw_str_queued(&mut out, y_fly + i as i32 + py3, x + 63, LCAR[i], cols, lines);
                }

                if self.args.accident {
                    self.draw_man_queued(&mut out, y_fly + 1, x + 14, cols, lines);
                    self.draw_man_queued(&mut out, y_fly + 1 + py2, x + 45, cols, lines);
                    self.draw_man_queued(&mut out, y_fly + 1 + py2, x + 53, cols, lines);
                    self.draw_man_queued(&mut out, y_fly + 1 + py3, x + 66, cols, lines);
                    self.draw_man_queued(&mut out, y_fly + 1 + py3, x + 74, cols, lines);
                }
                self.add_smoke(y_fly - 1, x + LOGOFUNNEL as i32);

            } else if self.args.c51 {
                let pattern = ((C51LENGTH as i32 + x).abs() % C51PATTERNS as i32) as usize;
                for i in 0..=C51HEIGHT {
                    let c51_str = if i < 7 { C51STR[i] } else if i < 11 { C51WH[pattern][i-7] } else { C51DEL };
                    self.draw_str_queued(&mut out, y + i as i32, x, c51_str, cols, lines);
                    let coal_idx = if i == 0 || i == 11 { 10 } else { i - 1 };
                    self.draw_str_queued(&mut out, y + i as i32 + dy, x + 55, COAL[coal_idx], cols, lines);
                }
                if self.args.accident {
                    self.draw_man_queued(&mut out, y + 3, x + 45, cols, lines);
                    self.draw_man_queued(&mut out, y + 3, x + 49, cols, lines);
                }
                self.add_smoke(y - 1, x + C51FUNNEL as i32);
            } else {
                let pattern = ((D51LENGTH as i32 + x).abs() % D51PATTERNS as i32) as usize;
                for i in 0..=D51HEIGHT {
                    let d51_line = match i {
                        0 => D51STR1, 1 => D51STR2, 2 => D51STR3, 3 => D51STR4,
                        4 => D51STR5, 5 => D51STR6, 6 => D51STR7,
                        7 => D51WHL[pattern][0], 8 => D51WHL[pattern][1], 9 => D51WHL[pattern][2],
                        _ => D51DEL,
                    };
                    self.draw_str_queued(&mut out, y + i as i32, x, d51_line, cols, lines);
                    self.draw_str_queued(&mut out, y + i as i32 + dy, x + 53, COAL[i], cols, lines);
                }
                if self.args.accident {
                    self.draw_man_queued(&mut out, y + 2, x + 43, cols, lines);
                    self.draw_man_queued(&mut out, y + 2, x + 47, cols, lines);
                }
                self.add_smoke(y - 1, x + D51FUNNEL as i32);
            }

            for s in &self.smokes {
                self.draw_str_queued(&mut out, s.y, s.x, SMOKE[s.kind][s.ptrn], cols, lines);
            }

            out.flush()?;
            sleep(Duration::from_millis(40));
            x -= 1;
            
            self.smokes.retain(|s| s.ptrn < SMOKE_PATTERNS - 1);
        }

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut app = App::new(args);
    app.run()
}
