#![allow(dead_code)]
#![allow(unused_variables)]

use legion::*;

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

type Card = (Suit, Value);

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

fn main() {
    let mut world = World::default();
    world.extend(gen_deck_of_cards());

    // you define a query be declaring what components you want to find, and how you will access them
    let mut query = <(&Suit, &Value)>::query();

    // you can then iterate through the components found in the world
    for position in query.iter(&world) {
        println!("{:?}", position);
    }
}
