extern crate termion;

use std::io::Write;
use std::sync::mpsc::{channel, Receiver};
use termion::input::{TermRead, MouseTerminal};
use termion::raw::IntoRawMode;
use termion::event::{Event, Key, MouseEvent};

enum SimulationEvent {
    QUIT,
    PLAYPAUSE,
    DRAW(u16, u16)
}

struct Simulation {
    running: bool,
    term_width: u16,
    term_height: u16,
    cells: Vec<Vec<Cell>>,
    input_rx: Receiver<SimulationEvent>
}

impl Simulation {
    fn new(input_rx: Receiver<SimulationEvent>) -> Self {
        let (width, height) = termion::terminal_size().unwrap();
        let mut cells = Vec::with_capacity(height as usize);
        for _ in 0..height {
            cells.push(Vec::with_capacity(width as usize));
        }

        for i in 0..height as usize {
            for _ in 0..width as usize {
                cells[i].push(Cell { old_state: CellState::DEAD, state: CellState::DEAD });
            }
        }

        Simulation {
            running: false,
            term_width: width,
            term_height: height,
            cells: cells,
            input_rx: input_rx
        }
    }

    fn run(&mut self) {
        loop {
            if self.running {
                self.tick();
            }

            for event in self.input_rx.try_iter() {
                match event {
                    SimulationEvent::QUIT => return,
                    SimulationEvent::PLAYPAUSE => self.running = !self.running,
                    SimulationEvent::DRAW(x, y) => {
                        if self.cells[(y-1) as usize][(x-1) as usize].state == CellState::DEAD {
                            self.cells[(y-1) as usize][(x-1) as usize].state = CellState::ALIVE;
                            print!("{}{}", termion::cursor::Goto(x, y), 'o');
                        } else {
                            self.cells[(y-1) as usize][(x-1) as usize].state = CellState::DEAD;
                            print!("{}{}", termion::cursor::Goto(x, y), ' ');
                        }
                    }
                }
            }
        }
    }

    fn tick(&mut self) {
        for i in 0..self.term_height as usize {
            for j in 0..self.term_width as usize {
                self.cells[i][j].old_state = self.cells[i][j].state;
            }
        }

        for i in 0..self.term_height as usize {
            for j in 0..self.term_width as usize {
                let mut neighbors = 0;

                if i != 0 && j != 0 && self.cells[i-1][j-1].old_state == CellState::ALIVE {
                    neighbors += 1;
                }
                if i != 0 && self.cells[i-1][j].old_state == CellState::ALIVE {
                    neighbors += 1;
                }
                if i != 0 && j != self.term_width as usize - 1 && self.cells[i-1][j+1].old_state == CellState::ALIVE {
                    neighbors += 1;
                }
                if j != 0 && self.cells[i][j-1].old_state == CellState::ALIVE {
                    neighbors += 1;
                }
                if j != self.term_width as usize - 1 && self.cells[i][j+1].old_state == CellState::ALIVE {
                    neighbors += 1;
                }
                if i != self.term_height as usize - 1 && j != 0 && self.cells[i+1][j-1].old_state == CellState::ALIVE {
                    neighbors += 1;
                }
                if i != self.term_height as usize - 1 && self.cells[i+1][j].old_state == CellState::ALIVE {
                    neighbors += 1;
                }
                if i != self.term_height as usize - 1 && j != self.term_width as usize - 1 && self.cells[i+1][j+1].old_state == CellState::ALIVE {
                    neighbors += 1;
                }

                if self.cells[i][j].state == CellState::ALIVE && neighbors < 2 { // die
                    self.cells[i][j].state = CellState::DEAD;
                    print!("{}{}", termion::cursor::Goto((j+1) as u16, (i+1) as u16), ' ');
                } else if self.cells[i][j].state == CellState::ALIVE && neighbors > 3 { // die
                    self.cells[i][j].state = CellState::DEAD;
                    print!("{}{}", termion::cursor::Goto((j+1) as u16, (i+1) as u16), ' ');
                } else if self.cells[i][j].state == CellState::DEAD && neighbors == 3 { // live
                    self.cells[i][j].state = CellState::ALIVE;
                    print!("{}{}", termion::cursor::Goto((j+1) as u16, (i+1) as u16), 'o');
                }
            }
        }

        std::io::stdout().flush().unwrap();
    }
}

#[derive(PartialEq, Copy, Clone)]
enum CellState {
    ALIVE,
    DEAD
}

struct Cell {
    old_state: CellState,
    state: CellState
}

fn main() {
    let mut stdout = MouseTerminal::from(std::io::stdout().into_raw_mode().unwrap());

    write!(stdout, "{}{}{}",
           termion::screen::ToAlternateScreen,
           termion::clear::All,
           termion::cursor::Hide).unwrap();
    stdout.flush().unwrap();

    let (event_tx, event_rx) = channel();
    let mut simulation = Simulation::new(event_rx);

    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for event in stdin.events() {
            let event = event.unwrap();
            match event {
                Event::Key(Key::Char(' ')) => { // toggle simulation pause/resume
                    event_tx.send(SimulationEvent::PLAYPAUSE).unwrap();
                }

                Event::Mouse(MouseEvent::Press(_, x, y)) |
                Event::Mouse(MouseEvent::Hold(x, y)) => {
                    event_tx.send(SimulationEvent::DRAW(x, y)).unwrap();
                }

                Event::Key(Key::Char('q')) |
                Event::Key(Key::Esc) => {
                    event_tx.send(SimulationEvent::QUIT).unwrap();
                    break;
                }

                _ => {}
            }

            stdout.flush().unwrap();
        }
    });

    simulation.run();

    print!("{}{}",
           termion::screen::ToMainScreen,
           termion::cursor::Show);
}
