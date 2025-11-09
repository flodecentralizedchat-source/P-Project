# p-project-dao Features

- **Proposal lifecycle management** – `DaoGovernance::create_proposal` issues unique proposal IDs, persists them to MongoDB, sets a 7-day voting window, and marks proposals as active.
- **Token-gated voting** – `vote_on_proposal` restricts voting to active proposals and records each participant’s stance, keyed by `proposal_id` and `user_id`.
- **Weighted vote tallying** – `tally_votes` pulls voter token balances from `PProjectToken`, applies weight proportional to holdings, and finalizes proposals as `Passed` or `Rejected`.
- **Active proposal discovery** – `get_active_proposals` queries MongoDB for the current set of active proposals so other services can surface them.

