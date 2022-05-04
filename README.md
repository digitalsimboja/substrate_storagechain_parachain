

This task implements a pallet using the Substrate Node Template

The pallet stores a number and an action associated with each storage
item.

A user can change the associated action with the `change_action` extrinsic
A user can execute a call of either `Increment`, `Decrement`, `Idle` to affect
the stored value in a specific way.

I added a check to ensure the owner or the person who made a storage is the once
making changes on the storage


You can setup `tmux` to be able to split the terminal into several panes
`sudo apt install tmux`

Then start a tmux session
`tmux`

You can learn more about `tmux` [here](https://linuxize.com/post/getting-started-with-tmux/)



---Setup---

Ensure you have Rust setup on your machine. Follow these steps to get Substrate Node up and running
📝 Setting up Enviroment [Rust](https://docs.substrate.io/tutorials/v3/create-your-first-substrate-chain/)

Clone the repo:
`git clone https://github.com/digitalsimboja/t3rn_substrate_task.git`

CD into the Storagechain directory
`cd storagechain`

Run the command below to install the dependencies
`cargo build --release`

To start the Local Node:
` ./target/release/node-storagechain --dev --tmp`

Open another `tmux` pane and change to the substrate-front-end-template
`cd substrate-front-end-template`

Start the client 
`yarn start`

You can now interact with the Storagechain

Alternatively, you could use [Polkadot-JS](https://polkadot.js.org/apps/#/explorer)
Once it's open, select the Development and choose Local Node.
Then switch to `Storagechain` and interact with the pallet.
