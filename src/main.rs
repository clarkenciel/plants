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

    let num_plants = 10;
    let mut plants: Vec<Plant> = words
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

        let new_plants = rng.gen_range(0, 3);
        for _ in 0..new_plants {
            let parents: Vec<&Plant> = plants.as_slice().choose_multiple(&mut rng, 2).collect();
            let new_plant = breeder.breed(parents[0], parents[1]);
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

fn plant_from_word<R: Rng>(rng: &mut R, word: &str) -> Plant {
    let dna = Dna(word.to_owned());
    let expiration = random_date_after(rng, &Utc::now());
    let expression = select_expression(rng, &dna);
    Plant {
        dna,
        expiration,
        expression,
    }
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

struct Plant {
    dna: Dna,
    expression: String,
    expiration: DateTime<Utc>,
}

impl Plant {
    fn is_dead(&self, time: &DateTime<Utc>) -> bool {
        self.expiration <= *time
    }
}

impl Display for Plant {
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

impl<R: Rng> Breeder<Plant> for RandomBreeder<R> {
    fn breed(&self, a: &Plant, b: &Plant) -> Plant {
        let dna = a.dna.combine(&b.dna);
        let mut rng = self.rng.borrow_mut();
        let expression = select_expression(&mut *rng, &dna);
        let expiration = random_date_after(&mut *rng, cmp::min(&a.expiration, &b.expiration));
        Plant {
            dna,
            expression,
            expiration,
        }
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
