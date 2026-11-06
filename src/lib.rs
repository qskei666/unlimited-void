//! Gojo Satoru — Domain Expansion: Unlimited Void (Бесконечная Пустота).
//! Нажми Y в игре (или введи `/void`), чтобы раскрыть территорию на 100 блоков.

use horizon_mod_sdk::{
    Mod, Registration,
    game,
    hud::{self, Corner, Line},
    input::{self, Event},
    log, render, world,
};

const VOID_SHADER: &str = include_str!("void_shader.wgsl");

/// Радиус купола Расширения Территории (100 блоков вокруг игрока)
const DOMAIN_RADIUS: i32 = 100;
/// Длительность территории в секундах (22 секунды)
const DOMAIN_LENGTH: f32 = 22.0;
/// Сколько колец сферы строить за один тик, чтобы не превысить лимит инструкций (fuel)
const RINGS_PER_TICK: i32 = 6;

/// Формулы бесконечных знаний, которые по очереди всплывают над головами всех мобов в домене
const KNOWLEDGE_FORMULAS: &[&str] = &[
    "§b§l[∞ 0.2s = 0.5 YEARS INFO] §f∫Ψ*(x)∇²Ψ(x)dx = ∞",
    "§9§l[SYNAPSE OVERLOAD 99.9%] §fR_μν - ½Rg_μν = 8πG·T_μν",
    "§b§l[CEREBRAL PARALYSIS] §flim(Δt→0) ∑ Ω(t) = ∞",
    "§f§l[無量空処 : UNLIMITED VOID] §biℏ ∂/∂t |Ψ⟩ = Ĥ|Ψ⟩",
    "§3§l[INFO STREAM: INFINITE] §fζ(s) = ∑ n⁻ˢ = ∏(1-p⁻ˢ)⁻¹",
    "§b§l[MIND FREEZE] §f∮_∂Ω ω = ∫_Ω dω // CORTEX LOCKED",
];

struct UnlimitedVoid {
    /// Скомпилирован ли уже шейдер Черной дыры в движке
    compiled: bool,
    warming: bool,
    ticks_in_world: u64,
    /// Активен ли сейчас домен
    active: bool,
    domain_ticks: u32,
    center: [f64; 3],
    center_block: [i32; 3],
    /// Текущий прогресс строительства купола сверху вниз (от +100 до -100 по Y)
    dome_y_cursor: i32,
    dome_built: bool,
    /// Параметры, передаваемые каждый кадр в void_shader.wgsl (16 float-значений)
    params: [f32; 16],
    /// Очередь команд для плавной постройки сферы 100 блоков
    cmd_queue: std::collections::VecDeque<String>,
    formula_cursor: usize,
}

impl UnlimitedVoid {
    fn new() -> Self {
        Self {
            compiled: false,
            warming: false,
            ticks_in_world: 0,
            active: false,
            domain_ticks: 0,
            center: [0.0; 3],
            center_block: [0; 3],
            dome_y_cursor: DOMAIN_RADIUS,
            dome_built: false,
            params: [0.0; 16],
            cmd_queue: std::collections::VecDeque::new(),
            formula_cursor: 0,
        }
    }

    /// Предварительная компиляция шейдера через 3 секунды после входа в мир (как в orbital-laser)
    fn warm_shader(&mut self) {
        if self.active {
            self.warming = false;
            return;
        }
        if self.warming {
            render::clear_post();
            self.warming = false;
        } else if !self.compiled && self.ticks_in_world >= 60 {
            render::set_post(VOID_SHADER);
            render::set_params(&[0.0; 16]);
            self.compiled = true;
            self.warming = true;
        }
    }

    /// Запуск или досрочное снятие Расширения Территории по нажатию Y
    fn toggle_domain(&mut self) {
        if self.active {
            self.stop_domain();
            return;
        }
        let Some(p) = game::get_player() else { return };

        self.active = true;
        self.domain_ticks = 0;
        self.center = [p.x, p.y, p.z];
        self.center_block = [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32];
        self.dome_y_cursor = DOMAIN_RADIUS;
        self.dome_built = false;
        self.cmd_queue.clear();

        // Передаем координаты центра и радиус 100 блоков в шейдер
        self.params = [0.0; 16];
        self.params[0] = p.x as f32;
        self.params[1] = p.y as f32;
        self.params[2] = p.z as f32;
        self.params[3] = 0.0; // Время в секундах (обновляется в frame)
        self.params[4] = DOMAIN_RADIUS as f32;
        self.params[5] = 1.0; // Флаг активности

        render::set_post(VOID_SHADER);
        self.compiled = true;
        render::set_params(&self.params);

        // Вспышка тряски камеры в момент скрещивания пальцев
        render::shake(0.85, 1.2);

        let [cx, cy, cz] = self.center_block;

        // 1. Запускаем кастомную озвучку Годжо (если положен файл gojo_domain.ogg) + ванильные слои космоса/баса
        world::play_sound("unlimited_void:gojo_domain", None, 2.0, 1.0);
        world::play_sound("minecraft:block.beacon.activate", None, 1.5, 0.5);
        world::play_sound("minecraft:entity.warden.sonic_boom", None, 1.2, 0.55);
        world::play_sound("minecraft:block.end_portal.spawn", None, 1.5, 0.5);

        // 2. Титул на весь экран в стиле аниме + прогрузка чанков на 100 блоков
        let init_cmds = vec![
            format!("forceload add {} {} {} {}", cx - 100, cz - 100, cx + 100, cz + 100),
            r#"title @s times 5 45 15"#.to_string(),
            r#"title @s title {"text":"領域展開「無量空処」","color":"aqua","bold":true}"#.to_string(),
            r#"title @s subtitle {"text":"РАСШИРЕНИЕ ТЕРРИТОРИИ: БЕСКОНЕЧНАЯ ПУСТОТА","color":"white","bold":true}"#.to_string(),
            // Выдаем самому Годжо ночное зрение, левитацию/полет и неуязвимость внутри его домена
            "effect give @s minecraft:night_vision 25 0 true".to_string(),
            "effect give @s minecraft:resistance 25 4 true".to_string(),
        ];
        world::run_commands(&init_cmds);

        log::write(log::Level::Info, &format!("Unlimited Void activated at {cx} {cy} {cz} (R=100)"));
    }

    /// Генерирует команды постройки сферического барьера радиусом 100 блоков сверху вниз
    fn queue_next_dome_rings(&mut self) {
        if self.dome_built {
            return;
        }
        let [cx, cy, cz] = self.center_block;
        let r = DOMAIN_RADIUS;
        let r_sq = r * r;

        for _ in 0..RINGS_PER_TICK {
            if self.dome_y_cursor < -DOMAIN_RADIUS {
                self.dome_built = true;
                // Мощный басовый удар, когда барьер на 100 блоков полностью сомкнулся
                world::play_sound("minecraft:entity.generic.explode", None, 1.5, 0.5);
                render::shake(1.0, 0.8);
                break;
            }
            let dy = self.dome_y_cursor;
            self.dome_y_cursor -= 1;

            let wy = cy + dy;
            if !(-63..=318).contains(&wy) {
                continue;
            }

            // Радиус горизонтального кольца на высоте dy: r_ring = sqrt(R^2 - dy^2)
            let ring_r_sq = r_sq - dy * dy;
            let ring_r = (ring_r_sq as f32).sqrt() as i32;

            // Строим кольцо по точкам окружности (Алгоритм Брезенхэма/шага угла без перегрузки чанков)
            let steps = (ring_r * 6).max(24);
            for step in 0..steps {
                let angle = (step as f32) * std::f32::consts::TAU / (steps as f32);
                let dx = (angle.cos() * (ring_r as f32)).round() as i32;
                let dz = (angle.sin() * (ring_r as f32)).round() as i32;
                let bx = cx + dx;
                let bz = cz + dz;

                // Чередуем черный обсидиан и плачущий обсидиан (дает фиолетовые пятна свечения на стенах)
                let block = if (bx + wy + bz).rem_euclid(7) == 0 {
                    "minecraft:crying_obsidian"
                } else {
                    "minecraft:obsidian"
                };
                // replace air — меняем только воздух, чтобы не ломать существующие постройки игрока!
                self.cmd_queue.push_back(format!(
                    "fill {} {} {} {} {} {} {} replace minecraft:air",
                    bx, wy, bz, bx + 1, wy, bz + 1, block
                ));
            }
        }
    }

    /// Замораживает всех мобов в радиусе 100 блоков и вешает над их головами формулы бесконечных знаний
    fn apply_infinite_knowledge_to_mobs(&mut self) {
        let [cx, cy, cz] = self.center_block;
        let formula = KNOWLEDGE_FORMULAS[self.formula_cursor % KNOWLEDGE_FORMULAS.len()];
        self.formula_cursor += 1;

        let mob_cmds = vec![
            // 1. Полный стан всех мобов в радиусе 100 блоков (кроме самого игрока)
            format!(
                "execute positioned {cx} {cy} {cz} run effect give @e[distance=1..100,type=!minecraft:player,type=!minecraft:item] minecraft:slowness 3 255 true"
            ),
            format!(
                "execute positioned {cx} {cy} {cz} run effect give @e[distance=1..100,type=!minecraft:player,type=!minecraft:item] minecraft:Jump_boost 3 250 true"
            ),
            format!(
                "execute positioned {cx} {cy} {cz} run effect give @e[distance=1..100,type=!minecraft:player,type=!minecraft:item] minecraft:glowing 3 1 true"
            ),
            // 2. Выводим формулы Бесконечной Пустоты прямо в CustomName над головой каждого моба в радиусе 100 блоков!
            format!(
                r#"execute positioned {cx} {cy} {cz} as @e[distance=1..100,type=!minecraft:player,type=!minecraft:item] run data merge entity @s {{CustomName:'"{formula}"',CustomNameVisible:1b,NoAI:1b}}"#
            ),
            // 3. Спавним вокруг головы каждого моба спиральные частицы знаний (enchant + end_rod + sonic_boom)
            format!(
                "execute positioned {cx} {cy} {cz} at @e[distance=1..100,type=!minecraft:player,type=!minecraft:item] run particle minecraft:enchant ~ ~1.9 ~ 0.4 0.5 0.4 1.5 25 force"
            ),
            format!(
                "execute positioned {cx} {cy} {cz} at @e[distance=1..100,type=!minecraft:player,type=!minecraft:item] run particle minecraft:end_rod ~ ~2.2 ~ 0.25 0.25 0.25 0.02 4 force"
            ),
        ];

        world::run_commands(&mob_cmds);
    }

    /// Завершение территории: убираем шейдер, возвращаем мобам ИИ и очищаем купол
    fn stop_domain(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        self.cmd_queue.clear();
        render::clear_post();
        render::shake(0.6, 0.5);

        let [cx, cy, cz] = self.center_block;
        world::play_sound("minecraft:block.glass.break", None, 1.5, 0.6);
        world::play_sound("minecraft:entity.warden.heartbeat", None, 1.0, 0.8);

        // Снимаем NoAI и убираем купол обсидиана обратно в воздух
        let mut cleanup = vec![
            format!(
                r#"execute positioned {cx} {cy} {cz} as @e[distance=1..105,type=!minecraft:player,type=!minecraft:item] run data merge entity @s {{CustomNameVisible:0b,NoAI:0b}}"#
            ),
            format!("forceload remove {} {} {} {}", cx - 100, cz - 100, cx + 100, cz + 100),
            r#"title @s actionbar {"text":"Территория развеяна","color":"aqua"}"#.to_string(),
        ];

        // Очищаем построенную сферу слоями по высоте
        let min_y = (cy - DOMAIN_RADIUS).max(-63);
        let max_y = (cy + DOMAIN_RADIUS).min(318);
        for y_slice in (min_y..=max_y).step_by(10) {
            let y_top = (y_slice + 9).min(max_y);
            cleanup.push(format!(
                "fill {} {} {} {} {} {} minecraft:air replace minecraft:obsidian",
                cx - 100, y_slice, cz - 100, cx + 100, y_top, cz + 100
            ));
            cleanup.push(format!(
                "fill {} {} {} {} {} {} minecraft:air replace minecraft:crying_obsidian",
                cx - 100, y_slice, cz - 100, cx + 100, y_top, cz + 100
            ));
        }

        world::run_commands(&cleanup);
        hud::clear();
    }

    fn update_hud(&self) {
        if !self.active {
            return;
        }
        let sec = self.domain_ticks as f32 * 0.05;
        let rem = (DOMAIN_LENGTH - sec).max(0.0);
        let panel = vec![
            Line { text: "領域展開 : UNLIMITED VOID".to_string(), color: 0x55FFFF },
            Line { text: format!("Радиус купола: {} блоков", DOMAIN_RADIUS), color: 0xFFFFFF },
            Line { text: format!("Черная дыра: АКТИВНА ({:.1}с)", rem), color: 0x82B1FF },
            Line { text: "Разум врагов: ПЕРЕГРУЖЕН (∞)".to_string(), color: 0xCE93D8 },
            Line { text: "Нажми [Y] чтобы развеять".to_string(), color: 0x90A4AE },
        ];
        hud::set_panel(Corner::TopRight, &panel);
    }
}

impl Mod for UnlimitedVoid {
    fn init() -> Result<(Self, Registration), String> {
        // Регистрируем клавишу Y в управлении Horizon
        input::register_key("activate_void", "Gojo: Unlimited Void (Domain Expansion)", "key.keyboard.y");
        Ok((
            UnlimitedVoid::new(),
            Registration::new().command("void").on_tick().on_frame(),
        ))
    }

    fn tick(&mut self, _tick: u64) {
        if game::get_player().is_none() {
            if self.active {
                self.active = false;
                render::clear_post();
            }
            return;
        }
        self.ticks_in_world += 1;

        // Проверяем нажатие клавиши Y
        for event in input::poll() {
            if let Event::KeyDown(k) = event
                && k == "activate_void"
            {
                self.toggle_domain();
            }
        }

        if !self.active {
            return;
        }

        self.domain_ticks += 1;
        let now = self.domain_ticks as f32 * 0.05;

        // Начиная с 1.0 секунды (после фразы "Ryoiki Tenkai" и вспышки) строим купол 100 блоков
        if now >= 1.0 && !self.dome_built {
            self.queue_next_dome_rings();
        }

        // Выполняем до 120 команд постройки барьера за тик (чтобы было 0 фризов)
        let mut batch = Vec::new();
        while batch.len() < 120 {
            let Some(c) = self.cmd_queue.pop_front() else { break };
            batch.push(c);
        }
        if !batch.is_empty() {
            world::run_commands(&batch);
        }

        // Каждые 4 тика (0.2 секунды) обновляем формулы знаний над головами мобов и спавним частицы
        if self.domain_ticks % 4 == 0 {
            self.apply_infinite_knowledge_to_mobs();
        }

        // Звуковая пульсация Черной дыры каждые 2.5 секунды
        if self.domain_ticks % 50 == 0 {
            world::play_sound("minecraft:block.beacon.ambient", None, 1.2, 0.55);
            render::shake(0.18, 0.4);
        }

        self.update_hud();

        if now >= DOMAIN_LENGTH {
            self.stop_domain();
        }
    }

    fn frame(&mut self, _seconds: f32) {
        self.warm_shader();
        if self.active {
            self.params[3] = self.domain_ticks as f32 * 0.05;
            if let Some(p) = game::get_player() {
                self.params[6] = p.x as f32;
                self.params[7] = p.y as f32;
                self.params[8] = p.z as f32;
                self.params[9] = p.yaw;
                self.params[10] = p.pitch;
            }
            render::set_params(&self.params);
        }
    }

    fn command(&mut self, _name: &str, _args: &str) -> Result<Option<String>, String> {
        self.toggle_domain();
        Ok(Some("Gojo Domain Expansion: Unlimited Void toggled!".into()))
    }
}

horizon_mod_sdk::export_mod!(UnlimitedVoid);