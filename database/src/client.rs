use serde::de::DeserializeOwned;
use std::sync::Mutex;
use futures::stream::iter_ok;
use tokio::timer::Interval;
use std::time::Instant;
use std::io::{BufReader,BufRead};
use std::str::FromStr;
use std::fs::File;

use std::time::Duration;
use rand::distributions::*;
use rand::prelude::*;

use std::sync::RwLock;
use crate::buffer_pool::ClockBuffer;
use std::sync::Arc;
use tokio::prelude::*;

use crate::future_signal::BufferedSignal;


pub enum Amount {
	Limited (u64),
	Unlimited,
}

pub enum RunPeriod {
	Finite (Duration),
	Indefinite,
}

pub enum Frequency {
	Immediate,
	Delayed(Interval),
}


pub struct Client<T,U> 
	where T: Stream<Item=U,Error=()>
{
	producer: T,
	amount: Amount,
	run_period: RunPeriod,
	frequency: Frequency,
	start: Instant,
	produced: Option<u64>,
}

pub fn client_from_stream<T,U>(producer: T, amount: Amount, run_period: RunPeriod,
			   frequency: Frequency)
			    -> impl Stream<Item=U,Error=()>
	where T: Stream<Item=U,Error=()>,
{
	let produced = match amount {
		Amount::Limited(_) => Some(0),
		Amount::Unlimited  => None,
	};

	Client { 
		producer: producer,
		amount: amount,
		run_period: run_period,
		frequency: frequency,
		start: Instant::now(),
		produced: produced,
	}
}

pub fn client_from_iter<T,U>(producer: T, amount: Amount, run_period: RunPeriod,
				   frequency: Frequency)
				    -> impl Stream<Item=U,Error=()>
	where T: Iterator<Item=U>
{
	let produced = match amount {
		Amount::Limited(_) => Some(0),
		Amount::Unlimited  => None,
	};

	Client { 
		producer:iter_ok(producer),
		amount: amount,
		run_period: run_period,
		frequency: frequency,
		start: Instant::now(),
		produced: produced,
	}
}


impl<T,U> Stream for Client<T,U> 
	where T: Stream<Item=U,Error=()>
{
	type Item = U;
	type Error = ();

	fn poll(&mut self) -> Poll<Option<U>,()> {
		
		/* Terminate stream if hit time-limit */
		if let RunPeriod::Finite(dur) = self.run_period {
			let now = Instant::now();
			let time = now.duration_since(self.start); 
			if time >= dur { return Ok(Async::Ready(None)) }
		}

		/* Terminate stream if hit max production */
		if let Amount::Limited(max_items) = self.amount {
			if let Some(items) = self.produced {
				if items >= max_items { return Ok(Async::Ready(None)) }
			}
		}

		/* Either poll to determine if enough time has passed or
		 * immediately get the value depending on Frequency Mode
		 * Must call poll on the stream within the client
		 */
		match &mut self.frequency {
			Frequency::Immediate => {
				let poll_val = try_ready!(self.producer.poll());
				Ok(Async::Ready(poll_val))
			}
			Frequency::Delayed(interval) => {
				match interval.poll() {
					Ok(Async::NotReady) => Ok(Async::NotReady),
					Err(e) => { 
						println!("{:?}", e); 
						Err(())
					}
					_ =>  {
						let poll_val = try_ready!(self.producer.poll());
						Ok(Async::Ready(poll_val))
					}
				}
				
			}
		}

	}
}

fn construct_file_iterator<T>(file: &str, delim: u8) -> Result<impl Iterator<Item=T>,()> 
	where T: DeserializeOwned
{
	let f = match File::open(file) {
		Ok(f) => f,
		Err(_) => return Err(()),
	};

	Ok(BufReader::new(f)
		.split(delim)
		.filter_map(|x| match x {
			Ok(val) => bincode::deserialize(&val).ok(),
			_ => None
		})
	)
}

/* Must use type annotation on function to declare what to 
 * parse CSV entries as 
 */
fn construct_file_iterator_skip_newline<T>(file: &str, skip_val: usize, delim: char) -> Result<impl Iterator<Item=T>,()> 
	where T: FromStr
{
	let f = match File::open(file) {
		Ok(f) => f,
		Err(_) => return Err(()),
	};

	Ok(BufReader::new(f)
		.lines()
		.filter_map(Result::ok)
		.flat_map(move |line: String| {
			line.split(delim)
				.skip(skip_val)
				.filter_map(|item: &str| item.parse::<T>().ok())
				.collect::<Vec<T>>()
				.into_iter()
		})
	)
}


pub fn construct_file_client<T>(file: &str, delim: u8, amount: Amount, 
						 run_period: RunPeriod, frequency: Frequency)
						 -> Result<impl Stream<Item=T,Error=()>,()> 
	where T: DeserializeOwned
{
	let producer = construct_file_iterator::<T>(file, delim)?;
	Ok(client_from_iter(producer, amount, run_period, frequency))
}

/* An example of how to combine an iterator and IterClient constructor */
pub fn construct_file_client_skip_newline<T>(file: &str, skip_val: usize, delim: char, amount: Amount, run_period: RunPeriod,
				   		 frequency: Frequency)
				    	 -> Result<impl Stream<Item=T,Error=()>,()>
	where T: FromStr,
{
	let producer = construct_file_iterator_skip_newline::<T>(file, skip_val, delim)?;
	Ok(client_from_iter(producer, amount, run_period, frequency))
}


pub fn construct_gen_client<'a,T,U:'a,R>(dist: &'a T, rng: &'a mut R, 
		amount: Amount, run_period: RunPeriod, frequency: Frequency) 
			-> impl Stream<Item=U,Error=()> + 'a
		where R: Rng,
		      T: Distribution<U>

{
	let producer = rng.sample_iter(dist);
	client_from_iter(producer, amount, run_period, frequency)
}

#[test]
fn construct_client() {
	let mut db_opts = rocksdb::Options::default();
	db_opts.create_if_missing(true);
	let fm = match rocksdb::DB::open(&db_opts, "../rocksdb") {
		Ok(x) => x,
		Err(e) => panic!("Failed to create database: {:?}", e),
	};

	let client = construct_file_client_skip_newline::<f32>("../UCRArchive2018/Ham/Ham_TEST", 1, ',', Amount::Unlimited, RunPeriod::Indefinite, Frequency::Immediate).unwrap();
	let buffer: Arc<Mutex<ClockBuffer<f32,rocksdb::DB>>>  = Arc::new(Mutex::new(ClockBuffer::new(500,fm)));
	let _sig1 = BufferedSignal::new(1, client, 400, buffer.clone(),|i,j| i >= j, |_| (), false);

	let dist = Normal::new(0.0,1.0);
	let mut rng = thread_rng();
	let _std_norm_client = construct_gen_client(&dist, &mut rng, Amount::Unlimited, RunPeriod::Indefinite, Frequency::Immediate);

	let _ = rocksdb::DB::destroy(&db_opts, "../rocksdb");
}