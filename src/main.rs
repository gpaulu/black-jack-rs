#![allow(dead_code)]
#![allow(unused_variables)]

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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Position {
    Deck,
    Discard,
    Hand(u8),
}

#[derive(Clone, Debug, PartialEq)]
struct Player {
    score: u8,
    id: u8,
    name: String,
}

type Card = (Suit, Value, Position);

fn gen_deck_of_cards() -> Vec<Card> {
    let mut deck = Vec::with_capacity(52);
    fn insert_cards_of_suit(deck: &mut Vec<Card>, suit: Suit) {
        deck.push((suit, Value::Ace, Position::Discard));
        for i in 2..=10 {
            deck.push((suit, Value::Num(i), Position::Discard));
        }
        deck.push((suit, Value::Jack, Position::Discard));
        deck.push((suit, Value::Queen, Position::Discard));
        deck.push((suit, Value::King, Position::Discard));
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
fn put_in_deck(pos: &mut Position, entity: &Entity, #[resource] deck: &mut Deck) {
    *pos = Position::Deck;
    deck.0.push(*entity);
}

#[system]
fn shuffle_deck(#[resource] deck: &mut Deck, #[resource] rng: &mut ChaCha8Rng) {
    deck.0.shuffle(rng);
}

#[system(for_each)]
#[write_component(Position)]
fn deal(player: &Player, entity: &Entity, world: &mut SubWorld, #[resource] deck: &mut Deck) {
    println!("Hello, deal");
    let mut deal1 = || {
        if deck.0.is_empty() {
            todo!();
        }
        let card = deck.0.pop().expect("Deck is not empty");
        let mut card = world.entry_mut(card).expect("Card exists");
        println!("card has {:?}", card.archetype().layout().component_types());
        let position = card
            .get_component_mut::<Position>()
            .expect("Card has Position");
        *position = Position::Hand(player.id);
        // let position = card.get_component::<Position>().expect("Card has Position");
        // println!("Position is {:?}", position);
    };
    deal1();
    deal1();
}

fn main() {
    let mut rng = thread_rng();
    let mut world = World::default();
    let mut cards: Vec<_> = world.extend(gen_deck_of_cards()).iter().collect();
    cards.shuffle(&mut rng);
    let dealer = world.push((Player {
        score: 0,
        id: 0,
        name: String::from("Dealer"),
    },));
    let player = world.push((Player {
        score: 0,
        id: 1,
        name: String::from("Player"),
    },));

    let mut query = <&Player>::query();

    for player in query.iter(&world) {
        if player.id == 0 {
            continue;
        }
        println!("Hello, {}", player.name);
    }

    // you define a query be declaring what components you want to find, and how you will access them
    let mut query = <(&Suit, &Value, &Position)>::query();

    // you can then iterate through the components found in the world
    for position in query.iter(&world) {
        println!("{:?}", position);
    }

    let mut schedule = Schedule::builder()
        .add_system(put_in_deck_system())
        .add_system(shuffle_deck_system())
        .add_system(deal_system())
        .build();
    let mut resources = Resources::default();
    resources.insert(Deck::default());
    resources.insert(rand_chacha::ChaCha8Rng::seed_from_u64(10));
    schedule.execute(&mut world, &mut resources);

    let mut query = <(&Suit, &Value, &Position)>::query();

    // you can then iterate through the components found in the world
    for position in query.iter(&world) {
        println!("{:?}", position);
    }
}
