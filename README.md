# eBay Pricer

A simple Rust command-line tool that helps estimate competitive pricing for eBay listings.

## What it does

* Takes one or more eBay item URLs (`/itm/...`).
* Pulls the anchor item data (title, price, condition) from the eBay Browse API.
* Finds comparable listings by keyword.
* Filters comps to the same condition within ±30% of the anchor price.
* Calculates median pricing and suggests three strategies:

  * **Fast sale** (~90% of median)
  * **Market** (median)
  * **Patient sale** (~110% of median)
* Exports comps to a per-item CSV (`comps_<itemid>.csv`).

## Example usage

```bash
cargo run -- \
  "https://www.ebay.com/itm/186600508742" \
  "https://www.ebay.com/itm/196477505111"
```

## Setup

1. Install Rust ([https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)).
2. Clone this repo and `cd` into it.
3. Create a `.env` file in the project root with your eBay API credentials:

   ```
   EBAY_CLIENT_ID=your_id
   EBAY_CLIENT_SECRET=your_secret
   ```
4. Build and run:

   ```bash
   cargo run -- "<ebay_item_url>"
   ```

## Notes

* CSV outputs are ignored by git (see `.gitignore`).
* Only works with item URLs (`/itm/...`) for now — not product (`/p/...`) pages.
