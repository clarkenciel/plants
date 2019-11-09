use chrono::{DateTime, Duration, Utc};
use rand::{self, seq::SliceRandom, Rng};
use std::cell::RefCell;
use std::cmp;
use std::fmt::{self, Display};
use std::io;
use std::str;

fn main() -> io::Result<()> {
    let words = vec!["hello", "sally", "maye"];

    let mut rng = rand::thread_rng();

    let mut plants: Vec<Plant> = words
        .iter()
        .map(|word| plant_from_word(&mut rng, word))
        .collect();

    let breeder = RandomBreeder::new(rand::thread_rng());
    while plants.len() > 1 {
        println!("{:-^repeat$}", "-", repeat=10);
        for (i, plant) in plants.iter().enumerate() {
            println!("{}: {}", i, plant);
        }

        let new_plants = rng.gen_range(0, 3);
        for _ in 0..new_plants {
            let parents: Vec<&Plant> = plants.as_slice().choose_multiple(&mut rng, 2).collect();
            let new_plant = breeder.breed(parents[0], parents[1]);
            plants.push(new_plant);
        }

        let kill_plants = rng.gen_range(0, cmp::min(3, plants.len()));
        for _ in 0..kill_plants {
            let tot_plants = plants.len();
            let kill_idx = rng.gen_range(0, tot_plants);
            plants.remove(kill_idx);
        }
    }

    Ok(())
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
    expression: u8,
    expiration: DateTime<Utc>,
}

impl Display for Plant {
    fn fmt(&self, fmtr: &mut fmt::Formatter) -> fmt::Result {
        let expr = [self.expression];
        let printable_expr = str::from_utf8(&expr).unwrap();
        write!(fmtr, "{}: {}", self.dna, printable_expr)
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
    let offset = rng.gen_range(1, 500);
    let duration = Duration::milliseconds(offset);
    *dt + duration
}

fn select_expression<R: Rng>(rng: &mut R, dna: &Dna) -> u8 {
    dna.0.as_bytes().choose(rng).unwrap().clone()
}
