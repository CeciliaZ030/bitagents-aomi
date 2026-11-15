# BIT Agents on Aomi

One Aomi app that runs every agent built on BITAGENTS.

An agent is a strategy spec stored by the BITAGENTS strategy service. The app
has two tools, and each call names the agent by id:

| Tool | What it does |
| --- | --- |
| `bitagents_status` | Reads the agent's strategy, limits, signals and the connected wallet's holdings. Never trades. |
| `bitagents_run` | Runs the strategy once for the connected Solana wallet and returns orders plus a `next` field. |

The app never signs or sends anything. In paper mode the strategy service has
already simulated each swap on mainnet. In live mode the model stages the
returned instructions with Aomi's own Solana tools, and the wallet signs.

## Secrets

Set both in the Project vault before the first chat:

| Slot | Value |
| --- | --- |
| `BITAGENTS_API_URL` | Public base URL of the BITAGENTS strategy service. |
| `BITAGENTS_APP_KEY` | The service's `BUILDER_APP_KEY`. |

## Test

```sh
cargo test --locked
```

## Deploy

```sh
aomi-build login --backend https://api-staging.aomi.dev
aomi-build project create
aomi-build deploy
```

`aomi.toml` leaves `server_tags` unset, so only staging backends load the
release. Set `server_tags = ["prod"]` and deploy against
`https://api.aomi.dev` for production.

## Callers

- `web/` renders `<AomiWidget>` in direct mode for the application id. Copy
  `web/src/main.tsx` into a site to embed the chat.
- `client/` sends one prompt from Node: `pnpm --filter client ask "Run agent ag1ea6b8de"`.

Bump `BUILD` in `src/lib.rs` on every change; both tools report it as `app_build`.
