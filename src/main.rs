mod error;

use clap::Clap;
use git_version::git_version;
use reqwest::Error as ReqwestError;
use thiserror::Error; 
use std::collections::HashMap;

// For get_exchange_rate
use reqwest::Url;
use sp_arithmetic::{
        FixedU128,
        traits::{
            One,
            CheckedDiv,
        }
    };

// Comes from interbtc
use primitives::{
        CurrencyId,
        CurrencyInfo,
    };

// const VERSION: &str = git_version!(args = ["--tags"]);
const VERSION: &str = "0.0.1";
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const NAME: &str = env!("CARGO_PKG_NAME");
const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

const BTC_CURRENCY: &str = "btc";

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid Currency")]
    InvalidCurrency,
    #[error("Invalid Response")]
    InvalidResponse,
    #[error("Invalid Exchange Rate")]
    InvalidExchangeRate,
    #[error("Invalid URL")]
    InvalidURL,
    
    #[error("ReqwestError: {0}")]
    ReqwestError(#[from] ReqwestError),
}


#[derive(Clap)]
#[clap(name = NAME, version = VERSION, author = AUTHORS, about = ABOUT)]
struct Opts {
   /// Target currency, e.g. "DOT" or "KSM".
   #[clap(long, parse(try_from_str = parse_collateral_currency))]
   currency_id: CurrencyId,

   /// Fetch the exchange rate from CoinGecko (https://api.coingecko.com/api/v3/).
   #[clap(long, conflicts_with("exchange-rate"))]
   coingecko: Option<Url>,

}    

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, log::LevelFilter::Info.as_str()),
    );

    let opts: Opts = Opts::parse();

    let currency_id = opts.currency_id;
    let coingecko_url = if let Some(mut url) = opts.coingecko {
        url.set_path(&format!("{}/simple/price", url.path()));
        url.set_query(Some(&format!(
            "ids={}&vs_currencies={}",
            currency_id.name().to_lowercase(),
            BTC_CURRENCY
        )));
        Some(url)
    } else {
        None
    };

    let coingecko_url = coingecko_url.ok_or(Error::InvalidURL)?;
    tracing::debug!("URL:{}", coingecko_url);
    let coingecko_result = get_exchange_rate_from_coingecko(currency_id, &coingecko_url).await?;
    tracing::debug!("Result: {}", coingecko_result);
    Ok(())
}

async fn get_exchange_rate_from_coingecko(currency_id: CurrencyId, url: &Url) -> Result<FixedU128, Error> {
    // https://www.coingecko.com/api/documentations/v3
    let resp = reqwest::get(url.clone())
        .await?
        .json::<HashMap<String, HashMap<String, f64>>>()
        .await?;


    let exchange_rate = *resp
        .get(&currency_id.name().to_lowercase())
        .ok_or(Error::InvalidResponse)?
        .get(BTC_CURRENCY)
        .ok_or(Error::InvalidResponse)?;

    tracing::debug!("Result: {}", exchange_rate);

     FixedU128::one()
         .checked_div(&FixedU128::from_float(exchange_rate))
         .ok_or(Error::InvalidExchangeRate)
}

pub fn parse_collateral_currency(src: &str) -> Result<CurrencyId, Error> {
    match src.to_uppercase().as_str() {
        id if id == CurrencyId::KSM.symbol() => Ok(CurrencyId::KSM),
        id if id == CurrencyId::DOT.symbol() => Ok(CurrencyId::DOT),
        _ => Err(Error::InvalidCurrency),
    }
}
