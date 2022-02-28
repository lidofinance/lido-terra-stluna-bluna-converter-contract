# Converter Contract

The Converter Contract contains the logic for swapping stLuna/bLuna tokens with the same API as [Astroport's native pool
contract](https://github.com/astroport-fi/astroport-core/tree/master/contracts/pair#executemsg), but it's just simply calls [Hub::Convert](https://docs.terra.lido.fi/contracts/hub#convert) under the hood.


## InstantiateMsg

Initializes a new converter contract.

```json
{
    "stluna_address": "terra1...",
    "bluna_address": "terra1...",
    "hub_address": "terra1...",
}
```

## ExecuteMsg

### `receive`

CW20 receive hander. Supports only Swap: {} message.

```json
{
  "receive": {
    "sender": "terra...",
    "amount": "123",
    "msg": "<base64_encoded_json_string>"
  }
}
```

### `provide_liquidity`

Not supported. Returns ```ContractError::NonSupported {}``` error.

### `withdraw_liquidity`

Not supported. Returns ```ContractError::NonSupported {}``` error.

### `swap`

Perform a swap. `offer_asset` is your source asset and `to` is the address that will receive the ask assets. All fields are optional except `offer_asset`.

Calls [Hub::Convert](https://docs.terra.lido.fi/contracts/hub#convert) under the hood.

NOTE: You should increase token allowance before swap.

```json
  {
    "swap": {
      "offer_asset": {
        "info": {
          "native_token": {
            "denom": "uluna"
          }
        },
        "amount": "123"
      },
      "belief_price": "123",
      "max_spread": "123",
      "to": "terra..."
    }
  }
```

### `update_config`

Not supported. Returns ```ContractError::NonSupported {}``` error.

## QueryMsg

All query messages are described below. A custom struct is defined for each query response.

### `pair`

Retrieve a pair's configuration (type, assets traded in it etc)

```json
{
  "pair": {}
}
```

### `pool`

*Returns the amount of tokens in the pool for all assets as well as the amount of LP tokens issued.*

In the case of the conveter contract *amount of tokens in the pool for all assets* means *the total amount of issued tokens for all assets* and LP tokens issued is always zero since the contract doesn't have LP tokens logic.

```json
{
  "pool": {}
}
```

### `config`

Get the pair contract configuration.

```json
{
  "config": {}
}
```

### `share`

*Returns the amount of assets someone would get from the pool if they were to burn a specific amount of LP tokens.*

In the case of the contract always returns an empty array since the converter contract doesn't have LP tokens support.

```json
{
  "share": {
    "amount": "123"
  }
}
```

### `simulation`

*Simulates a swap and returns the spread and commission amounts.*

The spread and comimission amounts equal to zero since no actual pool swap happens.

```json
{
  "simulation": {
    "offer_asset": {
      "info": {
        "native_token": {
          "denom": "uusd"
        }
      },
      "amount": "1000000"
    }
  }
}
```

### `reverse_simulation`

*Reverse simulates a swap (specifies the ask instead of the offer) and returns the offer amount, spread and commission.*

The spread and comimission amounts equal to zero since no actual pool swap happens.

```json
{
  "reverse_simulation": {
    "ask_asset": {
      "info": {
        "token": {
          "contract_addr": "terra..."
        }
      },
      "amount": "1000000"
    }
  }
}
```

### `cumulative_prices`

Returns the cumulative prices for the assets in the pair.

```json
{
  "cumulative_prices": {}
}
```


## TWAP

TWAP stands for the time-weighted average price. It’s a reliable average price that can exclude short-term price fluctuation or manipulation and have been widely used in DeFi ([Astroport's usage](https://docs.astroport.fi/astroport/smart-contracts/oracles#time-weighted-average-prices))

The time-weighted price algorithm is quite simple: the price P multiplied by how long it lasts T is continuously added to a cumulative value C. ([more info](https://docs.uniswap.org/protocol/V2/concepts/core-concepts/oracles)).

But the converter contract might have a problem with calculations of cumulative prices since stLuna price always grows (actually stLuna price increases every hour). For example, if no swaps were made in a large amount of time the converter can't know how long the stLuna price lasts.
Thus we see three solutions:
* implement a bot which will update a cumulative prices in the conveter contract each time when stLuna exchange rate changes in the hub;  
* querying the `last_index_modification` field from the hub (last time when stLuna exchange rate changes) to properly calculate how the stLuna price lasts;
* do no modifications in the code.


For this three solutions we've wrote a simple script which simulates all three of them:

On the upper left chart you can see how stLuna exchange rate grows.

On the upper right - accumulated prices for **three** ways of calculations: usual and bot ways are very close to each over (you can see a percentage difference between them on middle left chart - ~0.005%) and `last_index_modification` way is a way below.

So we've decided make no changes in the existed code.

### How to run a simulation

* Install dependencies:
```
pip3 install -r simulation/requirements.txt
```

* Run the script
```
python3 simulation/simulation.py
```