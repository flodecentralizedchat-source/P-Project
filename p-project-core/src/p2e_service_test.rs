use super::p2e_service::*;

fn default_config() -> P2EConfig {
    P2EConfig {
        cooperative_reward_per_member: 10.0,
        cooperative_bonus_per_extra_member: 1.0,
        puzzle_rewards: PuzzleRewardSchedule {
            easy: 5.0,
            medium: 8.0,
            hard: 12.0,
        },
        mission_reward: 15.0,
    }
}

fn setup_player(service: &mut P2EService, username: &str) -> PlayerProfile {
    service
        .register_player(username, &format!("wallet_{}", username))
        .unwrap()
}

#[test]
fn cooperative_task_rewards_each_member() {
    let mut service = P2EService::new(default_config());
    let alice = setup_player(&mut service, "alice");
    let bob = setup_player(&mut service, "bob");
    let task = service
        .create_cooperative_task("Garden", "Community garden cleanup", 2)
        .unwrap();

    service
        .join_cooperative_task(&task.task_id, &alice.player_id)
        .unwrap();
    service
        .join_cooperative_task(&task.task_id, &bob.player_id)
        .unwrap();

    let receipts = service.complete_cooperative_task(&task.task_id).unwrap();
    assert_eq!(receipts.len(), 2);
    assert_eq!(
        service.get_balance(&alice.player_id),
        service.config.cooperative_reward_per_member
    );
    assert_eq!(
        service.get_balance(&bob.player_id),
        service.config.cooperative_reward_per_member
    );
}

#[test]
fn solving_peace_puzzle_awards_tokens() {
    let mut service = P2EService::new(default_config());
    let player = setup_player(&mut service, "puzzlefan");
    let puzzle = service
        .create_puzzle(
            "Bridge Peace",
            "Solve the community bridge riddle",
            PuzzleDifficulty::Medium,
            "unity",
            None,
        )
        .unwrap();

    let reward = service
        .solve_puzzle(&player.player_id, &puzzle.puzzle_id, "Unity")
        .unwrap();
    assert_eq!(reward, service.config.puzzle_rewards.medium);
    assert_eq!(service.get_balance(&player.player_id), reward);

    let dup = service.solve_puzzle(&player.player_id, &puzzle.puzzle_id, "unity");
    assert!(dup.is_err());
}

#[test]
fn community_mission_rewards_only_for_community_focus() {
    let mut service = P2EService::new(default_config());
    let player = setup_player(&mut service, "builder");
    let mission = service
        .create_mission(
            "Block party",
            "Host a neighborhood block party",
            MissionFocus::CommunityBuilding,
        )
        .unwrap();

    let reward = service
        .complete_mission(&player.player_id, &mission.mission_id)
        .unwrap();
    assert_eq!(reward, service.config.mission_reward);
    assert_eq!(service.get_balance(&player.player_id), reward);

    let already = service.complete_mission(&player.player_id, &mission.mission_id);
    assert!(already.is_err());

    let other_mission = service
        .create_mission("Tree planting", "Plant trees", MissionFocus::Environment)
        .unwrap();
    let blocked = service.complete_mission(&player.player_id, &other_mission.mission_id);
    assert!(blocked.is_err());
}

#[test]
fn cooperative_task_requires_enough_members() {
    let mut service = P2EService::new(default_config());
    let player = setup_player(&mut service, "solo");
    let task = service
        .create_cooperative_task("Drive", "Community drive", 2)
        .unwrap();
    service
        .join_cooperative_task(&task.task_id, &player.player_id)
        .unwrap();
    let err = service.complete_cooperative_task(&task.task_id);
    assert!(err.is_err());
}
