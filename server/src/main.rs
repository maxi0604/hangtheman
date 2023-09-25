use rand::prelude::*;
use std::collections::BTreeSet;
use std::default::Default;
use std::env;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};

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
    "C++",
    "Latenz",
    "Durchsatz",
    "Bitrate",
    "UNIX",
    "Befehlssatzarchitektur",
    "Numerik",
    "Gleichung",
    "netcat",
    "TCP",
    "UDP",
    "Frontend",
    "Backend",
    "RISC-V",
    "Forward-Search"
];

fn main() {
    let snd_arg = env::args().nth(1);
    let port = snd_arg
        .and_then(|arg| arg.parse::<u16>().ok())
        .unwrap_or(1337);
    println!("Hello, world!");
    let n_players = env::args().nth(2)
        .and_then(|arg| arg.parse::<usize>().ok())
        .unwrap_or(2);
    let mut player_conns = accept_n_connections(n_players, port);
    loop {
        let mut session = GameSession::generate(10);
        player_conns.iter_mut().for_each(|(_, w)| {
            w.write_all("Eine neue Runde beginnt.\n".as_bytes());
            w.flush();
        });
        println!("Geheimer Debug-Output. Das Wort ist \"{}\"", session.word());
        let result = 'outer: loop {
            for (r, w) in player_conns.iter_mut() {
                w.write_all(
                    format!(
                        "Aktueller Stand: {}. Versuche: {}/{}. Bereits geraten:{}\n",
                        session.underscore_string(),
                        session.fails(),
                        session.max_fails(),
                        session.guessed_letters()
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
                    session.guess_word(response)
                };

                if result != GuessResult::Continue {
                    break 'outer result;
                }

                w.write_all(
                    format!(
                        "Aktueller Stand: {}. Versuche: {}/{}. Bereits geraten:{}\n",
                        session.underscore_string(),
                        session.fails(),
                        session.max_fails(),
                        session.guessed_letters()
                    )
                    .as_bytes(),
                );
                w.flush();
            }
        };
        let message = match result {
            GuessResult::Won => format!("\nGewonnen. Das Wort war {}.\n\n", session.word()),
            GuessResult::Lost => {
                format!("\nVerloren. Skill Issue. Das Wort war {}.\n\n", session.word())
            }
            GuessResult::Continue => unreachable!(),
        };

        player_conns.iter_mut().for_each(|(_, w)| {
            w.write_all(message.as_bytes());
            w.flush();
        });
    }
}

fn accept_n_connections(n: usize, port: u16) -> Vec<(BufReader<TcpStream>, BufWriter<TcpStream>)> {
    let bind_addr = format!("[::]:{port}");
    let listener = TcpListener::bind(bind_addr.as_str())
        .unwrap_or_else(|_| panic!("Can't bind to {}", bind_addr.as_str()));
    let mut player_conns: Vec<_> = vec![];

    while player_conns.len() < n {
        match listener.accept() {
            Ok((stream, addr)) => {
                let Ok(rstream) = stream.try_clone() else {
                    continue;
                };
                let Ok(wstream) = stream.try_clone() else {
                    continue;
                };
                let reader = BufReader::new(rstream);
                let mut writer = BufWriter::new(wstream);
                writer.write_all(format!("Vielen Dank, dass Sie sich für HangTheMan {} entschieden haben.\n", env!("CARGO_PKG_VERSION")).as_bytes());
                writer.write_all("Wir wünschen Ihnen einen angenehmen Aufenthalt.\n".as_bytes());
                writer.write_all(format!("Es sind bereits {}/{} Spieler*innen verbunden.\n\n", player_conns.len() + 1, n).as_bytes());
                writer.flush();
                println!("Neue Verbindung von {}", addr);
                player_conns.push((reader, writer));
            }
            Err(err) => { eprintln!("{}", err); continue; },
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
        if s.to_lowercase() == self.word.to_lowercase() {
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
                if self.guessed_chars.contains(&c.to_lowercase().to_string()) || !c.is_alphanumeric() {
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
    
    pub fn guessed_letters(&self) -> String {
        self.guessed_chars.iter().fold(String::new(), |acc, c| acc + " " + c)
    }
}
