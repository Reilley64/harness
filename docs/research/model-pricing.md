# OpenAI model pricing

Verified 2026-11-06 against OpenAI's official
[API pricing table](https://developers.openai.com/api/docs/pricing) and each
model's reference page.

## Catalog rates

These are **Standard processing, short-context text-token prices in USD per
million tokens**, not ChatGPT subscription charges. The pricing page's default
Short / Standard table agrees with the individual model pages.

| Model           |  Input | Cached input | Output | Model reference                                                              |
| --------------- | -----: | -----------: | -----: | ---------------------------------------------------------------------------- |
| `gpt-6-luna`    |  $0.10 |        $0.01 |  $0.50 | [GPT-6 Luna](https://developers.openai.com/api/docs/models/gpt-6-luna)       |
| `gpt-6.1-terra` |  $1.00 |        $0.05 |  $5.00 | [GPT-6.1 Terra](https://developers.openai.com/api/docs/models/gpt-6.1-terra) |
| `gpt-6.1-sol`   |  $2.00 |        $0.10 | $10.00 | [GPT-6.1 Sol](https://developers.openai.com/api/docs/models/gpt-6.1-sol)     |
| `gpt-6-astra`   | $10.00 |        $1.00 | $50.00 | [GPT-6 Astra](https://developers.openai.com/api/docs/models/gpt-6-astra)     |

All four IDs appear in the official
[model catalog](https://developers.openai.com/api/docs/models).

The values are recorded in `server/models.json` under `pricing.input`,
`pricing.cached_input`, and `pricing.output`. They are a manually verified
snapshot, not a live pricing integration.

## Limits

- The model references state that prompts with **more than 272K input tokens**
  use 2× input/cache rates and 1.5× output rates for the full request. The catalog
  stores only the base short-context rates.
- Batch/Flex, Fast, and other processing modes have different rates. Regional
  processing and other premiums may also apply; consult the
  [pricing table](https://developers.openai.com/api/docs/pricing).
- Cache-write pricing is distinct from cached-input/read pricing and is not
  represented by our three-field pricing schema. Tool fees are also excluded.
- API rates do not describe how a ChatGPT subscription's allowance is consumed.
  The picker continues to distinguish catalog prices from plan billing.
- This change updates prices only, not the catalog's effort capability list.

The individual model reference links above are the sources for the token-rate
and long-context figures. The pricing table is the source for processing-mode
comparisons and regional premiums.
