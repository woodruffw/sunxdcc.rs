//! `sunxdcc` is a small, unofficial Rust wrapper for the SunXDCC search engine's HTTP API.

#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]
#![allow(clippy::redundant_field_names)]
#![forbid(unsafe_code)]

use itertools::izip;
use serde::Deserialize;
use thiserror::Error;
use url::Url;

const BASE_URL: &str = "https://sunxdcc.com/deliver.php";

/// Represents the errors that can occur when retrieving search results.
#[derive(Debug, Error)]
pub enum Error {
    /// An request error occurred.
    #[error("request error")]
    Request(#[from] reqwest::Error),
    /// A response contains malformed results.
    #[error("malformed response: {0}")]
    Malformed(String),
}

/// Represents the raw results from a single search request's response.
#[derive(Deserialize)]
struct RawResult {
    network: Vec<String>,
    channel: Vec<String>,
    bot: Vec<String>,
    fsize: Vec<String>,
    fname: Vec<String>,
    packnum: Vec<String>,
    gets: Vec<String>,
    botrec: Vec<String>,
}

impl RawResult {
    /// Are the contents of this `RawResult` consistent?
    ///
    /// Internally, a `RawResult` is a bunch of adjacent lists, and is
    /// "consistent" if and only if all lists are the same length.
    fn is_consistent(&self) -> bool {
        self.network.len() == self.channel.len()
            && self.channel.len() == self.bot.len()
            && self.channel.len() == self.fsize.len()
            && self.channel.len() == self.fname.len()
            && self.channel.len() == self.packnum.len()
            && self.channel.len() == self.gets.len()
            && self.channel.len() == self.botrec.len()
    }

    /// Consume this `RawResult`, constructing into `results`.
    fn consume(self, results: &mut Vec<SearchResult>) -> Result<(), Error> {
        if !self.is_consistent() {
            return Err(Error::Malformed("mismatch in adjacent list sizes".into()));
        }

        // Each result is inserted in reverse order, so that we can `pop` them later.
        for (network, channel, bot, fsize, fname, packnum, gets, botrec) in izip!(
            self.network.into_iter().rev(),
            self.channel.into_iter().rev(),
            self.bot.into_iter().rev(),
            self.fsize.into_iter().rev(),
            self.fname.into_iter().rev(),
            self.packnum.into_iter().rev(),
            self.gets.into_iter().rev(),
            self.botrec.into_iter().rev(),
        ) {
            let botrec = match botrec.as_str() {
                "Na" => None,
                _ => Some(botrec),
            };

            results.push(SearchResult {
                network: network,
                channel: channel,
                bot: bot,
                filesize: fsize,
                filename: fname,
                packet_number: packnum,
                download_count: gets,
                upload_speed: botrec,
            })
        }

        Ok(())
    }
}

/// Represents a single result from SunXDCC's search API.
#[derive(Debug)]
pub struct SearchResult {
    /// The IRC network that this result's bot is on.
    ///
    /// This is typically the fully-qualified domain name (e.g. `irc.foo.net`) or
    /// IP address for the record.
    pub network: String,

    /// The IRC channel that this result's bot is on.
    ///
    /// This is typically formatted as `"#channelname"`.
    pub channel: String,

    /// The IRC bot's name.
    pub bot: String,

    /// The size of the file.
    ///
    /// This is typically formatted as `"[XXXS]"`, e.g. `"[123M]"` for 123MB.
    pub filesize: String,

    /// The filename.
    pub filename: String,

    /// The packet number for this result.
    ///
    /// This is typically formatted as `"#XXX"`, e.g. `"#123"` for packet number 123.
    pub packet_number: String,

    /// The file's download count.
    ///
    /// This is typically formatted as `"Xx"`, e.g `"5x"` for 5 downloads.
    pub download_count: String,

    /// The serving IRC bot's maximum upload speed, if known.
    ///
    /// This is typically formatted as `"XXXX.YYkB/s"`, e.g. `"1000.25kB/s"`.
    pub upload_speed: Option<String>,
}

/// A stateful iteration container for search results.
#[derive(Debug)]
pub struct SearchResults<'search> {
    /// The client to use for all requests.
    client: reqwest::blocking::Client,
    /// The search query.
    query: &'search str,
    /// The current result page.
    current_page: usize,
    /// The current list of results.
    current_results: Vec<SearchResult>,
}

impl<'search> SearchResults<'search> {
    fn new(query: &'search str) -> Self {
        // Each query returns a maximum number of 50 results, so reserve at least
        // that many elements in our `current_results` buffer.
        Self {
            client: reqwest::blocking::Client::new(),
            query: query,
            current_page: 0,
            current_results: Vec::with_capacity(50),
        }
    }

    /// Refresh our internal state, fetching more results from the API if available.
    ///
    /// This function doesn't check whether the current results have been fully consumed;
    /// callers must take care to fully consume all current results to avoid silently
    /// skipping results.
    fn refresh(&mut self) -> Result<(), Error> {
        self.current_results.clear();

        // Unwrap safety: BASE_URL is a correct URL and our parameters cannot cause an error.
        #[allow(clippy::unwrap_used)]
        let url = Url::parse_with_params(
            BASE_URL,
            &[
                ("sterm", self.query),
                ("page", &self.current_page.to_string()),
            ],
        )
        .unwrap();

        self.client
            .get(url)
            .send()?
            .json::<RawResult>()?
            .consume(&mut self.current_results)?;

        self.current_page += 1;

        Ok(())
    }
}

impl Iterator for SearchResults<'_> {
    type Item = Result<SearchResult, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // If we're just starting or we've exhausted our results, refresh our state.
        if self.current_page == 0 || self.current_results.is_empty() {
            if let Err(e) = self.refresh() {
                return Some(Err(e));
            }
        }

        // NOTE: This produces results in the correct order, despite the `pop`.
        // See the implementation of RawResult::consume.
        Ok(self.current_results.pop()).transpose()
    }
}

/// Search SunXDCC for the given `query`.
///
/// The returned `SearchResults` is an [`Iterator`](Iterator) over individual
/// [`SearchResult`](SearchResult) items.
///
/// ```no_run
/// # use sunxdcc;
/// for result in sunxdcc::search("the hitchhiker's guide to the galaxy") {
///     println!("{:?}", result.unwrap());
/// }
/// ```
pub fn search(query: &str) -> SearchResults {
    SearchResults::new(query)
}
