//! Link verification.

mod file;
mod web;

pub use file::File;
pub use web::Web;

use crate::{Cache, Link, Location};
use rayon::{iter::ParallelBridge, prelude::*};

/// Something used to check whether a link is valid.
pub trait Verifier: Sync {
    /// Check the link.
    ///
    /// # Note to Implementors
    ///
    /// If a particular type of [`Link`] isn't supported, you should quickly
    /// detect this and return [`ValidationResult ::Unsupported`] so another
    /// [`Verifier`] can be tried.
    fn verify(&self, link: &Link, cache: &dyn Cache) -> ValidationResult;
}

impl<F> Verifier for F
where
    F: Fn(&Link, &dyn Cache) -> ValidationResult,
    F: Sync,
{
    fn verify(&self, link: &Link, cache: &dyn Cache) -> ValidationResult {
        self(link, cache)
    }
}

#[derive(Debug)]
pub enum ValidationResult {
    /// This [`Link`] is valid.
    Valid,
    /// This type of link isn't supported.
    Unsupported,
    /// The link should be ignored.
    Ignored,
}

#[derive(Debug, Default)]
pub struct Outcome {}

impl Outcome {
    pub fn merge(left: Outcome, right: Outcome) -> Outcome { unimplemented!() }

    fn with_result(
        self,
        location: Location,
        link: Link,
        result: ValidationResult,
    ) -> Outcome {
        unimplemented!()
    }
}

impl FromParallelIterator<(Location, Link, ValidationResult)> for Outcome {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = (Location, Link, ValidationResult)>,
    {
        par_iter
            .into_par_iter()
            .fold(Outcome::default, |outcome, (location, link, result)| {
                outcome.with_result(location, link, result)
            })
            .reduce(Outcome::default, Outcome::merge)
    }
}

pub fn verify<L, C>(
    links: L,
    verifiers: &[Box<dyn Verifier>],
    cache: &dyn Cache,
) -> Outcome
where
    L: IntoIterator<Item = (Location, Link)>,
    L::IntoIter: Send,
    L::Item: Send,
{
    links
        .into_iter()
        .par_bridge()
        .map(|(location, link)| {
            let result = verify_one(&link, verifiers, cache);
            (location, link, result)
        })
        .collect()
}

fn verify_one(
    link: &Link,
    verifiers: &[Box<dyn Verifier>],
    cache: &dyn Cache,
) -> ValidationResult {
    if cache.is_valid(link.href()).unwrap_or(false) {
        // cache hit
        return ValidationResult::Valid;
    }

    for verifier in verifiers {
        match verifier.verify(link, cache) {
            ValidationResult::Unsupported => continue,
            other => return other,
        }
    }

    ValidationResult::Unsupported
}
