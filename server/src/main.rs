use rand::prelude::*;
use std::collections::BTreeSet;
use std::default::Default;
use std::env;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

const WORDS: &[&str] = &[
    "Socket",
    "Oktober",
    "Client",
    "Rechnerkommunikation",
    "Server",
    "China",
    "Pipeline",
    "Rechner",
    "Mikroarchitektur",
    "Krabbe",
    "Taschenratte",
    "Keith",
    "C++",
];

fn main() {
    let snd_arg = env::args().nth(1);
    let port = snd_arg
        .and_then(|arg| arg.parse::<u16>().ok())
        .unwrap_or(1337);
    println!("Hello, world!");
    let n_players = 2;
    let player_conns = accept_n_connections(n_players, port);
    let mut readers_writers = player_conns
        .iter()
        .map(|(stream, _)| {
            (
                BufReader::new(stream.clone()),
                BufWriter::new(stream.clone()),
            )
        })
        .collect::<Vec<_>>();
    loop {
        let mut session = GameSession::generate(10);
        println!("Geheimer Debug-Output. Das Wort ist \"{}\"", session.word());
        let result = 'outer: loop {
            for (r, w) in readers_writers.iter_mut() {
                w.write_all(
                    format!(
                        "Aktueller Stand: {}. Versuche: {}/{}\n",
                        session.underscore_string(),
                        session.fails(),
                        session.max_fails()
                    )
                    .as_bytes(),
                );
                w.write_all("Rate einen Buchstaben oder ein Wort.\n".as_bytes());
                w.flush();

                let mut response = String::new();
                r.read_line(&mut response);
                let response = response.trim();
                println!("Spieler hat {} geraten", response);

                let result = if response.chars().count() == 1 {
                    session.guess_char(response.chars().next().unwrap())
                } else {
                    session.guess_word(&response)
                };

                if result != GuessResult::Continue {
                    break 'outer result;
                }

                w.write_all(
                    format!(
                        "Aktueller Stand: {}. Versuche: {}/{}\n",
                        session.underscore_string(),
                        session.fails(),
                        session.max_fails()
                    )
                    .as_bytes(),
                );
                w.flush();
            }
        };
        let message = match result {
            GuessResult::Won => format!("Gewonnen. Das Wort war {}.\n", session.word()),
            GuessResult::Lost => {
                format!("Verloren. Skill Issue. Das Wort war {}.\n", session.word())
            }
            GuessResult::Continue => unreachable!(),
        };

        readers_writers.iter_mut().for_each(|(_, w)| {
            w.write_all(message.as_bytes());
        });
    }
}

fn accept_n_connections(n: usize, port: u16) -> Vec<(TcpStream, SocketAddr)> {
    let bind_addr = format!("[::]:{port}");
    let listener = TcpListener::bind(bind_addr.as_str())
        .expect(format!("Can't bind to {}", bind_addr.as_str()).as_str());
    let mut player_conns: Vec<_> = vec![];

    while player_conns.len() < n {
        match listener.accept() {
            Ok(conn) => player_conns.push(conn),
            Err(err) => eprintln!("{}", err),
        }
    }
    player_conns
}

#[derive(PartialEq)]
enum GuessResult {
    Won,
    Lost,
    Continue,
}

#[derive(Debug)]
struct GameSession<'a> {
    word: &'a str,
    fails: u64,
    max_fails: u64,
    guessed_chars: BTreeSet<String>,
}

impl GameSession<'_> {
    pub fn generate(max_fails: u64) -> Self {
        let word = WORDS
            .choose(&mut rand::thread_rng())
            .expect("No words in list.");
        GameSession {
            word,
            fails: 0,
            max_fails,
            guessed_chars: Default::default(),
        }
    }

    pub fn guess_char(&mut self, c: char) -> GuessResult {
        self.guessed_chars.insert(c.to_lowercase().to_string());
        if self.word.chars().any(|i| i == c) {
            return GuessResult::Continue;
        } else {
            self.fails += 1;
        }

        self.check_loss()
    }

    pub fn guess_word(&mut self, s: &str) -> GuessResult {
        if s == self.word {
            return GuessResult::Won;
        } else {
            self.fails += 1;
        }

        self.check_loss()
    }

    fn check_loss(&self) -> GuessResult {
        if self.fails >= self.max_fails {
            GuessResult::Lost
        } else {
            GuessResult::Continue
        }
    }

    pub fn underscore_string(&self) -> String {
        self.word
            .chars()
            .map(|c| {
                if self.guessed_chars.contains(&c.to_lowercase().to_string()) {
                    c
                } else {
                    '_'
                }
            })
            .collect()
    }

    pub fn fails(&self) -> u64 {
        self.fails
    }

    pub fn max_fails(&self) -> u64 {
        self.max_fails
    }

    pub fn word(&self) -> &str {
        self.word
    }
}
