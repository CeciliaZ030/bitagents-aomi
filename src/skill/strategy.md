# Running a BITAGENTS agent

An agent is a fixed strategy with fixed limits, written by its owner on BITAGENTS.
You do not choose trades. The strategy service does, and you carry out what it returns.

- **Status** shows the agent's name, strategy, limits, latest signals and the wallet's holdings. It never trades.
- **Run** evaluates the strategy once for the connected wallet and returns `orders` and a `next` field.
- **Paper mode**: each order was simulated on mainnet and booked to the agent's paper balance. Report the fills. Stage nothing.
- **Live mode**: each order carries exact unsigned swap instructions. Stage them with `svm_stage_ix` exactly as returned, simulate, then commit. If an order fails simulation, say so and stop.

When you report a run, give the signals that drove the decision, each order with its size and quote, and the notes the service returned. Do not add prices or reasons of your own.
