#![warn(clippy::all, clippy::pedantic)]

use std::io::stdin;

#[derive(Debug)]
struct Visitor {
    name: String,
    action: VisitorAction,
    age: i8,
}

#[derive(Debug)]
enum VisitorAction {
    Accept,
    AcceptWithNote { note: String },
    Refuse,
    Probation,
}

impl Visitor {
    fn new(name: &str, action: VisitorAction, age: i8) -> Self {
        Self {
            name: name.to_lowercase(),
            action,
            age,
        }
    }

    fn greet(&self) {
        match &self.action {
            VisitorAction::Accept => println!("welcome, {}", self.name),
            VisitorAction::AcceptWithNote { note } => {
                println!("welcome, {}", self.name);
                println!("note: {}", note);
                if self.age < 21 {
                    println!("no alcohol for {}", self.name);
                }
            }
            VisitorAction::Probation => println!("{} is a probationary member now...", self.name),
            VisitorAction::Refuse => println!("Not allowed, {}, *kicks chest*", self.name),
        }
    }
}

fn retrieve_name() -> String {
    let mut user_name = String::new();
    stdin()
        .read_line(&mut user_name)
        .expect("Failed to read line");
    user_name.trim().to_lowercase()
}

fn main() {
    let mut visitors = vec![
        Visitor::new(
            "aaron",
            VisitorAction::AcceptWithNote {
                note: String::from("he's my brother I have to"),
            },
            20,
        ),
        Visitor::new("jasmine", VisitorAction::Accept, 22),
        Visitor::new("nate", VisitorAction::Accept, 22),
        Visitor::new("jerald", VisitorAction::Refuse, 79),
    ];

    loop {
        println!("name. or i'm kicking you off this tree:");
        let name = retrieve_name();

        let known_visitor = visitors.iter().find(|visitor| visitor.name == name);
        match known_visitor {
            Some(visitor) => visitor.greet(),
            None => {
                if name.is_empty() {
                    break;
                } else {
                    println!(
                        "you're not on the list, {} *kicks chest*, come back next time",
                        name
                    );
                    visitors.push(Visitor::new(&name, VisitorAction::Probation, 0));
                }
            }
        }
    }
    println!("visitors:");
    println!("{:#?}", visitors);
}
