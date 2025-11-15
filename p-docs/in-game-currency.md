# In-Game Currency

This guide explains how the system rewards peaceful missions and positive behavior with P-Coin-based tokens.

## Key Concepts

- **Peaceful missions**: Game activities tagged as non-violent (e.g., community service, environmental care).
- **Non-violent reward mechanics**: Missions must be non-violent, carry a multiplier that affects token payout, and can only be registered by admins.
- **Positive behavior reward tokens**: Players earn tokens when logging behaviors like helping neighbors or mediating conflicts.

## Environment variables

- `GAME_BASE_MISSION_REWARD` – Base amount awarded per mission before multiplier (default `25`).
- `GAME_BEHAVIOR_HELPING_HANDS`, `GAME_BEHAVIOR_ENVIRONMENTAL`, `GAME_BEHAVIOR_CONFLICT`, `GAME_BEHAVIOR_EDUCATION` – Token awards for each behavior type (defaults: 15, 12, 18, 10).

## 1. Register a peaceful mission (admin only)

```
POST /game/register-mission
Authorization: Bearer <admin_jwt>
{
  "mission": {
    "mission_id": "community-cleanup",           // optional; generated if empty
    "name": "Community Cleanup",
    "description": "Work with neighbors to clean the park",
    "reward_multiplier": 1.6,
    "tags": ["community", "environment"],
    "is_nonviolent": true,
    "created_at": "2025-04-01T00:00:00"
  }
}
```

Only non-violent missions with positive multipliers are allowed; repeated registrations update the stored mission.

## 2. Complete a mission (user/admin)

```
POST /game/complete-mission
Authorization: Bearer <jwt>
{
  "player_id": "player123",
  "mission_id": "community-cleanup"
}
```

Response includes a `RewardReceipt` with the number of tokens awarded (`base_reward * multiplier`), updated balance, and a timestamp. Each mission can only be completed once per player.

## 3. Record positive behavior

```
POST /game/record-behavior
Authorization: Bearer <jwt>
{
  "player_id": "player123",
  "behavior": "HelpingHands"
}
```

Behavior names correspond to the `PositiveBehavior` enum. The API adds the configured award to the player's balance and returns the receipt.

## 4. Check a player's in-game balance

```
GET /game/balance/player123
Authorization: Bearer <jwt>
```

Returns the total tokens earned through missions and behaviors.

## Notes

- The current implementation stores missions and balances in memory, so it is best suited for prototyping; connect the service to persisted storage before production use.
- Design missions around peaceful activities (education, grants, conservation) and ensure `is_nonviolent` is always `true`.
