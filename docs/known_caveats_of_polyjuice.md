# Known Caveats Of Polyjuice

When designing and building polyjuice, we aim at the highest level of compatibility, meaning:

* The EVM used in polyjuice shall be almost 100% compatible with the latest fork version of Ethereum;
* Via a [Web3 layer](https://github.com/nervosnetwork/godwoken-web3) and [Polyjuice web3 provider](https://github.com/nervosnetwork/polyjuice-provider), polyjuice shall be 100% compatible with Ethereum with respect to Web3 interfaces;

However, due to drastically different architecture and design considerations, there will inevitably be some differences when working on polyjuice. This article aims to document and communicate such caveats.

## Account Creation

One must create an account on a godwoken chain in order to use polyjuice on that godwoken chain.

There are two ways to create a layer 2 account:

1. Deposit funds to Godwoken at layer 1.
2. Calling Godwoken builtin contract to create an account at layer 2.

## pETH

pETH is a fixed sUDT token type chosen when deploying a polyjuice chain. pETH token type to a polyjuice chain is analogous to ETH to an Ethereum chain: it is used as a native token for charging transaction fees. The gas price of polyjuice transactions is measured using pETH designated for the polyjuice chain, which will be deducted from sender's account when the transaction is committed on chain. Different polyjuice chains might use different token type as pETH: while one polyjuice chain might use CKB as pETH, another polyjuice chain might choose to use a different sUDT type(for example, one can map native ETH tokens to CKB's sUDT via force bridge) as pETH.

Note that a godwoken chain might contain multiple polyjuice chains, each is denoted by a "creator account" much like the `0x0000 ... 0000` address on Ethereum. Different pETH token types might be used for different polyjuice chains, even though they might all coexist on the same godwoken chain.

## All Tokens Are ERC20 Tokens

Ethereum differs in the processing of ERC20 tokens, and native ETH tokens. This is also the reason why wETH is invented. Godwoken conceals this differences: whether you use a native CKB or any sUDT token type, they will all be represented in godwoken as a single layer 2 sUDT type. Polyjuice starts from this single layer 2 sUDT type and ensures that all the tokens on godwoken are in compliance with the ERC20 standard, no matter if they are backed by a native CKB or a sUDT. This means you don't need to distinguish between native token and ERC20 tokens. All you have to deal with is the same ERC20 interface for all different types of tokens.

## Godwoken Address vs Ethereum Address

Godwoken address is the biggest and the most controversial change introduced by interoperability. We strongly recommend you to read this section thoroughly and make sure you are aware of the caveats involved here.

There are 2 separate components for each blockchain:

* Identity check: How does a blockchain check the owner of an on-chain account? Typically, a digital signature verification takes place.
* Computation / Verification(depending on how you approach the problem): how does one transaction affect the on-chain state?

Ethereum, like many other blockchains of its age, has coupled the 2 components together. However, at Nervos, we believe in the power of [interoperability](https://talk.nervos.org/t/blockchain-abstraction-and-interoperability-2-0/5440). The design of godwoken, in which identity check and on-chain computation are fully decoupled and separated, unleashes new potentials:

* An Ethereum app can be used not only via Ethereum wallets such as MetaMask, but also Tron, EOS, BTC or other wallets.
* An Etheruem wallet can work on not only Ethereum apps, but also [diem](https://www.diem.com/en-us/) or other blockchain apps.

In a world where users can interact with numerous apps through a single wallet and a single address, a dapp opens up to users with different wallets using a single deployment. The hassle of having one address/wallet per blockchain would be long gone. 

Yet this aspiration comes with a price: Ethereum uses 20 byte address format at both identity and computation side. This works well for Ethereum since it couples identity check with computation. In godwoken we would want something different: not just Ethereum addresses, but also Tron, EOS, BTC, CKB, or other addresses could be used as identity. We would need to pack all those different addresses in a 20-byte address space on the computation side of polyjuice. We would not have space for all the addresses generated by other blockchains if we kept using the full 20-byte Ethereum address in polyjuice.

For this consideration, we are obliged to introduce the concept of godwoken address heres:

Godwoken address is created when godwoken creates an account. It uniquely identifies the identity address used on godwoken. When a user deposits funds to godwoken and successfully creates an account, the godwoken address will be created. The user then uses this address to uniquely locate the account on this very godwoken chain, no matter the account is a EVM contract, a Diem contract, or another EoA.

We do understand the hassles of adding a new address concept here, but we are confident that the merits will outweigh the demerits and accordingly, we have provided utilities that can tackle the problem here:

1. For each supported address formats, we will provide helper functions to convert between godwoken addresses, and the identity addresses.
2. Transaction signing will be catered entirely, so that one will only sign the transactions by using the identity address of each wallet.
3. A web3 provider tool is introduced, so when we know a parameter is an address(such as `data` in `eth_getStorageAt`, or `to` in `eth_call`), we will perform the address translation automatically.
4. We have provided a new syscall `recover_account` to replace the `ecrecover` pre-compiled contract to return godwoken address format. This way polyjuice can deal with not only Ethereum EoA, but all EoAs that godwoken supported.

Based on these changes, a few suggestions on building a polyjuice application are as follows:

1. With the polyjuice web3 provider, the workflow you have in place to deploy Ethereum contracts are expected to remain working. Godwoken addresses will be translated and returned as Ethereum addresses.
2. We do recommend to use the identity address as widely as possible on the UI side of the app, and thus users will be able to have the same address in the app and in their wallets.
3. To make a contract call with an address in one of the call parameters, it is suggested to use the polyjuice web3 provider to transform the identity address into a Godwoken address before passing the address to the contract call as parameters.
4. You will probably need to make RPC calls to convert the Godwoken addresses back to identity addresses so as to present them to the users in case you need to query the on-chain storage to obtain the addresses.

In General, it is recommended to use the identity address as often as possible and undertake the conversion to the identity address if that is absolutely necessary.
