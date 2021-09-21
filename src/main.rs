#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::VecDeque;
use std::io::stdin;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};

use legion::world::SubWorld;
use legion::*;
use rand::prelude::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::{Rng, SeedableRng};
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

// #[derive(Clone, Copy, Debug, PartialEq)]
// enum Position {
//     Deck,
//     Discard,
//     Hand(u8),
// }

#[derive(Clone, Debug, PartialEq)]
struct Player {
    score: u8,
    id: u8,
    name: String,
}

#[derive(Clone, Debug, PartialEq)]
struct Hand(Vec<Entity>);

type Card = (Suit, Value /*, Position*/);

fn gen_deck_of_cards() -> Vec<Card> {
    let mut deck = Vec::with_capacity(52);
    fn insert_cards_of_suit(deck: &mut Vec<Card>, suit: Suit) {
        deck.push((suit, Value::Ace));
        for i in 2..=10 {
            deck.push((suit, Value::Num(i)));
        }
        deck.push((suit, Value::Jack));
        deck.push((suit, Value::Queen));
        deck.push((suit, Value::King));
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

fn deal1(player: &Player, hand: &mut Hand, deck: &mut Deck) {
    if deck.0.is_empty() {
        todo!();
    }
    let card = deck.0.pop().expect("Deck is not empty");
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
// #[write_component(Position)]
fn deal(player: &Player, hand: &mut Hand, #[resource] deck: &mut Deck) {
    deal1(player, hand, deck);
    deal1(player, hand, deck);
}

#[system(for_each)]
#[read_component(Suit)]
#[read_component(Value)]
fn display_cards(player: &Player, hand: &Hand, world: &SubWorld) {
    println!("player {}", player.name);
    for entity in &hand.0 {
        let card = world.entry_ref(*entity).expect("Card exists");
        print!(
            "{:?} of {:?}, ", //TODO: impl Display
            card.get_component::<Value>().expect("Card has Value"),
            card.get_component::<Suit>().expect("Card has Suit")
        );
    }
    println!();
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
        Decision::Hit => deal1(player, hand, deck),
        Decision::Hold => unreachable!(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Decision {
    Hit,
    Hold,
}

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
