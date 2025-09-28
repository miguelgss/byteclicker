use macroquad::prelude::*;

#[derive(Clone, Debug)]
enum ELevel {
    ROOKIE,
    CHAMPION,
    ULTIMATE,
}

#[derive(Clone, Debug)]
enum EAttribute {
    VACCINE,
    DATA,
    VIRUS,
    FREE,
}

#[derive(Clone, Debug)]
struct HpSystem {
    hp_base: u64,
    hp: u64,
}

impl HpSystem {
    fn update_hp(&mut self, value: i64) {
        if value >= 0 {
            self.hp = clamp(self.hp + value as u64, 0, self.hp_base);
        }
    }

    fn do_damage(&mut self, value: i64) {
        if value as u64 > self.hp {
            self.hp = 0;
        } else {
            self.hp = self.hp - value as u64;
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }
}

#[derive(Clone)]
struct LevelStatusSystem {
    level: u32,
    exp: u64,
    status_upgrade: Status,
    exp_needed: u64,
    needed_multiplier: u64,
}

impl LevelStatusSystem {
    fn formula_lvlup(&self) -> u64 {
        self.exp_needed * self.needed_multiplier.pow(self.level as u32)
    }

    fn to_next_level(&self) -> u64 {
        if self.level < 999 {
            return self.formula_lvlup() - self.exp;
        }
        0
    }

    fn update_exp(&mut self, exp: u64) {
        self.exp += exp;
        if self.exp > self.formula_lvlup() && self.level < 999 {
            self.level += 1;
        }
    }

    fn given_exp(&self, lvl: u32) -> u64 {
        self.formula_lvlup() / 20 * clamp(self.level as i64 - lvl as i64, 1, 999) as u64 + 1
    }

    fn get_upgraded_status(&self) -> Status {
        Status {
            str: &self.status_upgrade.str * self.level as u64,
            speed: &self.status_upgrade.speed * self.level as u64,
            def: &self.status_upgrade.def * self.level as u64,
        }
    }
}

impl Default for LevelStatusSystem {
    fn default() -> Self {
        Self {
            level: 1,
            exp: 0,
            exp_needed: 30,
            status_upgrade: Status {
                str: 5,
                def: 4,
                speed: 2,
            },
            needed_multiplier: 1,
        }
    }
}

#[derive(Clone)]
struct Status {
    str: u64,
    def: u64,
    speed: u64,
}

impl Default for Status {
    fn default() -> Self {
        Self {
            str: 20,
            def: 10,
            speed: 10,
        }
    }
}

impl Status {
    fn new(str: u64, def: u64, speed: u64) -> Self {
        Self { str, def, speed }
    }

    fn sum(&mut self, status_upg: &Status) -> &Status {
        self.str += status_upg.str;
        self.def += status_upg.def;
        self.speed += status_upg.speed;

        self
    }
}

#[derive(Clone)]
struct BytePet {
    id: u8,
    s_level: LevelStatusSystem,
    byte_level: ELevel,
    attribute: EAttribute,
    status: Status,
}

impl Default for BytePet {
    fn default() -> Self {
        Self {
            id: 1,
            s_level: LevelStatusSystem::default(),
            byte_level: ELevel::ROOKIE,
            attribute: EAttribute::FREE,
            status: Status::default(),
        }
    }
}

impl BytePet {
    fn new(
        id: u8,
        level: LevelStatusSystem,
        byte_level: ELevel,
        attribute: EAttribute,
        status: Status,
    ) -> Self {
        Self {
            id,
            byte_level,
            s_level: level,
            attribute,
            status,
        }
    }

    fn get_power(&self) -> Status {
        let total_status = Status {
            str: self.status.str + self.s_level.get_upgraded_status().str,
            def: self.status.def + self.s_level.get_upgraded_status().def,
            speed: self.status.speed + self.s_level.get_upgraded_status().speed,
        };

        total_status
    }
}

#[derive(Clone)]
struct Battler {
    s_hp: HpSystem,
    name: String,
    turn_timer: u32,
    data: BytePet,
}

impl Default for Battler {
    fn default() -> Self {
        let new_hp = HpSystem {
            hp_base: 200,
            hp: 200,
        };
        Self {
            s_hp: new_hp,
            name: "PHoldermon".to_owned(),
            turn_timer: 0,
            data: BytePet::default(),
        }
    }
}

impl Battler {
    fn get_hp(&mut self) -> u64 {
        self.s_hp.hp
    }

    fn change_hp(&mut self, value: i64) {
        self.s_hp.update_hp(value);
    }
}

struct TeamManager {
    active_team: [Option<Battler>; 3],
}

impl TeamManager {
    fn get_team_power(&mut self) -> Status {
        let mut team_status = Status {
            str: 0,
            def: 0,
            speed: 0,
        };

        for e in self.active_team.iter_mut() {
            if let Some(x) = e {
                team_status.sum(&x.data.get_power());
            }
        }

        team_status
    }
}

struct Player {
    clicks: u64,
    total_defeated: u64,
    active_team: TeamManager,
}

impl Default for Player {
    fn default() -> Self {
        const EMPTY_PET: Option<Battler> = None;
        Self {
            clicks: 0,
            total_defeated: 0,
            active_team: TeamManager {
                active_team: [EMPTY_PET; 3],
            },
        }
    }
}

impl Player {
    fn add_pet(&mut self, new_bp: Battler) -> bool {
        for e in self.active_team.active_team.iter_mut() {
            if e.is_none() {
                *e = Some(new_bp);
                return true;
            }
        }
        false
    }

    fn get_power(&mut self) -> i64 {
        let mut dmg: i64 = 0;
        dmg = dmg + self.active_team.get_team_power().str as i64;
        print!("POWER: {}", dmg);
        dmg
    }
}

struct Scene {
    name: String,
    possible_enemies: Vec<Battler>,
    active_enemy: Battler,
}

impl Scene {
    fn do_damage(&mut self, dmg: i64) -> Option<&Battler> {
        print!("dmg value: {}", dmg);
        self.active_enemy.s_hp.do_damage(dmg);
        self.check_enemy_defeated()
    }

    fn check_enemy_defeated(&mut self) -> Option<&Battler> {
        if !self.active_enemy.s_hp.is_alive() {
            self.new_enemy();
            return Some(&self.active_enemy);
        }
        None
    }

    fn new_enemy(&mut self) {
        if let Some(x) = self.possible_enemies.get(0) {
            self.active_enemy = x.clone();
        }
    }

    fn auto_damage(&mut self, dt: f64) {}
}

struct GameState {
    player: Player,
    scene: Scene,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "ByteClicker".to_owned(),
        fullscreen: false,
        window_resizable: false,
        window_width: 360,
        window_height: 640,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let initial_scene = Scene {
        possible_enemies: vec![Battler::default()],
        active_enemy: Battler::default(),
        name: "Test1".to_owned(),
    };

    let mut player = Player {
        ..Default::default()
    };

    player.add_pet(Battler::default());

    let mut gs = GameState {
        player,
        scene: initial_scene,
    };

    let t: u8 = 10;
    let m: u8 = 15;

    let c = clamp(t as i16 - m as i16, 1, 20);
    println!("{}", c);
    loop {
        clear_background(BLACK);
        update(&mut gs).await;
        draw(&mut gs).await;
        next_frame().await;
    }
}

async fn update(gs: &mut GameState) {
    if is_key_pressed(KeyCode::G) || is_mouse_button_pressed(MouseButton::Left) {
        let e = gs.scene.do_damage(gs.player.get_power());
        gs.player.clicks += 1;

        if let Some(_) = e {
            gs.player.total_defeated += 1;
        }
    }
}

async fn draw(gs: &mut GameState) {
    draw_text(
        &format!(
            "Area: {}\n Clicks: {}\n EnemyHP: {}",
            gs.scene.name,
            gs.player.clicks,
            gs.scene.active_enemy.get_hp()
        ),
        10.,
        20.,
        20.,
        GREEN,
    );

    draw_text(
        &format!("Defeated: {}", gs.player.total_defeated),
        10.,
        40.,
        20.,
        GREEN,
    );
    draw_rectangle_lines(5., 4., screen_width() - 10., 60., 2., GREEN);

    draw_enemy(gs).await;
}

async fn draw_enemy(gs: &mut GameState) {
    let x_text: f32 = screen_width() / 24.;
    const OFFSET: f32 = 22.;
    const Y_TEXT: f32 = 260.;
    draw_text(
        &format!("HP: {}", gs.scene.active_enemy.get_hp()),
        x_text,
        Y_TEXT,
        26.,
        RED,
    );
    draw_text(
        &format!("NAME: {}", gs.scene.active_enemy.name),
        x_text,
        Y_TEXT + OFFSET,
        26.,
        RED,
    );
    draw_text(
        &format!("LEVEL: {}", gs.scene.active_enemy.data.s_level.level),
        x_text,
        Y_TEXT + OFFSET * 2.,
        26.,
        RED,
    );
    draw_text(
        &format!("ATTRIBUTE: {:?}", gs.scene.active_enemy.data.attribute),
        x_text,
        Y_TEXT + OFFSET * 3.,
        26.,
        RED,
    );
    draw_circle_lines(screen_width() / 2., screen_height() / 4., 80., 4., RED);
}
