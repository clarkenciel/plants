use chrono::{DateTime, Duration, Utc};
use rand::{self, seq::SliceRandom, Rng};
use std::cell::RefCell;
use std::cmp;
use std::fmt::{self, Display};
use std::fs;
use std::io::{self, prelude::*};
use std::str;
use std::thread;
use std::time;

fn main() -> io::Result<()> {
    let words = read_words()?;
    let mut rng = rand::thread_rng();

    let num_plants = 50;
    let mut plants: Vec<Plant> = words
    let mut plants: Vec<Plant<rand::rngs::ThreadRng>> = words
        .as_slice()
        .choose_multiple(&mut rng, num_plants)
        .map(|word| plant_from_word(&mut rng, word))
        .collect();

    let breeder = RandomBreeder::new(rand::thread_rng());
    while plants.len() > 1 {
        let now = Utc::now();
        println!("{now:-^width$}", now = format!("{}", now), width = 70);
        for plant in plants.iter() {
            print!("{}", plant.expression);
        }
        println!();

        let new_plants = rng.gen_range(0, num_plants / 2);
        for _ in 0..new_plants {
            let new_plant = if *[true, false].choose(&mut rng).unwrap() {
                let parents: Vec<&Plant<rand::rngs::ThreadRng>> =
                    plants.as_slice().choose_multiple(&mut rng, 2).collect();
                breeder.breed(parents[0], parents[1])
            } else {
                let mut children = plants
                    .as_slice()
                    .choose(&mut rng)
                    .unwrap()
                    .expand();
                let child_idx = rng.gen_range(0, children.len());
                children.remove(child_idx)
            };
            plants.push(new_plant);
        }

        plants.retain(|plant| !plant.is_dead(&now));
        thread::sleep(time::Duration::from_millis(500));
    }

    Ok(())
}

fn read_words() -> io::Result<Vec<String>> {
    let mut file = fs::File::open("/usr/share/dict/usa")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.split("\n").map(|s| s.to_owned()).collect())
}

fn plant_from_word<R: Rng + Clone>(rng: &mut R, word: &str) -> Plant<R> {
    let dna = Dna(word.to_owned());
    let expiration = random_date_after(rng, &Utc::now());
    let expression = select_expression(rng, &dna);
    Plant::new(dna, expiration, expression, rng.clone())
}

struct Dna(String);

impl Dna {
    fn combine(&self, other: &Self) -> Self {
        Dna(self.0.clone() + &other.0)
    }
}

impl Display for Dna {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(fmtr, "{}", self.0)
    }
}

struct Plant<R>
where
    R: Rng,
{
    dna: Dna,
    expression: String,
    expiration: DateTime<Utc>,
    rng: RefCell<R>,
}

impl<R: Rng + Clone> Plant<R> {
    fn new(dna: Dna, expiration: DateTime<Utc>, expression: String, rng: R) -> Self {
        Plant {
            dna,
            expiration,
            expression,
            rng: RefCell::new(rng),
        }
    }

    fn is_dead(&self, time: &DateTime<Utc>) -> bool {
        self.expiration <= *time
    }

    fn expand(&self) -> Vec<Self> {
        let mut rng = self.rng.borrow_mut();
        (0..4)
            .map(|_| plant_from_word(&mut *rng, &*self.dna.0))
            .collect()
    }
}

impl<R: Rng> Display for Plant<R> {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmtr,
            "<Plant phe={phe}, exp={exp}, dna={dna}>",
            dna = self.dna,
            phe = self.expression,
            exp = self.expiration
        )
    }
}

trait Breeder<T> {
    fn breed(&self, a: &T, b: &T) -> T;
}

struct RandomBreeder<R>
where
    R: Rng,
{
    rng: RefCell<R>,
}

impl<R: Rng> RandomBreeder<R> {
    fn new(rng: R) -> Self {
        Self {
            rng: RefCell::new(rng),
        }
    }
}

impl<PR: Rng + Clone, R: Rng> Breeder<Plant<PR>> for RandomBreeder<R> {
    fn breed(&self, a: &Plant<PR>, b: &Plant<PR>) -> Plant<PR> {
        let dna = a.dna.combine(&b.dna);
        let mut rng = self.rng.borrow_mut();
        let expression = select_expression(&mut *rng, &dna);
        let expiration = random_date_after(&mut *rng, cmp::min(&a.expiration, &b.expiration));
        Plant::new(dna, expiration, expression, (*a.rng.borrow()).clone())
    }
}

fn random_date_after<R: Rng>(rng: &mut R, dt: &DateTime<Utc>) -> DateTime<Utc> {
    let offset = rng.gen_range(1, 5000);
    let duration = Duration::milliseconds(offset);
    *dt + duration
}

fn select_expression<R: Rng>(rng: &mut R, dna: &Dna) -> String {
    let c = dna.0.as_bytes().choose(rng).unwrap();
    String::from_utf8(vec![*c]).unwrap()
}
