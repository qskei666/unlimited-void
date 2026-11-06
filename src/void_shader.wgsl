// Шейдер Бесконечной Пустоты Годжо Сатору для Horizon Mod SDK

struct StageIn {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var scene_tex: texture_2d<f32>;
@group(0) @binding(1) var scene_samp: sampler;
@group(0) @binding(2) var<uniform> params: array<vec4<f32>, 4>;

fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise2d(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash21(i + vec2<f32>(0.0, 0.0)), hash21(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash21(i + vec2<f32>(0.0, 1.0)), hash21(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y
    );
}

// Фрактальный шум для генерации белых пятен и клякс как на фоне в аниме
fn fbm(p: vec2<f32>) -> f32 {
    var v = 0.0;
    var a = 0.5;
    var shift = vec2<f32>(100.0, 100.0);
    var coord = p;
    for (var i = 0; i < 5; i++) {
        v += a * noise2d(coord);
        coord = coord * 2.1 + shift;
        a *= 0.5;
    }
    return v;
}

@fragment
fn fs_main(in: StageIn) -> @location(0) vec4<f32> {
    let time = params[0].w;
    let active = params[1].y;
    let base_col = textureSample(scene_tex, scene_samp, in.uv);

    if (active < 0.5 || time <= 0.001) {
        return base_col;
    }

    let centered_uv = (in.uv - vec2<f32>(0.5, 0.5)) * vec2<f32>(1.77, 1.0);
    let dist_center = length(centered_uv);

    // =========================================================================
    // ФАЗА 1 (0.0 - 1.1 сек): IMPACT FRAME (Скрещивание пальцев + Шесть Глаз)
    // =========================================================================
    if (time < 1.1) {
        let luma = dot(base_col.rgb, vec3<f32>(0.299, 0.587, 0.114));
        let angle = atan2(centered_uv.y, centered_uv.x);
        let speed_lines = sin(angle * 45.0 + time * 30.0);

        // Резкий черно-белый контраст в стиле Jujutsu Kaisen
        let bw = select(0.02, 0.98, luma + speed_lines * 0.15 > 0.45);
        var impact_col = vec3<f32>(bw);

        // Голубая вспышка Проклятой Энергии Годжо из центра
        let blue_flash = exp(-dist_center * 3.2) * sin(time * 2.85);
        impact_col = mix(impact_col, vec3<f32>(0.2, 0.75, 1.0), blue_flash * 0.8);

        return vec4<f32>(impact_col, 1.0);
    }

    // =========================================================================
    // ФАЗА 2 (1.1 - 22.0 сек): ЧЕРНАЯ ДЫРА И КОСМОС БЕСКОНЕЧНОЙ ПУСТОТЫ
    // =========================================================================
    let yaw = params[2].y * 0.0174533;
    let pitch = params[2].z * 0.0174533;

    // Привязываем положение Черной дыры к небу, чтобы при повороте камеры она стояла в пространстве купола
    let bh_offset = vec2<f32>(sin(yaw) * 0.45, sin(pitch) * 0.45 - 0.12);
    let bh_vec = centered_uv - bh_offset;
    let r = length(bh_vec);

    // Гравитационное линзирование: искажаем картинку мира вокруг горизонта событий
    let event_horizon = 0.19;
    let lens_pull = (0.018 / max(r - event_horizon * 0.85, 0.012)) * smoothstep(0.85, 0.15, r);
    let warped_uv = clamp(in.uv - normalize(bh_vec + vec2<f32>(0.0001)) * lens_pull, vec2<f32>(0.0), vec2<f32>(1.0));
    let world_warped = textureSample(scene_tex, scene_samp, warped_uv).rgb;

    // 1. Фон космической Бездны + белые пятна-кляксы как в аниме
    let space_coord = (centered_uv + vec2<f32>(yaw * 0.6, pitch * 0.6)) * 3.8;
    let n1 = fbm(space_coord + vec2<f32>(time * 0.07, -time * 0.05));
    let n2 = fbm(space_coord * 2.3 - vec2<f32>(time * 0.12, time * 0.08));

    // Резкие белые аниме-пятна и мягкие синие туманности
    let white_spots = smoothstep(0.64, 0.69, n1) * 0.85;
    let cyan_spots = smoothstep(0.58, 0.65, n2) * 0.55;
    let ink_streaks = smoothstep(0.82, 0.95, sin(space_coord.x * 12.0 + n1 * 8.0 + time * 2.0));

    var void_bg = vec3<f32>(0.015, 0.02, 0.05); // Глубокий космос
    void_bg += vec3<f32>(0.10, 0.32, 0.75) * n1 * 0.9; // Синее свечение Бездны
    void_bg += vec3<f32>(0.40, 0.80, 1.00) * cyan_spots;
    void_bg += vec3<f32>(0.95, 0.98, 1.00) * white_spots; // Белые пятна из аниме
    void_bg += vec3<f32>(0.70, 0.90, 1.00) * ink_streaks * 0.4;

    // 2. Аккреционный диск и Фотонное кольцо Черной Дыры
    let swirl_angle = atan2(bh_vec.y, bh_vec.x) + time * 0.6 + 1.2 / (r + 0.05);
    let disk_noise = fbm(vec2<f32>(cos(swirl_angle), sin(swirl_angle)) * 3.0 + r * 8.0);

    let photon_ring = exp(-pow((r - event_horizon) * 38.0, 2.0));
    let accretion_glow = (0.032 / max(abs(r - event_horizon), 0.008)) * (0.6 + 0.7 * disk_noise);

    var bh_color = void_bg;
    bh_color += vec3<f32>(0.30, 0.68, 1.0) * accretion_glow;
    bh_color += vec3<f32>(1.0, 1.0, 1.0) * photon_ring * 3.0; // Ослепительно белое кольцо вокруг дыры

    // Внутри горизонта событий — абсолютная чернота
    if (r < event_horizon) {
        bh_color = vec3<f32>(0.0, 0.0, 0.0);
    }

    // 3. Смешиваем мир Майнкрафта (силуэты мобов и земли) с космосом Бесконечной Пустоты
    // Темные блоки обсидианового купола автоматически заменяются на космос и Черную Дыру!
    let world_luma = dot(world_warped, vec3<f32>(0.299, 0.587, 0.114));
    let is_dark_dome = smoothstep(0.18, 0.03, world_luma);

    // Окрашиваем мобов и рельеф в холодные тона Бесконечности
    let tinted_world = mix(world_warped, vec3<f32>(0.35, 0.75, 1.0) * world_luma * 1.6, 0.65);

    var final_col = mix(tinted_world, bh_color, max(is_dark_dome, smoothstep(0.42, 0.18, r)));

    // Добавляем поверх ослепительное белое кольцо Черной дыры, чтобы оно сияло поверх всего
    if (r >= event_horizon) {
        final_col += vec3<f32>(0.85, 0.95, 1.0) * photon_ring * 1.8;
    } else {
        final_col = vec3<f32>(0.0, 0.0, 0.0);
    }

    return vec4<f32>(final_col, 1.0);
}