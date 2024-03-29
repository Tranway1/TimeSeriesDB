pub mod compress;
pub mod deep_learning;
pub mod gorilla_encoder;
pub mod gorilla_decoder;
pub mod bit_packing;
pub mod fcm_encoder;
pub mod prec_double;
pub mod parquet;
use std::fmt;
use crate::dictionary::{DictionaryId};

/*
 * Overview:
 * This is the API for compression methods using
 * The goal is to construct a general framework that
 * each compression method must implement. Furthermore,
 * this file acts as a hub for each of the compression methods.
 *
 * Design Choice:
 * The two main goals were to construct
 * 1. A struct to hold all of the compression methods. Where
 *    each method is a struct wrapped in an option. The method
 *    struct should contain everything, besides the dictionary,
 *    needed to compress.
 *	  Note: This implementation unless optimized (not sure) will be
 *    space expensive. I assumed the number of dictionaries as well
 *    as the parameters in each method would be relatively small.
 *    Or that each dicitonary is heavily implemented. Making
 *    an unoptimized space usage probably acceptable.
 * 2. An enum to contain every compression method fully implemented
 *    so that Segments can indicate which compression method should
 *    be used.
 *
 * Current Implementations:
 * I have files for each of the three listed however,
 * they have not been implemented or heavily explored.
 */


/* An enum holding every supported compression format
 * Used to indicate what method of compression should be used
 */
#[derive(Clone,Debug,Serialize,Deserialize, PartialEq)]
pub enum Methods {
    Uncompr,    // 0
    Gorilla,
    Gzip,
    Snappy,
    Zlib,
    Sprintz (usize),
    Buff (usize),
    Kernel (DictionaryId),
    SparseLearning (DictionaryId),
    DeepLearning (String),
    Rrd_sample,
    Bufflossy (usize,usize),
    Paa (usize),
    Fourier (f64),
    Pla (f64)
}

pub fn IsLossless(m: &Methods) -> bool {
    match m {
        Methods::Uncompr => true,
        Methods::Gorilla => true,
        Methods::Sprintz(_) => true,
        Methods::Gzip => true,
        Methods::Zlib => true,
        Methods::Snappy => true,
        Methods::Buff(_) => true,
        Methods::Rrd_sample => false,
        Methods::Bufflossy (_,_) => false,
        Methods::Paa (_) => false,
        Methods::Fourier (_) => false,
        Methods::Pla(_) => false,
        _ => {  false },
    }
}

impl fmt::Display for Methods {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Methods::Fourier (ratio) => write!(f,"{}", format!("Fourier w/ ratio {:?}", ratio)),
            Methods::Pla (ratio) => write!(f,"{}", format!("PLA lttb w/ ratio {:?}", ratio)),
            Methods::Buff (scale) => write!(f,"{}", format!("BUFF w/ scale {:?}", scale)),
            Methods::Sprintz (scale) => write!(f,"{}", format!("Sprintz w/ scale {:?}", scale)),
            Methods::Bufflossy (scale, bits) => write!(f,"{}", format!("BUFF w/ scale {:?}, bits {:?}", scale, bits)),
            Methods::Paa (ws) => write!(f,"{}", format!("Paa w/ window size {:?}", ws)),
            Methods::Rrd_sample => write!(f,"{}", format!("Round robin data management")),
            Methods::Uncompr => write!(f,"{}", format!("No compression applied")),
            Methods::Kernel (id) => write!(f, "{}", format!("Kernel w/ DictionaryId {:?}", id)),
            Methods::SparseLearning (id) => write!(f, "{}", format!("Sparse Learning w/ DictionaryId {:?}", id)),
            Methods::DeepLearning (file) => write!(f, "{}", format!("Deep Learning w/ file {:?}", file)),
            _ => todo!()
        }
    }
}

/* Structures to be fleshed out later to provide error information
 * when methods are not properly implemented/used
 */
pub struct MethodUsageError {
    err_msg: &'static str,
    err_num: u8,
}

pub struct MethodImplementError {
    err_msg: &'static str,
    err_num: u8,
}