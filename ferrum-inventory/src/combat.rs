#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health {
    current: u32,
    max: u32,
}

impl Health {
    pub fn new(max: u32) -> Self {
        Self { current: max, max }
    }

    pub fn current(&self) -> u32 {
        self.current
    }

    pub fn max(&self) -> u32 {
        self.max
    }

    pub fn is_dead(&self) -> bool {
        self.current == 0
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.current = self.current.saturating_sub(amount);
    }

    pub fn heal(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn respawn(&mut self) {
        self.current = self.max;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weapon {
    Fist,
    WoodenSword,
    StoneSword,
    IronSword,
    DiamondSword,
    WoodenAxe,
    StoneAxe,
    IronAxe,
    DiamondAxe,
}

impl Weapon {
    pub fn damage(&self) -> u32 {
        match self {
            Weapon::Fist => 1,
            Weapon::WoodenSword => 4,
            Weapon::StoneSword => 5,
            Weapon::IronSword => 6,
            Weapon::DiamondSword => 7,
            Weapon::WoodenAxe => 7,
            Weapon::StoneAxe => 9,
            Weapon::IronAxe => 9,
            Weapon::DiamondAxe => 9,
        }
    }
}

pub fn attack(weapon: &Weapon, target: &mut Health) {
    if !target.is_dead() {
        target.take_damage(weapon.damage());
    }
}
