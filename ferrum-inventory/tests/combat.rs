use ferrum_inventory::{attack, Health, Weapon};

#[test]
fn test_health_creation() {
    let health = Health::new(20);
    assert_eq!(health.current(), 20);
    assert_eq!(health.max(), 20);
    assert!(!health.is_dead());
}

#[test]
fn test_take_damage() {
    let mut health = Health::new(20);
    health.take_damage(5);

    assert_eq!(health.current(), 15);
    assert!(!health.is_dead());
}

#[test]
fn test_death_at_zero() {
    let mut health = Health::new(10);
    health.take_damage(10);

    assert_eq!(health.current(), 0);
    assert!(health.is_dead());
}

#[test]
fn test_death_below_zero() {
    let mut health = Health::new(10);
    health.take_damage(15);

    assert_eq!(health.current(), 0);
    assert!(health.is_dead());
}

#[test]
fn test_heal() {
    let mut health = Health::new(20);
    health.take_damage(10);
    health.heal(5);

    assert_eq!(health.current(), 15);
}

#[test]
fn test_heal_cannot_exceed_max() {
    let mut health = Health::new(20);
    health.take_damage(5);
    health.heal(10);

    assert_eq!(health.current(), 20);
}

#[test]
fn test_fist_damage() {
    let weapon = Weapon::Fist;
    assert_eq!(weapon.damage(), 1);
}

#[test]
fn test_wooden_sword_damage() {
    let weapon = Weapon::WoodenSword;
    assert_eq!(weapon.damage(), 4);
}

#[test]
fn test_stone_sword_damage() {
    let weapon = Weapon::StoneSword;
    assert_eq!(weapon.damage(), 5);
}

#[test]
fn test_iron_sword_damage() {
    let weapon = Weapon::IronSword;
    assert_eq!(weapon.damage(), 6);
}

#[test]
fn test_diamond_sword_damage() {
    let weapon = Weapon::DiamondSword;
    assert_eq!(weapon.damage(), 7);
}

#[test]
fn test_wooden_axe_damage() {
    let weapon = Weapon::WoodenAxe;
    assert_eq!(weapon.damage(), 7);
}

#[test]
fn test_stone_axe_damage() {
    let weapon = Weapon::StoneAxe;
    assert_eq!(weapon.damage(), 9);
}

#[test]
fn test_iron_axe_damage() {
    let weapon = Weapon::IronAxe;
    assert_eq!(weapon.damage(), 9);
}

#[test]
fn test_diamond_axe_damage() {
    let weapon = Weapon::DiamondAxe;
    assert_eq!(weapon.damage(), 9);
}

#[test]
fn test_attack_reduces_health() {
    let mut health = Health::new(20);
    let weapon = Weapon::StoneSword;

    attack(&weapon, &mut health);

    assert_eq!(health.current(), 15);
}

#[test]
fn test_attack_can_kill() {
    let mut health = Health::new(5);
    let weapon = Weapon::DiamondSword;

    attack(&weapon, &mut health);

    assert!(health.is_dead());
}

#[test]
fn test_multiple_attacks() {
    let mut health = Health::new(20);
    let weapon = Weapon::Fist;

    for _ in 0..10 {
        attack(&weapon, &mut health);
    }

    assert_eq!(health.current(), 10);
}

#[test]
fn test_attack_after_death_does_nothing() {
    let mut health = Health::new(5);
    let weapon = Weapon::DiamondSword;

    attack(&weapon, &mut health);
    assert!(health.is_dead());

    attack(&weapon, &mut health);
    assert_eq!(health.current(), 0);
}

#[test]
fn test_respawn() {
    let mut health = Health::new(20);
    health.take_damage(20);
    assert!(health.is_dead());

    health.respawn();

    assert_eq!(health.current(), 20);
    assert!(!health.is_dead());
}
