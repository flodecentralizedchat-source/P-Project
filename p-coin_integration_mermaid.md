# P‑Coin DApp Integration — Mermaid Diagrams Only

```mermaid
flowchart TD
  U[Users<br/>Donors • Holders • Merchants • Players] --> W[Wallets<br/>MetaMask / WalletConnect]
  W --> FE[DApp Frontend<br/>React/Yew + Ethers.js]
  FE -->|read/write| CHAIN[(EVM Network<br/>Base/Polygon/BSC)]
  FE --> IDX[[Indexers<br/>The Graph / Subsquid]]
  FE --> PAY[Payments API<br/>NowPayments/Coinbase Commerce]
  FE --> STOR[(IPFS/Arweave<br/>docs, images)]

  subgraph On-chain Contracts
    TKN[PToken (ERC20)]
    DAO[PGovernor (DAO) + Timelock]
    MST[MultiSig Treasury]
    ALLOC[CharityAllocator]
    AIR[AirdropDistributor<br/>(Merkle + Allowlist)]
    VEST[VestingVaults]
    STAKE[StakingRewards]
    LQ[LiquidityManager]
    BR[BridgeAdapter]
  end

  CHAIN --- TKN
  CHAIN --- DAO
  CHAIN --- MST
  CHAIN --- ALLOC
  CHAIN --- AIR
  CHAIN --- VEST
  CHAIN --- STAKE
  CHAIN --- LQ
  CHAIN --- BR

  ADM[Ops/Admin Panel] --> MST
  ADM --> DAO
  NGO[Verified NGOs] -->|pull grants| ALLOC
  MERCH[Merchants] --> PAY
  PAY --> FE
  IDX --> FE
```

```mermaid
classDiagram
  class PToken {
    +name()
    +symbol()
    +decimals()
    +totalSupply()
    +transfer()
    +permit(EIP-2612)*
    +mint(owner-only)*
    +burn()
    <<Events>> Transfer, Burn, Mint
  }

  class PGovernor {
    +propose(targets,calldata,desc)
    +voteWithToken()
    +queue()
    +execute()
    Uses: TimelockController
    <<Events>> ProposalCreated, VoteCast, Executed
  }

  class Treasury {
    +submitTx()
    +confirmTx()
    +executeTx()
    Signers: N-of-M
  }

  class CharityAllocator {
    +setNGO(address, verified)
    +allocate(amount, ngo)
    +disburse()
    Guard: DAO-only
  }

  class AirdropDistributor {
    +claim(index, account, amount, merkleProof)
    +deadline
    +clawback(unclaimed->Treasury)
  }

  class VestingVault {
    +createStream(beneficiary, amount, cliff, duration)
    +withdraw()
    +revoke(DAO-only)
  }

  class StakingRewards {
    +stake(amount)
    +withdraw()
    +getReward()
    +notifyRewardAmount()
    Source: Treasury/Emission
  }

  class LiquidityManager {
    +seedLP(baseToken, amountP, amountBase)
    +lockLP(duration)
  }

  class BridgeAdapter {
    +mintWrappedP()
    +burnForBridge()
    Integrates: official bridge
  }

  PGovernor --> Treasury : controls
  PGovernor --> CharityAllocator : config
  Treasury --> StakingRewards : funds rewards
  Treasury --> AirdropDistributor : funds drops
  PToken <.. StakingRewards : rewardToken
  PToken <.. AirdropDistributor : airdropToken
  PToken <.. LiquidityManager : add/remove liquidity
  PToken <.. VestingVault : streams/locks
```