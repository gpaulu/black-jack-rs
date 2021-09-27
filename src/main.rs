#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::VecDeque;
use std::io::stdin;
use std::sync::{Arc, Mutex};

use legion::world::SubWorld;
use legion::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Suit {
    Heart,
    Diamond,
    Spade,
    Club,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Value {
    Num(u8),
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Face {
    Up,
    Down,
}

// #[derive(Clone, Copy, Debug, PartialEq)]
// enum Position {
//     Deck,
//     Discard,
//     Hand(u8),
// }

#[derive(Clone, Debug, PartialEq)]
struct Player {
    score: u32,
    id: u8,
    name: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Hand(Vec<Entity>);

type Card = (Suit, Value, Face);

fn gen_deck_of_cards() -> Vec<Card> {
    let mut deck = Vec::with_capacity(52);
    fn insert_cards_of_suit(deck: &mut Vec<Card>, suit: Suit) {
        let make_card = |suit: Suit, value: Value| (suit, value, Face::Up);
        deck.push(make_card(suit, Value::Ace));
        for i in 2..=10 {
            deck.push(make_card(suit, Value::Num(i)));
        }
        deck.push(make_card(suit, Value::Jack));
        deck.push(make_card(suit, Value::Queen));
        deck.push(make_card(suit, Value::King));
    }
    insert_cards_of_suit(&mut deck, Suit::Heart);
    insert_cards_of_suit(&mut deck, Suit::Diamond);
    insert_cards_of_suit(&mut deck, Suit::Spade);
    insert_cards_of_suit(&mut deck, Suit::Club);
    deck
}

// fn deal(world: &mut World, deck: &mut Vec<Entity>, to: Entity) {
//     let player = world
//         .entry_mut(to)
//         .expect("Player Exists")
//         .get_component_mut::<Player>()
//         .expect("Player has Player component");

//     if deck.is_empty() {
//         todo!()
//     }
//     let card = deck.pop().expect("deck is not empty");
//     let card = world
//         .entry_mut(card)
//         .expect("Card exists")
//         .get_component_mut::<(Position,)>()
//         .expect("Card has Position");
// }

#[derive(Clone, Debug, PartialEq, Default)]
struct Deck(Vec<Entity>);

#[system(for_each)]
#[filter(component::<Suit>())]
fn put_in_deck(entity: &Entity, #[resource] deck: &mut Deck) {
    // *pos = Position::Deck;
    deck.0.push(*entity);
}

#[system]
fn shuffle_deck(#[resource] deck: &mut Deck, #[resource] rng: &mut ChaCha8Rng) {
    deck.0.shuffle(rng);
}

fn pop_deck(deck: &mut Deck) -> Entity {
    if deck.0.is_empty() {
        todo!();
    }
    deck.0.pop().expect("Deck is not empty")
}

fn deal1(hand: &mut Hand, deck: &mut Deck) {
    let card = pop_deck(deck);
    hand.0.push(card);
    // let mut card = world.entry_mut(card).expect("Card exists");
    // println!("card has {:?}", card.archetype().layout().component_types());
    // let position = card
    //     .get_component_mut::<Position>()
    //     .expect("Card has Position");
    // *position = Position::Hand(player.id);
    // let position = card.get_component::<Position>().expect("Card has Position");
    // println!("Position is {:?}", position);
}

#[system(for_each)]
#[write_component(Face)]
fn deal(player: &Player, hand: &mut Hand, world: &mut SubWorld, #[resource] deck: &mut Deck) {
    if player.name == "Dealer" {
        let card_entity = pop_deck(deck);
        let mut card = world.entry_mut(card_entity).expect("Card exists");
        let face = card.get_component_mut::<Face>().expect("Card has Face");
        *face = Face::Down;
        hand.0.push(card_entity);
    } else {
        deal1(hand, deck);
    }
    deal1(hand, deck);
}

#[system(for_each)]
#[read_component(Suit)]
#[read_component(Value)]
#[read_component(Face)]
fn display_cards(player: &Player, hand: &Hand, world: &SubWorld) {
    println!("player {}", player.name);
    let mut has_any_face_down = false;
    for entity in &hand.0 {
        let card = world.entry_ref(*entity).expect("Card exists");
        let face = card.get_component::<Face>().expect("Card has Face");
        match face {
            Face::Up => {
                print!(
                    "{:?} of {:?}, ", //TODO: impl Display
                    card.get_component::<Value>().expect("Card has Value"),
                    card.get_component::<Suit>().expect("Card has Suit")
                );
            }
            Face::Down => {
                has_any_face_down = true;
                print!("???, ")
            }
        }
    }
    if has_any_face_down {
        println!("Score: ???");
    } else {
        println!("Score: {}", player.score);
    }
}

#[system(for_each)]
fn action(
    player: &Player,
    hand: &mut Hand,
    #[resource] deck: &mut Deck,
    #[resource] decision_queue: &Arc<Mutex<VecDeque<Decision>>>,
) {
    if player.name == "Dealer" {
        return;
    }
    let mut decision_queue = decision_queue.lock().unwrap();
    if decision_queue.is_empty() {
        return;
    }
    let decision = decision_queue.pop_front().unwrap();
    match decision {
        Decision::Hit => deal1(hand, deck),
        Decision::Hold => unreachable!(),
    }
}

#[system(for_each)]
#[read_component(Suit)]
#[read_component(Value)]
fn score(player: &mut Player, hand: &Hand, world: &SubWorld) {
    // let aces = hand.iter().filter(|entity| if let )
    let card_values = hand.0.iter().map(|entity| {
        let card = world.entry_ref(*entity).expect("Card exists");
        card.into_component::<Value>().expect("Card has Value")
    });
    let aces = card_values
        .clone()
        .filter(|card| matches!(card, Value::Ace))
        .count();
    let score: u32 = card_values
        .clone()
        .filter_map(|card| match card {
            Value::Num(n) => Some(*n as u32),
            Value::Jack => Some(10),
            Value::Queen => Some(10),
            Value::King => Some(10),
            Value::Ace => None,
        })
        .sum();
    let mut possible_scores = vec![score];
    for _ in 0..aces {
        let mut new_scores = Vec::new();
        for i in &possible_scores {
            new_scores.push(i + 1);
            new_scores.push(i + 11);
        }
        possible_scores = new_scores;
    }
    let score = possible_scores
        .iter()
        .filter(|scr| **scr <= 21)
        .max()
        .unwrap_or_else(|| possible_scores.iter().min().expect("A score must exist"));
    player.score = *score;
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Decision {
    Hit,
    Hold,
}

#[allow(clippy::needless_return)] // return in match arm is an early return and makes sense to keep
fn player_decision() -> Decision {
    println!("Choose: 1) Hit, 2) Hold");
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).expect("Read input");
    let res = match buffer.trim_end() {
        "" => return player_decision(),
        response => response,
    };
    let num = match res.parse::<u8>() {
        Err(_) => return player_decision(),
        Ok(num) => num,
    };
    match num {
        1 => Decision::Hit,
        2 => Decision::Hold,
        _ => return player_decision(),
    }
}

fn main() {
    let mut rng = thread_rng();
    let mut world = World::default();
    let mut cards: Vec<_> = world.extend(gen_deck_of_cards()).iter().collect();
    cards.shuffle(&mut rng);
    let dealer = world.push((
        Player {
            score: 0,
            id: 0,
            name: String::from("Dealer"),
        },
        Hand(Vec::new()),
    ));
    let player = world.push((
        Player {
            score: 0,
            id: 1,
            name: String::from("Player"),
        },
        Hand(Vec::new()),
    ));

    let mut query = <&Player>::query();

    for player in query.iter(&world) {
        if player.id == 0 {
            continue;
        }
        println!("Hello, {}", player.name);
    }

    // you define a query be declaring what components you want to find, and how you will access them
    let mut query = <(&Suit, &Value)>::query();

    // you can then iterate through the components found in the world
    for position in query.iter(&world) {
        println!("{:?}", position);
    }

    let mut schedule = Schedule::builder()
        .add_system(put_in_deck_system())
        .add_system(shuffle_deck_system())
        .add_system(deal_system())
        .build();

    let mut gameplay_loop = Schedule::builder()
        .add_system(action_system())
        .add_system(score_system())
        .add_system(display_cards_system())
        .build();

    let mut resources = Resources::default();
    resources.insert(Deck::default());
    resources.insert(rand_chacha::ChaCha8Rng::seed_from_u64(10));
    let decision_queue = Arc::new(Mutex::new(VecDeque::new()));
    resources.insert(decision_queue.clone());

    schedule.execute(&mut world, &mut resources);
    loop {
        gameplay_loop.execute(&mut world, &mut resources);
        let decision = player_decision();
        if let Decision::Hold = decision {
            break;
        }
        decision_queue
            .lock()
            .expect("lock failed")
            .push_back(decision);
    }

    // let mut query = <(&Suit, &Value, &Position)>::query();

    // // you can then iterate through the components found in the world
    // for position in query.iter(&world) {
    //     println!("{:?}", position);
    // }
}
